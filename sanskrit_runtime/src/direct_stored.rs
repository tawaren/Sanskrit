use ::CONFIG;
use sanskrit_common::store::{Store, StorageClass};
use core::cell::{Cell, RefCell};
use sanskrit_common::encoding::{ParserAllocator, Serializer, Parser};
use sanskrit_interpreter::model::{TransactionDescriptor, TxTParam, TxTReturn, Entry, RuntimeType};
use verify::TransactionVerificationContext;
use sanskrit_common::errors::*;
use compute::TransactionExecutionContext;
use sanskrit_common::hashing::{Hasher, HashingDomain};
use core::ops::Deref;
use core::convert::TryInto;
use sanskrit_common::model::{Hash, hash_from_slice, Ptr};
use sanskrit_common::arena::VirtualHeapArena;
use core::marker::PhantomData;
use ::{Context, TransactionBundle};
use alloc::vec::Vec;
use alloc::collections::BTreeSet;

pub enum UniquenessScope{
    Bundle,
    Transaction,
    None
}

pub trait SystemDataManager<B:TransactionBundle> {
    fn providable_size(typ:Ptr<RuntimeType>) -> Result<u32>;
    fn providable_gas(typ:Ptr<RuntimeType>) -> Result<u64>;
    fn is_chain_value(typ:Ptr<RuntimeType>) -> bool;
    fn provided_value_key(typ:Ptr<RuntimeType>, section_no:u8,  txt_no:u8, p_num:u8) -> Option<Vec<u8>>;
    fn create_provided_value<'a,'h>(bundle:&B, typ:Ptr<RuntimeType>, alloc: &'a VirtualHeapArena<'h>, block_no: u64, section_no:u8,  txt_no:u8, p_num:u8) -> Result<Entry<'a>>;
}

pub struct StatefulEntryStoreVerifier<B:TransactionBundle, SDM: SystemDataManager<B>>{
    section_gas:Cell<u64>,
    used_keys:RefCell<BTreeSet<Vec<u8>>>,
    _phantom_sdm:PhantomData<SDM>,
    _phantom_b:PhantomData<B>,

}

pub fn read_transaction_desc<'d, S:Store, A:ParserAllocator>(target:&Hash, store:&S, heap: &'d A) -> Result<TransactionDescriptor<'d>> {
    store.parsed_get(StorageClass::Descriptor, target, CONFIG.max_structural_dept, heap)
}


impl<S:Store,B:TransactionBundle, SDM: SystemDataManager<B>> TransactionVerificationContext<S,B> for StatefulEntryStoreVerifier<B,SDM>  {

    fn new() -> Self {
        StatefulEntryStoreVerifier {
            section_gas: Cell::new(0),
            used_keys: RefCell::new(BTreeSet::new()),
            _phantom_sdm: Default::default(),
            _phantom_b: Default::default()
        }
    }

    fn read_transaction_desc<'d, A: ParserAllocator>(&self, ctx:&Context<S,B>, target: &Hash, heap: &'d A) -> Result<TransactionDescriptor<'d>> {
        let txt_desc:TransactionDescriptor = ctx.store.parsed_get(StorageClass::Descriptor, target, CONFIG.max_structural_dept, heap)?;
        self.section_gas.set(self.section_gas.get() + CONFIG.entry_load_cost.compute(txt_desc.byte_size.unwrap() as u64));
        Ok(txt_desc)
    }

    fn account_for_chain_value_load(&self, _ctx:&Context<S,B>, param: TxTParam, first_access:bool){
        if first_access {
            self.section_gas.set(self.section_gas.get() + CONFIG.entry_load_cost.compute(param.desc.max_serialized_size() as u64));
        }
    }

    fn account_for_chain_value_delete(&self, _ctx:&Context<S,B>, _param: TxTParam, _first_access: bool) {
        self.section_gas.set(self.section_gas.get() + CONFIG.entry_store_cost.compute(0));
    }

    fn account_for_chain_value_store(&self, _ctx:&Context<S,B>, ret: TxTReturn) {
        self.section_gas.set(self.section_gas.get() + CONFIG.entry_store_cost.compute(ret.desc.max_serialized_size() as u64));
    }

    fn store_access_gas(&self, _ctx:&Context<S,B>) -> u64{
        let res = self.section_gas.get();
        self.section_gas.set(0);
        res
    }

    fn is_chain_value(&self, _ctx:&Context<S,B>, typ:Ptr<RuntimeType>) -> bool {
        SDM::is_chain_value(typ)
    }

    fn verify_providable(&self, _ctx:&Context<S,B>, typ:Ptr<RuntimeType>, section_no:u8,  txt_no:u8, p_num:u8) -> Result<(u64,u32)> {
        match SDM::provided_value_key(typ, section_no, txt_no, p_num) {
            Some(key) => {
                if self.used_keys.borrow().contains(&key) {
                    return error(||"provided value already used")
                }
                self.used_keys.borrow_mut().insert(key);
            },
            None => {}
        }
        let size = SDM::providable_size(typ)?;
        let gas = SDM::providable_gas(typ)?;
        Ok((gas,size))
    }
}


pub struct StatefulEntryStoreExecutor<B:TransactionBundle, SDM: SystemDataManager<B>> {
    _phantom_sdm:PhantomData<SDM>,
    _phantom_b:PhantomData<B>
}
//Helper to calc the key for a storage slot
fn entry_hash(typ:&[u8], data_hash:&Hash) -> Hash {
    //Make a 20 byte digest hascher
    let mut context = HashingDomain::Entry.get_domain_hasher();
    //push the data into it
    context.update(&typ);
    context.update(data_hash);

    //calc the Hash
    context.finalize()
}


impl<S:Store,B:TransactionBundle, SDM: SystemDataManager<B>> TransactionExecutionContext<S,B> for StatefulEntryStoreExecutor<B,SDM> {
    fn new() -> Self {
        StatefulEntryStoreExecutor{
            _phantom_sdm: Default::default(),
            _phantom_b: Default::default()        }
    }

    fn read_transaction_desc<'d, A: ParserAllocator>(&self, ctx:&Context<S,B>, target: &[u8; 20], heap: &'d A) -> Result<TransactionDescriptor<'d>> {
        ctx.store.parsed_get(StorageClass::Descriptor, target, CONFIG.max_structural_dept, heap)
    }

    fn create_provided_value<'a,'h>(&self, ctx: &Context<S,B>, typ: Ptr<RuntimeType>, alloc: &'a VirtualHeapArena<'h>, block_no: u64, section_no:u8,  txt_no:u8, p_num:u8) -> Result<Entry<'a>> {
        SDM::create_provided_value(ctx.txt_bundle, typ, alloc, block_no, section_no, txt_no, p_num)
    }


    fn chain_value_load<'d>(&self, ctx:&Context<S,B>, index: u16, param: TxTParam, parameter_heap:&'d VirtualHeapArena) -> Result<Entry<'d>> {
        let key_hash = &ctx.txt_bundle.stored()[index as usize];
        let data = ctx.store.get(StorageClass::EntryValue, key_hash, |d|d.to_vec())?;
        let mut data_hash = Hasher::new();
        data_hash.update(&data);
        let value_hash = data_hash.finalize();

        let control_type = Serializer::serialize_fully(&param.typ,CONFIG.max_structural_dept)?;
        let expected_hash = entry_hash(&control_type,&value_hash);

        let control_hash = ctx.store.get(StorageClass::EntryHash, key_hash,  |d|hash_from_slice(d))?;
        if control_hash != expected_hash { return error(||"stored value had wrong type")}

        let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
        param.desc.parse_value(&mut parser, parameter_heap)
    }

    fn chain_value_delete(&self, ctx:&Context<S,B>, index: u16) -> Result<()> {
        let del = &ctx.txt_bundle.stored()[index as usize];
        ctx.store.delete(StorageClass::EntryValue, del)?;
        ctx.store.delete(StorageClass::EntryHash, del)
    }

    fn chain_value_store(&self, ctx:&Context<S,B>, ret_entry: &Entry, ret: TxTReturn) -> Result<()> {

        let mut s = Serializer::new(CONFIG.max_structural_dept);
        ret.desc.serialize_value(*ret_entry, &mut s)?;
        let data = s.extract();
        let mut data_hash = Hasher::new();
        data_hash.update(&data);

        let value_hash = data_hash.finalize();
        let control_type = Serializer::serialize_fully(&ret.typ,CONFIG.max_structural_dept)?;
        let expected_hash = entry_hash(&control_type, &value_hash);

        let id = unsafe {ret_entry.adt.1.get(0).expect("entry has to few fields").data.deref()}.try_into().expect("entry id has incorrect length");
        ctx.store.set(StorageClass::EntryHash, id, expected_hash.to_vec())?;
        ctx.store.set(StorageClass::EntryValue, id,  data)?;
        Ok(())
    }

    fn commit(&self, ctx:&Context<S,B>)  {
        ctx.store.commit(StorageClass::EntryValue);
        ctx.store.commit(StorageClass::EntryHash);
    }

    fn revert(&self, ctx:&Context<S,B>) {
        ctx.store.rollback(StorageClass::EntryValue);
        ctx.store.rollback(StorageClass::EntryHash);
    }
}