//extern crate blake2_rfc;

use sanskrit_sled_store::SledStore;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use sanskrit_runtime::{execute, Tracker, CONFIG, read_transaction_desc, Context};
#[cfg(feature = "embedded")]
use sanskrit_runtime::deploy;
use sanskrit_common::store::*;
use sanskrit_common::encoding::*;
use sanskrit_common::model::*;
use sanskrit_common::errors::*;

use sled::Db;
use rand::rngs::OsRng;
use ed25519_dalek::{Keypair, Signature, Signer};

use hex::encode;
use sanskrit_common::arena::{Heap, VirtualHeapArena};
use sanskrit_common::hashing::HashingDomain;

use sanskrit_runtime::model::{ParamRef, ParamMode, RetType, BundleSection, SectionType, Transaction, TransactionBundleCore, BaseTransactionBundle};
#[cfg(feature = "embedded")]
use sanskrit_runtime::model::{DeployTransaction, DeployType};
use sanskrit_interpreter::model::{Entry, TxTParam, TxTReturn, TransactionDescriptor, ValueSchema, Adt};
use externals::{ServerSystem, ServerSystemDataManager, get_ed_dsa_module};
#[cfg(feature = "embedded")]
use externals::ServerExternals;
use std::time::Instant;
use std::ops::{Deref, Add};
use std::convert::TryInto;
use std::str::from_utf8;
use sanskrit_core::model::Module;
use convert_error;
use ed25519_dalek::ed25519::SIGNATURE_LENGTH;
use sanskrit_runtime::system::SystemContext;
use sanskrit_runtime::direct_stored::SystemDataManager;
use externals::crypto::{raw_plain_hash, raw_join_hash};
#[cfg(feature = "wasm")]
use compiler::CompilerInstance;
#[cfg(feature = "embedded")]
use sanskrit_deploy::deploy_module;

pub struct Tx {
    pub desc:Hash,
    pub params:Vec<Param>,
    pub returns:Vec<Ret>
}

pub enum Param {
    Lit(Vec<u8>),
    Sig(String),       //Signs with account with id x
    Pk(String),        //Produces a Pk literal for account with id x
    Subject(String),   //Produces a Subject literal for account with id x
    Consume(Hash),
    Borrow(Hash),
    Copy(Hash),
    LocalConsume(String),
    LocalBorrow(String),
    LocalCopy(String),
    Provided
}

pub enum Ret {
    Log,
    Elem,
    Drop,
    Assign(String),
}

#[derive(Clone)]
pub struct ExecutionState {
    pub consumed_elems:BTreeSet<String>,
    pub produced_elems:BTreeMap<String, (Hash,String)>,
    pub param_names:VecDeque<String>,
    pub return_names:VecDeque<String>,
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
    #[cfg(feature = "wasm")]
    pub instance:CompilerInstance,
    pub store:SledStore,
    pub accounts:Db,
    pub system_entries:Db,
    pub module_name_mapping:Db,
    pub transaction_name_mapping:Db,
    pub tracking:TrackingState,
    pub meta_data:Db,
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
    pub fn new() -> Self {
        ExecutionState {
            consumed_elems: BTreeSet::new(),
            produced_elems:  BTreeMap::new(),
            param_names: VecDeque::new(),
            return_names: VecDeque::new(),
            success:false
        }
    }
}

fn pretty_print_data(value:&Entry, desc:&ValueSchema) -> String {
    match *desc {
        ValueSchema::Adt(_,ctrs) => {
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
            for (f_value, (_,f_schema)) in fields.iter().zip(ctr.iter()) {
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
    fn section_start(&mut self, _section: &BundleSection) {  }
    fn transaction_start(&mut self, _transaction: &Transaction) { }

    fn parameter_load(&mut self, p_ref: &ParamRef, _p_desc: &TxTParam, _value: &Entry) {
        let name = self.exec_state.param_names.pop_front().unwrap();
        match p_ref {
            ParamRef::Load(ParamMode::Consume, _) => {
                self.exec_state.consumed_elems.insert(name);
            },
            _ => {},
        };
    }

    fn return_value(&mut self, r_typ:&RetType, r_desc:&TxTReturn, value:&Entry){
        let name = self.exec_state.return_names.pop_front().unwrap();
        match r_typ {
            RetType::Store => {
                let id = unsafe {value.adt.1.get(0).unwrap().data.deref()}.try_into().unwrap();
                let pretty = pretty_print_data(value, &r_desc.desc);
                self.exec_state.produced_elems.insert(name, (id, pretty));
            },
            RetType::Put(_) => {}
            RetType::Drop => {},
            RetType::Log => {},
        };
    }

    fn transaction_finish(&mut self, _transaction: &Transaction, _success: bool) { }
    fn section_finish(&mut self, _section: &BundleSection, success: bool) {
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
                    Ok(Some(_)) => {}
                    Err(x) => Err(x).unwrap(),
                }
                self.element_data.insert(name.clone(), data).unwrap();
            }
            self.active_elems.flush().unwrap();
            self.element_data.flush().unwrap();
            self.exec_state.consumed_elems = BTreeSet::new();
            self.exec_state.produced_elems = BTreeMap::new();
        }
    }
}

const MAX_PARSE_DEPTH:usize = 1024;

impl State {


    pub fn execute_bundle(&mut self, bundle:&[u8], block_no:u64, heap:&Heap) -> Result<()> {
        let txt_bundle_alloc = heap.new_virtual_arena(CONFIG.max_bundle_size);
        let txt_bundle= ServerSystem::parse_bundle(&bundle,&txt_bundle_alloc)?;
        let ctx = Context {
            store: &self.store,
            txt_bundle: &txt_bundle
        };
        execute::<_, ServerSystem>(ctx,block_no, &heap, &mut self.tracking)
    }

    #[cfg(feature = "wasm")]
    pub fn deploy_module(&mut self, module:Vec<u8>, system_mode_on:bool, system_id:Option<u8>) -> Result<Hash> {
        let now = Instant::now();
        let (hash, gas) = if system_mode_on {
            //Todo: Configure gas read from somewhere??
            self.instance.compile_system_module(&module, &self.store, system_id.map(|id|id as usize), 100000000,10000)?
        } else {
            self.instance.compile_module(&module, &self.store, 100000000,10000)?
        };
        let end = now.elapsed().as_micros();
        println!("deployed Module with hash {:?} of size {:?} in {:?} us using {:?} gas", encode(&hash),module.len(),end, gas);
        Ok(hash)
    }

    #[cfg(feature = "wasm")]
    pub fn deploy_transaction(&mut self, transaction:Vec<u8>) -> Result<(Hash,Hash)> {
        let now = Instant::now();
        //Todo: Configure gas read from somewhere??
        let (t_hash, hash, gas) = self.instance.compile_transaction(&transaction,&self.store, 100000000,10000)?;
        let end = now.elapsed().as_micros();
        println!("deployed transaction in {:?} us using {:?} gas", end, gas);
        let desc_size = self.store.get( StorageClass::Descriptor, &t_hash,|d|d.len())?;
        println!("  - function with hash {:?} of size {:?}", encode(&hash), transaction.len());
        println!("  - descriptor with hash {:?} of size {:?}", encode(&t_hash), desc_size);
        Ok((hash,t_hash))
    }

    #[cfg(feature = "embedded")]
    pub fn execute_deploy(&mut self, bundle:&[u8], system_mode_on:bool) -> Result<Hash> {
        let heap = Heap::new(CONFIG.calc_heap_size(2),2.0);
        deploy::<_, ServerExternals>(&self.store, &bundle, &heap, system_mode_on)
    }

    #[cfg(feature = "embedded")]
    pub fn deploy_module(&mut self, module:Vec<u8>, system_mode_on:bool, _system_id:Option<u8>) -> Result<Hash> {
        let now = Instant::now();
        let hash1 = store_hash(&[&module]);
        //if !self.store.contains(StorageClass::Module, &hash1){
        let txt = DeployTransaction{
            typ: DeployType::Module,
            data: SlicePtr::wrap(&module)
        };
        let mut s = Serializer::new(usize::max_value());
        txt.serialize(&mut s)?;
        self.execute_deploy(&s.extract(), system_mode_on)?;
        let end = now.elapsed().as_micros();
        println!("deployed Module with hash {:?} of size {:?} in {:?} us", encode(&hash1),module.len(),end);
        //}
        Ok(hash1)
    }

    #[cfg(feature = "embedded")]
    pub fn deploy_transaction(&mut self, transaction:Vec<u8>) -> Result<(Hash,Hash)> {
        let now = Instant::now();
        let hash = store_hash(&[&transaction]);
        let txt = DeployTransaction{
            typ: DeployType::Transaction,
            data: SlicePtr::wrap(&transaction)
        };
        let mut s = Serializer::new(usize::max_value());
        txt.serialize(&mut s)?;
        let t_hash = self.execute_deploy(&s.extract(), false)?;
        let end = now.elapsed().as_micros();
        println!("deployed transaction in {:?} us", end);
        let desc_size = self.store.get( StorageClass::Descriptor, &t_hash,|d|d.len())?;
        println!("  - function with hash {:?} of size {:?}", encode(&hash), transaction.len());
        println!("  - descriptor with hash {:?} of size {:?}", encode(&t_hash), desc_size);
        Ok((hash,t_hash))
    }


    fn build_core<'c>(
        gas:u64,
        param_heap_limit:u16,
        descs:&[TransactionDescriptor],
        sections:SlicePtr<'c, BundleSection<'c>>,
        descriptors:SlicePtr<'c, Hash>,
        stored: SlicePtr<'c, Hash>,
        literal:SlicePtr<'c, SlicePtr<'c,u8>>,
        scratch_pad_limit:u8,
        meta:SlicePtr<'c, u8>,
        earliest_block:u64,
    ) -> TransactionBundleCore<'c> {
        let mut transaction_heap_limit = SlicePtr::<TransactionDescriptor>::SIZE as u32;
        let mut stack_elem_limit:u16 = 0;
        let mut stack_frame_limit:u16 = 0;
        let mut runtime_heap_limit:u16 = 0;
        for txt_desc in descs {
            transaction_heap_limit = transaction_heap_limit.add(txt_desc.virt_size.unwrap() as u32);
            stack_elem_limit = stack_elem_limit.max(txt_desc.max_stack);
            stack_frame_limit = stack_frame_limit.max(txt_desc.max_frames);
            runtime_heap_limit = runtime_heap_limit.max(txt_desc.max_mem);
        }

        TransactionBundleCore {
            //The correct info will be filled in later
            essential_gas_cost: gas,
            total_gas_cost: gas,

            byte_size: None,
            meta,
            earliest_block,
            param_heap_limit,
            //This is to low why -- oh I see, this is byte size we need parsed size -- can we inject in derive -- ?
            // Alla alloc Size??
            scratch_pad_limit,
            transaction_heap_limit,
            stack_elem_limit,
            stack_frame_limit,
            runtime_heap_limit,

            sections,
            descriptors,
            stored,
            literal,
        }
    }

    fn print_bundle_stats(bundle:&BaseTransactionBundle){
        println!("Transaction Bundle Stats:");
        println!("Size: {} byte", bundle.byte_size.unwrap());
        println!("Essential gas: {}", bundle.core.essential_gas_cost);
        println!("Total gas: {}", bundle.core.total_gas_cost);
        println!("Virtual runtime heap memory required: {} bytes", bundle.core.runtime_heap_limit);
        println!("Virtual transaction storage memory required: {} bytes", bundle.core.transaction_heap_limit);
        println!("Maximum stack slots: {} (required memory: {} bytes)", bundle.core.stack_elem_limit, ( bundle.core.stack_elem_limit as usize) * Entry::SIZE);
        println!("Maximum frame slots: {} (required memory: ~{} bytes)", bundle.core.stack_frame_limit, ( bundle.core.stack_frame_limit as usize) * (5*8));
    }


    pub fn bench_transaction(&mut self, txts:&[Tx]) -> Result<()> {
        let n = 5000;
        //todo make softer exits
        //todo: improve overall heap management only alloc 1 Heap
        //2*Because heap is not reset between verify and execute
        let block_no = match convert_error(self.meta_data.get("block_no"))? {
            None => 0,
            Some(val) => Parser::parse_fully(&val, 1, &NoCustomAlloc())?
        };
        let mut heap = Heap::new(2*CONFIG.calc_heap_size(2),2.0);
        let mut elapsed = 0;
        let base_exec_state = self.tracking.exec_state.clone();
        for i in 0..n {
            self.tracking.exec_state = base_exec_state.clone();
            let full_heap = heap.new_virtual_arena(100000 as usize);
            let bundle = self.build_transactions(txts, &full_heap, block_no+i)?;
            let ser = Serializer::serialize_fully(&bundle,MAX_PARSE_DEPTH)?;
            heap = heap.reuse();
            let now = Instant::now();
            self.execute_bundle( &ser,block_no+i, &heap)?;
            elapsed+=now.elapsed().as_millis();
            heap = heap.reuse();
        }
        //we flush manually as this would be done once per block and not per txt
        println!("Bundle executed in: {}ms",(elapsed as f64)/(n as f64));
        let now = Instant::now();
        self.store.flush(StorageClass::EntryValue);
        self.store.flush(StorageClass::EntryHash);
        println!("Bundle executed and flushed in: {}ms", ((elapsed as f64)/(n as f64)) + now.elapsed().as_millis() as f64);
        let new_block_no = block_no+n;
        convert_error(self.meta_data.insert("block_no", Serializer::serialize_fully(&new_block_no, 1)?))?;
        convert_error(self.meta_data.flush())?;
        Ok(())
    }

    pub fn execute_transaction(&mut self, txts:&[Tx]) -> Result<()> {
        //todo make softer exits
        //todo: improve overall heap management only alloc 1 Heap
        //2*Because heap is not reset between verify and execute
        let mut heap = Heap::new(2*CONFIG.calc_heap_size(2),2.0);
        let full_heap = heap.new_virtual_arena(100000 as usize);
        let block_no = match convert_error(self.meta_data.get("block_no"))? {
            None => 0,
            Some(val) => Parser::parse_fully(&val, 1, &NoCustomAlloc())?
        };
        let bundle = self.build_transactions(txts, &full_heap, block_no)?;
        Self::print_bundle_stats(&bundle);
        let ser = Serializer::serialize_fully(&bundle,MAX_PARSE_DEPTH)?;
        heap = heap.reuse();
        println!("Starting bundle execution");
        let now = Instant::now();
        self.execute_bundle( &ser,block_no, &heap)?;
        //we flush manually as this would be done once per block and not per txt
        println!("Bundle executed in: {}ms", now.elapsed().as_millis());
        self.store.flush(StorageClass::EntryValue);
        self.store.flush(StorageClass::EntryHash);
        println!("Bundle executed and flushed in: {}ms", now.elapsed().as_millis());
        let new_block_no = block_no+1;
        convert_error(self.meta_data.insert("block_no", Serializer::serialize_fully(&new_block_no, 1)?))?;
        convert_error(self.meta_data.flush())?;
        Ok(())
    }

    pub fn build_transactions<'c,'h>(&mut self, txts:&[Tx], full_heap:&'c VirtualHeapArena<'h>, block_no:u64) -> Result<BaseTransactionBundle<'c>>{
        let mut transactions:Vec<Transaction> = Vec::with_capacity(txts.len());

        //todo: build on Heap?
        let mut lits:Vec<SlicePtr<u8>> = Vec::new();
        let mut lit_dedup:BTreeMap<Vec<u8>,u16> = BTreeMap::new();

        let mut sigs:Vec<String> = Vec::new();
        let mut sigs_dedup:BTreeMap<String,u16> = BTreeMap::new();

        let mut stores:Vec<Hash> = Vec::new();
        let mut stores_dedup:BTreeMap<Hash,u16> = BTreeMap::new();

        let mut param_heap = 0;
        let mut ret_assigns:BTreeMap<String,u8> = BTreeMap::new();

        //Todo: Later do dedup
        let mut descs:Vec<Hash> = Vec::new();
        let mut txt_descs:Vec<TransactionDescriptor> = Vec::new();
        let mut desc_dedup:BTreeMap<Hash,u16> = BTreeMap::new();

        let mut gas = CONFIG.bundle_base_cost;
        for Tx{ref desc, ref params, ref returns} in txts {

            let txt_desc = if desc_dedup.contains_key(desc) {
                let txt_index = desc_dedup.get(desc).unwrap();
                &txt_descs[*txt_index as usize]
            } else {
                let l_desc = read_transaction_desc(desc,&self.store, full_heap)?;
                let txt_index = txt_descs.len();
                gas += l_desc.gas_cost as u64;
                txt_descs.push(l_desc);
                descs.push(*desc);
                desc_dedup.insert(*desc, txt_index as u16);
                &txt_descs[txt_index]
            };


            if txt_desc.params.len() != params.len() {
                panic!("Number of supplied parameter missmatches")
            }
            if txt_desc.returns.len() != returns.len() {
                panic!("Number of supplied returns missmatches")
            }

            let mut txt_params:Vec<ParamRef> = Vec::with_capacity(params.len());

            gas += CONFIG.entry_load_cost.compute(txt_desc.byte_size.unwrap() as u64);

            //todo: make collect
            for (p, txt_p) in params.iter().zip(txt_desc.params.iter()) {
                match p {
                    Param::Lit(data) => {
                        let max_size = txt_p.desc.max_runtime_size()?;
                        param_heap += max_size as usize;
                        if lit_dedup.contains_key(data) {
                            txt_params.push(ParamRef::Literal(*lit_dedup.get(data).unwrap()))
                        } else {
                            gas += CONFIG.parsing_cost.compute(max_size as u64);
                            lits.push(full_heap.copy_alloc_slice(&data)?);
                            lit_dedup.insert(data.clone(), (lits.len()-1) as u16);
                            txt_params.push(ParamRef::Literal((lits.len()-1) as u16))
                        }
                    },

                    Param::Pk(account) => {
                        let max_size = txt_p.desc.max_runtime_size()?;
                        param_heap += max_size as usize;
                        let data = self.get_account(account)?.public.to_bytes().to_vec();
                        if lit_dedup.contains_key(&data) {
                            txt_params.push(ParamRef::Literal(*lit_dedup.get(&data).unwrap()))
                        } else {
                            gas += CONFIG.parsing_cost.compute(max_size as u64);
                            lits.push(full_heap.copy_alloc_slice(&data)?);
                            lit_dedup.insert(data, (lits.len()-1) as u16);
                            txt_params.push(ParamRef::Literal((lits.len()-1) as u16))
                        }
                    },

                    Param::Subject(account) => {
                        let max_size = txt_p.desc.max_runtime_size()?;
                        param_heap += max_size as usize;
                        let pk = self.get_account(account)?.public.to_bytes().to_vec();
                        //compute the edDsaSubject
                        let subject = Self::calc_subject(&pk,&full_heap)?;
                        //vectorized to store in lit dedup
                        let data = subject.to_vec();
                        if lit_dedup.contains_key(&data) {
                            txt_params.push(ParamRef::Literal(*lit_dedup.get(&data).unwrap()))
                        } else {
                            gas += CONFIG.parsing_cost.compute(max_size as u64);
                            lits.push(subject);
                            lit_dedup.insert(data, (lits.len()-1) as u16);
                            txt_params.push(ParamRef::Literal((lits.len()-1) as u16))
                        }
                    },

                    Param::Sig(account) => {
                        let max_size = txt_p.desc.max_runtime_size()?;
                        param_heap += max_size as usize;
                        if sigs_dedup.contains_key(account) {
                            txt_params.push(ParamRef::Witness(*sigs_dedup.get(account).unwrap()))
                        } else {
                            gas += CONFIG.parsing_cost.compute(max_size as u64);
                            sigs.push(account.clone());
                            sigs_dedup.insert(account.clone(), (sigs.len()-1) as u16);
                            txt_params.push(ParamRef::Witness((sigs.len()-1) as u16))
                        }
                    }

                    Param::Provided => {
                        gas += ServerSystemDataManager::providable_gas(txt_p.typ)?;
                        param_heap += ServerSystemDataManager::providable_size(txt_p.typ)? as usize;
                        txt_params.push(ParamRef::Provided)
                    },

                    //todo: share common stuff
                    Param::Consume(id) => {
                        param_heap += txt_p.desc.max_runtime_size()? as usize;
                        if stores_dedup.contains_key(id) {
                            txt_params.push(ParamRef::Load(ParamMode::Consume,*stores_dedup.get(id).unwrap()))
                        } else {
                            gas += CONFIG.entry_load_cost.compute(txt_p.desc.max_serialized_size() as u64);
                            gas += CONFIG.entry_store_cost.compute(0);
                            stores.push(id.clone());
                            stores_dedup.insert(id.clone(), (stores.len()-1) as u16);
                            txt_params.push(ParamRef::Load(ParamMode::Consume,(stores.len()-1) as u16))
                        }
                    },
                    Param::Borrow(id) => {
                        param_heap += txt_p.desc.max_runtime_size()? as usize;
                        if stores_dedup.contains_key(id) {
                            txt_params.push(ParamRef::Load(ParamMode::Borrow,*stores_dedup.get(id).unwrap()))
                        } else {
                            gas += CONFIG.entry_load_cost.compute(txt_p.desc.max_serialized_size() as u64);
                            stores.push(id.clone());
                            stores_dedup.insert(id.clone(), (stores.len()-1) as u16);
                            txt_params.push(ParamRef::Load(ParamMode::Borrow,(stores.len()-1) as u16))
                        }
                    },
                    Param::Copy(id) => {
                        param_heap += txt_p.desc.max_runtime_size()? as usize;
                        if stores_dedup.contains_key(id) {
                            txt_params.push(ParamRef::Load(ParamMode::Copy,*stores_dedup.get(id).unwrap()))
                        } else {
                            gas += CONFIG.entry_load_cost.compute(txt_p.desc.max_serialized_size() as u64);
                            stores.push(id.clone());
                            stores_dedup.insert(id.clone(), (stores.len()-1) as u16);
                            txt_params.push(ParamRef::Load(ParamMode::Copy,(stores.len()-1) as u16))
                        }
                    }

                    Param::LocalConsume(key) => {
                        if !ret_assigns.contains_key(key) {
                            return error(||"Element name unknown")
                        }
                        txt_params.push(ParamRef::Fetch(ParamMode::Consume,*ret_assigns.get(key).unwrap() as u8))

                    }
                    Param::LocalBorrow(key) => {
                        if !ret_assigns.contains_key(key) {
                            return error(||"Element name unknown")
                        }
                        txt_params.push(ParamRef::Fetch(ParamMode::Borrow,*ret_assigns.get(key).unwrap() as u8))
                    }
                    Param::LocalCopy(key) => {
                        if !ret_assigns.contains_key(key) {
                            return error(||"Element name unknown")
                        }
                        txt_params.push(ParamRef::Fetch(ParamMode::Copy,*ret_assigns.get(key).unwrap() as u8))
                    }
                }
            }

            let mut txt_rets:Vec<RetType> = Vec::with_capacity(returns.len());
            for (r, txt_r) in returns.iter().zip(txt_desc.returns.iter()) {
                match r {
                    Ret::Log => {
                        //Logs may cost something in the future
                        txt_rets.push(RetType::Log)
                    },
                    Ret::Assign(name) => {
                        if !ret_assigns.contains_key(name) {
                            ret_assigns.insert(name.clone(), ret_assigns.len() as u8);
                        }
                        txt_rets.push(RetType::Put(*ret_assigns.get(name).unwrap()));
                        let runtime_size = txt_r.desc.max_runtime_size()?;
                        gas += CONFIG.copy_cost.compute(runtime_size as u64);
                        param_heap += runtime_size as usize;
                    }

                    Ret::Elem => {
                        gas += CONFIG.entry_store_cost.compute(txt_r.desc.max_serialized_size() as u64);
                        txt_rets.push(RetType::Store)
                    },
                    Ret::Drop => txt_rets.push(RetType::Drop),
                }
            }

            transactions.push(Transaction {
                txt_desc: (descs.len()-1) as u16,
                params: full_heap.copy_alloc_slice(&txt_params)?,
                returns: full_heap.copy_alloc_slice(&txt_rets)?,
            });
        }

        //Todo: Allow many Sections
        let section = BundleSection {
            typ: SectionType::Essential,
            txts: full_heap.copy_alloc_slice(&transactions)?
        };

        let meta = Serializer::serialize_fully(&block_no,1).unwrap();

        let bundle_core = Self::build_core(
            0,
            param_heap as u16,
            &txt_descs,
            full_heap.copy_alloc_slice(&[section])?,
            full_heap.copy_alloc_slice(&descs)?,
            full_heap.copy_alloc_slice(&stores)?,
            full_heap.copy_alloc_slice(&lits)?,
             ret_assigns.len() as u8,
            full_heap.copy_alloc_slice(&meta)?,
            block_no
        );

        let bundle_core_data = &Serializer::serialize_fully(&bundle_core,MAX_PARSE_DEPTH)?;  //todo: is the max here correct
        //this is not nice but the cyclic dependency requires it
        let mut core_reparsed:TransactionBundleCore = Parser::parse_fully(&bundle_core_data,MAX_PARSE_DEPTH, full_heap)?;  //todo: is the max here correct
        let witness_size = (sigs.len() * (SIGNATURE_LENGTH + 2)) + 2;
        let full_size = core_reparsed.byte_size.unwrap() + witness_size;

        gas += CONFIG.parsing_cost.compute(full_size as u64);
        core_reparsed.essential_gas_cost = gas;
        core_reparsed.total_gas_cost = gas;

        let bundle_hash = HashingDomain::Bundle.hash(&Serializer::serialize_fully(&core_reparsed,MAX_PARSE_DEPTH)?); //todo: is the max here correct

        let mut witness:Vec<SlicePtr<u8>> = Vec::with_capacity(sigs.len());

        for w in &sigs {
            let keys = self.get_account(w)?;
            let signature: Signature = keys.sign(&bundle_hash);
            witness.push(full_heap.copy_alloc_slice(&signature.to_bytes())?);
        };
        let witness_slice  = full_heap.copy_alloc_slice(&witness)?;
        Ok(BaseTransactionBundle {
            byte_size: Some(core_reparsed.byte_size.unwrap() + witness_slice.len()*(64+2) +2), //cheat here a bit so we can do stats
            core: core_reparsed,
            witness: witness_slice,
        })
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

    pub fn calc_subject<'a,'h>(pk:&[u8], full_heap:&'a VirtualHeapArena<'h>) -> Result<SlicePtr<'a,u8>>{
        //compute the edDsaSubject
        let id = raw_plain_hash(pk, &full_heap)?;
        //compute the subject Manager Subject
        raw_join_hash(&get_ed_dsa_module(), &id, HashingDomain::Derive, &full_heap)
    }

}

