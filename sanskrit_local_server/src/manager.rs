//extern crate blake2_rfc;

use sanskrit_sled_store::SledStore;

use std::collections::{BTreeMap, BTreeSet};
use sanskrit_deploy::{deploy_module, deploy_function};
use sanskrit_runtime::{execute, Tracker, read_elem, read_transaction_desc, CONFIG};
use sanskrit_common::store::*;
use sanskrit_common::encoding::*;
use sanskrit_common::model::*;
use sanskrit_common::errors::*;

use sled::{Db, IVec, Error};
use rand::rngs::OsRng;
use ed25519_dalek::{Keypair, Signature, Signer};

use hex::encode;
use sanskrit_common::arena::Heap;
use sanskrit_common::hashing::HashingDomain;

use sanskrit_runtime::model::{TransactionBundle, DeployTransaction, DeployType, ParamRef, ParamMode, RetType, BundleSection, SectionType, Transaction, TransactionBundleCore};
use sanskrit_core::accounting::Accounting;
use std::cell::Cell;
use sanskrit_compile::compile_function;
use sanskrit_compile::limiter::Limiter;
use sanskrit_interpreter::model::{Entry, TxTParam, TxTReturn, TransactionDescriptor, ValueSchema, Adt};
use externals::{ServerExternals, ServerSystem};
use std::time::Instant;
use std::ops::Deref;
use std::convert::TryInto;
use std::str::from_utf8;
use sanskrit_core::model::Module;
use convert_error;

pub enum Param {
    Lit(Vec<u8>),
    Sig(String),       //Signs with account with id x
    Pk(String),        //Produces a Pk literal for account with id x
    Consume(Hash),
    Borrow(Hash),
    Copy(Hash),
    Provided
}

pub enum Ret {
    Log,
    Elem,
    Drop,
}

pub struct ExecutionState {
    pub consumed_elems:BTreeSet<String>,
    pub produced_elems:BTreeMap<String, (Hash,String)>,
    pub param_names:Vec<String>,
    pub return_names:Vec<String>,
    pub success:bool,
}

//NOTE: THIS WORKS ONLY WITH A SINGLE TXT
// IF WE HAVE FULL BUNDLE IT NEEDS TO BE IMPROVED
pub struct TrackingState {
    pub exec_state:ExecutionState,
    pub active_elems:Db,
    pub element_data:Db,
    pub data_names:Db,
}

pub struct State {
    pub store:SledStore,
    pub system:ServerSystem,
    pub accounts:Db,
    pub system_entries:Db,
    pub module_name_mapping:Db,
    pub transaction_name_mapping:Db,
    pub tracking:TrackingState
}

#[derive(Debug, Parsable, Serializable)]
pub struct CtrName(pub EncString, pub u8);
#[derive(Debug, Parsable, Serializable)]
pub struct DataNames(pub EncString, pub Vec<CtrName>);
#[derive(Debug, Parsable, Serializable)]
pub struct ModuleNames(pub EncString, pub Vec<DataNames>);

#[derive(Debug)]
pub struct EncString(pub String);
impl<'a> Parsable<'a> for EncString{
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        let len = u16::parse(p,alloc)?;
        let string_data = p.consume_bytes(len as usize)?;
        Ok(EncString(convert_error(String::from_utf8(string_data.to_owned()))?))
    }
}

impl Serializable for EncString{
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        assert!(self.0.len() <= u16::max_value() as usize);
        (self.0.len() as u16).serialize(s)?;
        s.produce_bytes(self.0.as_bytes());
        Ok(())
    }
}

impl ExecutionState {
    pub fn new(param_names:Vec<String>, return_names:Vec<String>) -> Self {
        ExecutionState {
            consumed_elems: BTreeSet::new(),
            produced_elems:  BTreeMap::new(),
            param_names,
            return_names,
            success:false
        }
    }
}

fn pretty_print_data(value:&Entry, desc:&ValueSchema) -> String {
    match *desc {
        ValueSchema::Adt(ctrs) => {
            let Adt(tag, fields) = unsafe {value.adt};

            //if their are zero fields we omit the fields
            let ctr =  ctrs[tag as usize];
            assert_eq!(fields.len(), ctr.len());
            let mut string = String::new();
            if ctr.len() != 1 {
                string.push_str(&format!("({}|", tag))
            } else {
                string.push_str(&format!("(|"))
            }
            let mut first = true;
            for (f_value, f_schema) in fields.iter().zip(ctr.iter()) {
                if !first {
                    string.push_str(", ")
                }
                string.push_str(&pretty_print_data(f_value, f_schema));
                first = false;
            }
            string.push_str(")");
            string
        },
        ValueSchema::Data(_) =>  format!("0x{}",encode(unsafe {value.data}.deref())),
        ValueSchema::Unsigned(1) => format!("{}",unsafe {value.u8}),
        ValueSchema::Unsigned(2) => format!("{}",unsafe {value.u16}),
        ValueSchema::Unsigned(4) => format!("{}",unsafe {value.u32}),
        ValueSchema::Unsigned(8) => format!("{}",unsafe {value.u64}),
        ValueSchema::Unsigned(16) => format!("{}",unsafe {value.u128}),
        ValueSchema::Signed(1) => format!("{}",unsafe {value.i8}),
        ValueSchema::Signed(2) => format!("{}",unsafe {value.i16}),
        ValueSchema::Signed(4) => format!("{}",unsafe {value.i32}),
        ValueSchema::Signed(8) => format!("{}",unsafe {value.i64}),
        ValueSchema::Signed(16) => format!("{}",unsafe {value.i128}),
        _ => unreachable!()
    }
}


impl Tracker for TrackingState {
    fn section_start(&mut self, section: &BundleSection) {  }
    fn transaction_start(&mut self, transaction: &Transaction) { }

    fn parameter_load(&mut self, p_ref: &ParamRef, p_desc: &TxTParam, value: &Entry) {
        let name = self.exec_state.param_names.pop().unwrap();
        match p_ref {
            ParamRef::Load(ParamMode::Consume, _) => {
                self.exec_state.consumed_elems.insert(name);
            },
            _ => {},
        };
    }

    fn return_value(&mut self, r_typ:&RetType, r_desc:&TxTReturn, value:&Entry){
        let name = self.exec_state.return_names.pop().unwrap();
        match r_typ {
            RetType::Store => {
                let id = unsafe {value.adt.1.get(0).unwrap().data.deref()}.try_into().unwrap();
                let pretty = pretty_print_data(value, &r_desc.desc);
                println!("store({}): {}",name, pretty);
                self.exec_state.produced_elems.insert(name, (id, pretty));
            },
            RetType::Drop => {},
            RetType::Log => {
                println!("log({}): {}",name, pretty_print_data(value, &r_desc.desc))
            },
        };
    }

    fn transaction_finish(&mut self, transaction: &Transaction, success: bool) { }
    fn section_finish(&mut self, section: &BundleSection, success: bool) {
        self.exec_state.success = success;
        if success  {
            for elem in &self.exec_state.consumed_elems {
                self.active_elems.remove(elem).unwrap();
                self.element_data.remove(elem).unwrap();
            }
            for (name,(id,pretty)) in &self.exec_state.produced_elems {
                let data = pretty.clone().into_bytes();
                match self.active_elems.insert(name.clone(), id) {
                    Ok(None) => {},
                    Ok(Some(x)) => println!("Warning: {} had a previous mapping that can no longer be accessed over this tool (it remains in the state)", name),
                    Err(x) => Err(x).unwrap(),
                }
                self.element_data.insert(name.clone(), data).unwrap();
            }
            self.active_elems.flush().unwrap();
            self.element_data.flush().unwrap();
        }
    }
}

const MAX_PARSE_DEPTH:usize = 1024;
const SIG_SIZE:usize = 64; //Check this
const SIG_RUNTIME_SIZE:usize = SIG_SIZE + Entry::SIZE;


impl State {

    fn max_accounting(input_limit:usize) -> Accounting{
        Accounting{
            load_byte_budget: Cell::new(usize::max_value()),
            store_byte_budget: Cell::new(usize::max_value()),
            process_byte_budget: Cell::new(usize::max_value()),
            stack_elem_budget: Cell::new(usize::max_value()),
            //these two are a bit counter intuitive
            max_nesting: Cell::new(0),
            nesting_limit: usize::max_value(),
            input_limit
        }
    }

    fn max_limiter() -> Limiter {
        Limiter {
            max_functions: usize::max_value(),
            max_nesting: usize::max_value(),
            max_used_nesting: Cell::new(0),
            produced_functions: Cell::new(0)
        }
    }

    fn deploy_only_bundle(typ:DeployType, data: &[u8], eval_accounting:Accounting, eval_limiter:Option<Limiter>) -> TransactionBundle {
        let (max_contained_functions, max_compile_block_nesting) = match eval_limiter {
            None => (0,0),
            Some(limiter) => (limiter.produced_functions.get() as u32, limiter.max_used_nesting.get() as u32),
        };

        TransactionBundle {
            byte_size: None,
            core: TransactionBundleCore {
                byte_size: None,
                meta: SlicePtr::empty(),
                earliest_block: 0,
                deploy: Some(DeployTransaction{
                    typ,
                    data: SlicePtr::wrap(data),
                    max_load_bytes: (usize::max_value() - eval_accounting.load_byte_budget.get()) as u32,
                    max_store_bytes: (usize::max_value() - eval_accounting.store_byte_budget.get()) as u32,
                    max_process_bytes: (usize::max_value() - eval_accounting.process_byte_budget.get()) as u32,
                    max_stack_elems: (usize::max_value() - eval_accounting.stack_elem_budget.get()) as u32,
                    max_block_nesting: eval_accounting.max_nesting.get() as u32,
                    //We do not compile anything so we can leave 0
                    max_contained_functions,
                    max_compile_block_nesting
                }),
                //All these are empty because we do not have a payment txt
                param_heap_limit: 0,
                transaction_storage_heap: 0,
                stack_elem_limit: 0,
                stack_frame_limit: 0,
                runtime_heap_limit: 0,
                sections: SlicePtr::empty(),
                descriptors: SlicePtr::empty(),
                stored: SlicePtr::empty(),
                literal:  SlicePtr::empty(),
                witness_bytes_limit: 0,
            },
            witness:  SlicePtr::empty(),
            store_witness:  SlicePtr::empty()
        }
    }

    pub fn execute_bundle(&mut self, bundle:&[u8], system_mode_on:bool) -> Result<()> {
        let mut heap = Heap::new(100000000,2.0);
        execute::<_,_,ServerExternals, _>(&self.store, &self.system,&bundle,0, &heap, &mut self.tracking, system_mode_on)
    }

    pub fn deploy_module(&mut self, module:Vec<u8>, system_mode_on:bool) -> Result<Hash> {
        let now = Instant::now();
        let hash1 = store_hash(&[&module]);
        if !self.store.contains(StorageClass::Module, &hash1){
            //Estimation test run
            let accounting = Self::max_accounting(module.len());
            deploy_module(&self.store,&accounting,module.clone(),system_mode_on,false)?;
            self.store.rollback(StorageClass::Module);

            let bundle = Self::deploy_only_bundle(DeployType::Module, &module, accounting, None);

            let mut s = Serializer::new(usize::max_value());
            bundle.serialize(&mut s)?;
            self.execute_bundle(&s.extract(), system_mode_on)?;
            println!("deployed Module with hash {:?} of size {:?} in {:?} ms", encode(&hash1),module.len(),now.elapsed().as_millis());
        }
        Ok(hash1)
    }

    pub fn deploy_transaction(&mut self, transaction:Vec<u8>) -> Result<(Hash,Hash)> {
        let now = Instant::now();

        //Estimation test run
        let accounting = Self::max_accounting(transaction.len());
        let limiter = Self::max_limiter();

        let hash = deploy_function(&self.store,&accounting,transaction.clone(),false)?;
        let (t_hash,t_size) = compile_function::<SledStore,ServerExternals>(&self.store,&accounting,&limiter,hash, false)?;
        self.store.rollback(StorageClass::Transaction);
        self.store.rollback(StorageClass::Descriptor);
        let bundle = Self::deploy_only_bundle(DeployType::Transaction, &transaction, accounting, Some(limiter));
        let mut s = Serializer::new(usize::max_value());
        bundle.serialize(&mut s)?;
        self.execute_bundle(&s.extract(), false)?;
        println!("deployed transaction in {:?} ms", now.elapsed().as_millis());
        println!("  - function with hash {:?} of size {:?}", encode(&hash), transaction.len());
        println!("  - descriptor with hash {:?} of size {:?}", encode(&t_hash),t_size);
        Ok((hash,t_hash))
    }

    pub fn execute_transaction(&mut self, desc:&Hash, params:&[Param], returns:&[Ret]) -> Result<()>{
        //todo make softer exits
        //todo: improve overall heap management only alloc 1 Heap
        let mut heap = Heap::new(100000000,2.0);
        let mut full_heap = heap.new_virtual_arena(100000 as usize);
        let txt_desc = read_transaction_desc(desc,&self.store, &full_heap)?;
        if txt_desc.params.len() != params.len() {
            panic!("Number of supplied parameter missmatches")
        }
        if txt_desc.returns.len() != returns.len() {
            panic!("Number of supplied returns missmatches")
        }
        //todo: build on Heap
        let mut lits:Vec<SlicePtr<u8>> = Vec::new();
        let mut lit_dedup:BTreeMap<Vec<u8>,u16> = BTreeMap::new();

        let mut sigs:Vec<String> = Vec::new();
        let mut sigs_dedup:BTreeMap<String,u16> = BTreeMap::new();

        let mut stores:Vec<Hash> = Vec::new();
        let mut stores_dedup:BTreeMap<Hash,u16> = BTreeMap::new();
        let mut store_witnesses:Vec<Option<SlicePtr<u8>>> = Vec::new();

        let mut txt_params:Vec<ParamRef> = Vec::with_capacity(params.len());

        let mut entries_loaded = 0;
        let mut entries_created = 0;
        let mut entries_deleted = 0;
        let mut param_heap = 0;

        let mut wit_bytes = 0;

        //todo: make collect
        for (p, txt_p) in params.iter().zip(txt_desc.params.iter()) {
            match p {
                Param::Lit(data) => {
                    if lit_dedup.contains_key(data) {
                        txt_params.push(ParamRef::Literal(*lit_dedup.get(data).unwrap()))
                    } else {
                        param_heap += data.len();
                        lits.push(full_heap.copy_alloc_slice(&data)?);
                        lit_dedup.insert(data.clone(), (lits.len()-1) as u16);
                        txt_params.push(ParamRef::Literal((lits.len()-1) as u16))
                    }
                },

                Param::Pk(account) => {
                    let data = self.get_account(account)?.public.to_bytes().to_vec();
                    if lit_dedup.contains_key(&data) {
                        txt_params.push(ParamRef::Literal(*lit_dedup.get(&data).unwrap()))
                    } else {
                        param_heap += data.len();
                        lits.push(full_heap.copy_alloc_slice(&data)?);
                        lit_dedup.insert(data, (lits.len()-1) as u16);
                        txt_params.push(ParamRef::Literal((lits.len()-1) as u16))
                    }
                },

                Param::Sig(account) => {
                    if sigs_dedup.contains_key(account) {
                        txt_params.push(ParamRef::Witness(*sigs_dedup.get(account).unwrap()))
                    } else {
                        wit_bytes += (SIG_SIZE+2/*2 is the length*/);
                        param_heap += SIG_RUNTIME_SIZE;
                        sigs.push(account.clone());
                        sigs_dedup.insert(account.clone(), (sigs.len()-1) as u16);
                        txt_params.push(ParamRef::Witness((sigs.len()-1) as u16))
                    }
                }

                Param::Provided => {
                    param_heap += txt_p.desc.max_runtime_size()? as usize;
                    txt_params.push(ParamRef::Provided)
                },

                //todo: share
                Param::Consume(id) => {
                    entries_loaded +=1;
                    entries_deleted +=1;
                    if stores_dedup.contains_key(id) {
                        txt_params.push(ParamRef::Load(ParamMode::Consume,*stores_dedup.get(id).unwrap()))
                    } else {
                        let elem = read_elem(&id, &self.store)?;
                        param_heap += txt_p.desc.runtime_size(&elem)? as usize;
                        wit_bytes += (elem.len()+2+1/*2 is the length, +1 is optional tag*/); //todo: adapt if optional tag is gone
                        store_witnesses.push(Some(full_heap.copy_alloc_slice(&elem)?));
                        stores.push(id.clone());
                        stores_dedup.insert(id.clone(), (stores.len()-1) as u16);
                        txt_params.push(ParamRef::Load(ParamMode::Consume,(stores.len()-1) as u16))
                    }
                },
                Param::Borrow(id) => {
                    entries_loaded +=1;
                    if stores_dedup.contains_key(id) {
                        txt_params.push(ParamRef::Load(ParamMode::Borrow,*stores_dedup.get(id).unwrap()))
                    } else {
                        let elem = read_elem(&id, &self.store)?;
                        param_heap += txt_p.desc.runtime_size(&elem)? as usize;
                        wit_bytes += (elem.len()+2+1/*2 is the length, +1 is optional tag*/); //todo: adapt if optional tag is gone
                        store_witnesses.push(Some(full_heap.copy_alloc_slice(&elem)?));
                        stores.push(id.clone());
                        stores_dedup.insert(id.clone(), (stores.len()-1) as u16);
                        txt_params.push(ParamRef::Load(ParamMode::Borrow,(stores.len()-1) as u16))
                    }
                },
                Param::Copy(id) => {
                    entries_loaded +=1;
                    if stores_dedup.contains_key(id) {
                        txt_params.push(ParamRef::Load(ParamMode::Copy,*stores_dedup.get(id).unwrap()))
                    } else {
                        let elem = read_elem(&id, &self.store)?;
                        param_heap += txt_p.desc.runtime_size(&elem)? as usize;
                        wit_bytes += (elem.len()+2+1/*2 is the length, +1 is optional tag*/); //todo: adapt if optional tag is gone
                        store_witnesses.push(Some(full_heap.copy_alloc_slice(&elem)?));
                        stores.push(id.clone());
                        stores_dedup.insert(id.clone(), (stores.len()-1) as u16);
                        txt_params.push(ParamRef::Load(ParamMode::Copy,(stores.len()-1) as u16))
                    }
                }
            }
        }

        let mut txt_rets:Vec<RetType> = Vec::with_capacity(returns.len());
        //todo: make collect
        for r in returns {
            match r {
                Ret::Log => txt_rets.push(RetType::Log),
                Ret::Elem => {
                    entries_created +=1;
                    txt_rets.push(RetType::Store)
                },
                Ret::Drop => txt_rets.push(RetType::Drop)
            }
        }

        //Todo: build bundle section then bundle
        let transaction = Transaction {
            txt_desc: 0,
            params: full_heap.copy_alloc_slice(&txt_params)?,
            returns: full_heap.copy_alloc_slice(&txt_rets)?,
        };
        let section = BundleSection {
            typ: SectionType::Custom,
            entries_loaded,
            entries_created,
            entries_deleted,
            gas_limit: txt_desc.gas_cost as u64,
            txts: full_heap.copy_alloc_slice(&[transaction])?
        };

        let bundle_core = TransactionBundleCore {
            byte_size: None,
            deploy: None,
            meta: SlicePtr::empty(),
            earliest_block:0,
            param_heap_limit: param_heap as u16,
            //This is to low why -- oh I see, this is byte size we need parsed size -- can we inject in derive -- ?
            // Alla alloc Size??
            transaction_storage_heap: txt_desc.virt_size.unwrap() as u16,
            stack_elem_limit: txt_desc.max_stack,
            stack_frame_limit: txt_desc.max_frames,
            runtime_heap_limit: txt_desc.max_mem, //why we need that much more mem
            sections: full_heap.copy_alloc_slice(&[section])?,
            descriptors: full_heap.copy_alloc_slice(&[*desc])?,
            stored: full_heap.copy_alloc_slice(&stores)?,
            literal: full_heap.copy_alloc_slice(&lits)?,
            witness_bytes_limit: wit_bytes as u32,
        };

        let bundle_hash = HashingDomain::Bundle.hash(&Serializer::serialize_fully(&bundle_core,MAX_PARSE_DEPTH)?); //todo: is the max here correct

        let mut witness:Vec<SlicePtr<u8>> = Vec::with_capacity(sigs.len());

        for w in &sigs {
            let keys = self.get_account(w)?;
            let signature: Signature = keys.sign(&bundle_hash);
            witness.push(full_heap.copy_alloc_slice(&signature.to_bytes())?);
        };

        let bundle = TransactionBundle {
            byte_size: None,
            core: bundle_core,
            witness: full_heap.copy_alloc_slice(&witness)?,
            store_witness: full_heap.copy_alloc_slice(&store_witnesses)?,
        };

        self.execute_bundle(&Serializer::serialize_fully(&bundle,MAX_PARSE_DEPTH)?, false)
    }

    pub fn get_account(&mut self, ident:&str) -> Result<Keypair> {
        let key = ident.as_bytes();
        if convert_error(self.accounts.contains_key(&key))? {
            return convert_error(Keypair::from_bytes(&convert_error(self.accounts.get(&key))?.unwrap()))
        }
        let mut csprng = OsRng{};
        let kp = Keypair::generate(&mut csprng);
        convert_error(self.accounts.insert(key,kp.to_bytes().to_vec()))?;
        convert_error(self.accounts.flush())?;
        return Ok(kp);
    }

    pub fn get_accounts(&mut self) -> Result<Vec<(String,Keypair)>> {
        let mut res = Vec::with_capacity(self.accounts.len());
        for account in self.accounts.iter() {
            let (name_bytes,key_bytes) = convert_error(account)?;
            let name = convert_error(from_utf8(&name_bytes))?;
            let keypair = convert_error(Keypair::from_bytes(&key_bytes))?;
            res.push((name.to_owned(), keypair))
        }
        return Ok(res)
    }

    pub fn get_elems(&mut self) -> Result<Vec<(String,String)>> {
        let mut res = Vec::with_capacity(self.tracking.element_data.len());
        for account in self.tracking.element_data.iter() {
            let (name_bytes,data_bytes) = convert_error(account)?;
            let name = convert_error(from_utf8(&name_bytes))?;
            let data = convert_error(from_utf8(&data_bytes))?;
            res.push((name.to_owned(), data.to_owned()))
        }
        return Ok(res)
    }

    pub fn get_elem(&mut self, ident:&str) -> Result<String> {
        let data_bytes = convert_error(self.tracking.element_data.get(ident))?.unwrap();
        let data = convert_error(from_utf8(&data_bytes))?;
        return Ok(data.to_owned())
    }

    pub fn get_transactions(&mut self) -> Result<Vec<String>> {
        let mut res = Vec::with_capacity(self.transaction_name_mapping.len());
        for transaction in self.transaction_name_mapping.iter() {
            let (name_bytes,_) = convert_error(transaction)?;
            let name = convert_error(from_utf8(&name_bytes))?;
            res.push(name.to_owned())
        }
        return Ok(res)
    }

    pub fn get_transaction<'a:'b, 'b, A:ParserAllocator>(&'b mut self, ident:&str,  heap:&'a A) -> Result<TransactionDescriptor> {
        let hash_bytes = convert_error(self.transaction_name_mapping.get(ident))?.unwrap();
        let id = hash_from_slice(&hash_bytes);
        read_transaction_desc(&id, &self.store,  heap)
    }

    pub fn get_modules(&mut self) -> Result<Vec<String>> {
        let mut res = Vec::with_capacity(self.module_name_mapping.len());
        for module in self.module_name_mapping.iter() {
            let (name_bytes,_) = convert_error(module)?;
            let name = convert_error(from_utf8(&name_bytes))?;
            res.push(name.to_owned())
        }
        return Ok(res)
    }

    pub fn get_module<'a:'b, 'b, A:ParserAllocator>(&'b mut self, ident:&str,  heap:&'a A) -> Result<Module> {
        let hash_bytes = convert_error(self.module_name_mapping.get(ident))?.unwrap();
        let id = hash_from_slice(&hash_bytes);
        self.store.parsed_get(StorageClass::Module, &id, CONFIG.max_structural_dept, heap)
    }

}

