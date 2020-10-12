use ::CONFIG;
use sanskrit_common::store::{Store, StorageClass};
use core::cell::Cell;
use sanskrit_common::encoding::{ParserAllocator, Serializer, Parser};
use sanskrit_interpreter::model::{TransactionDescriptor, TxTParam, TxTReturn, Entry};
use verify::TransactionAccountingContext;
use sanskrit_common::errors::*;
use model::TransactionBundle;
use compute::ExecutableStore;
use sanskrit_common::hashing::{Hasher, HashingDomain};
use core::ops::Deref;
use core::convert::TryInto;
use sanskrit_common::model::{Hash, hash_from_slice};
use sanskrit_common::arena::VirtualHeapArena;

pub struct StatefulEntryStoreAccounter<'a,'b,'c,S:Store>{
    store:&'a S,
    section_gas:Cell<u64>,
    txt_bundle:&'b TransactionBundle<'c>,
}

impl<'a,'b,'c,S:Store> StatefulEntryStoreAccounter<'a,'b,'c,S> {
    pub fn new(store: &'a S, txt_bundle:&'b TransactionBundle<'c>) -> Self {
        StatefulEntryStoreAccounter {
            store,
            section_gas: Cell::new(0),
            txt_bundle
        }
    }
}

pub fn read_transaction_desc<'d, S:Store, A:ParserAllocator>(target:&Hash, store:&S, heap: &'d A) -> Result<TransactionDescriptor<'d>> {
    store.parsed_get(StorageClass::Descriptor, target, CONFIG.max_structural_dept, heap)
}


impl<'a,'b,'c, S:Store> TransactionAccountingContext for StatefulEntryStoreAccounter<'a,'b,'c, S>  {
    fn get_transaction_bundle(&self) -> &TransactionBundle {
        self.txt_bundle
    }

    fn read_transaction_desc<'d, A: ParserAllocator>(&self, target: &Hash, heap: &'d A) -> Result<TransactionDescriptor<'d>> {
        let txt_desc:TransactionDescriptor = self.store.parsed_get(StorageClass::Descriptor, target, CONFIG.max_structural_dept, heap)?;
        self.section_gas.set(self.section_gas.get() + CONFIG.parsing_cost.compute(txt_desc.byte_size.unwrap() as u64));
        Ok(txt_desc)
    }

    fn account_for_entry_load(&self, param: TxTParam, first_access:bool){
        if first_access {
            self.section_gas.set(self.section_gas.get() + CONFIG.entry_load_cost.compute(param.desc.max_serialized_size() as u64));
        }
    }

    fn account_for_entry_delete(&self, _param: TxTParam, _first_access: bool) {
        self.section_gas.set(self.section_gas.get() + CONFIG.entry_store_cost.compute(0));
    }

    fn account_for_entry_store(&self, ret: TxTReturn) {
        self.section_gas.set(self.section_gas.get() + CONFIG.entry_store_cost.compute(ret.desc.max_serialized_size() as u64));
    }

    fn store_access_gas(&self) -> u64{
        let res = self.section_gas.get();
        self.section_gas.set(0);
        res
    }
}


pub struct StatefulEntryStoreExecutor<'a,'b,'c,S:Store>{
    store:&'a S,
    txt_bundle:&'b TransactionBundle<'c>,
    bundle_hash:Hash,
    full_bundle_hash:Hash
}

impl<'a,'b,'c, S:Store> StatefulEntryStoreExecutor<'a,'b, 'c, S> {
    pub fn new(store: &'a S, txt_bundle:&'b TransactionBundle<'c>, bundle_hash:Hash, full_bundle_hash:Hash) -> Self {
        StatefulEntryStoreExecutor {
            store,
            txt_bundle,
            bundle_hash,
            full_bundle_hash
        }
    }
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


impl<'a,'b, 'c, S:Store> ExecutableStore for StatefulEntryStoreExecutor<'a,'b, 'c, S>  {
    fn get_bundle_hash(&self) -> Hash {
        self.bundle_hash
    }

    fn get_full_bundle_hash(&self) -> Hash {
        self.full_bundle_hash
    }

    fn get_transaction_bundle(&self) -> &TransactionBundle {
        self.txt_bundle
    }

    fn read_transaction_desc<'d, A: ParserAllocator>(&self, target: &[u8; 20], heap: &'d A) -> Result<TransactionDescriptor<'d>> {
        self.store.parsed_get(StorageClass::Descriptor, target, CONFIG.max_structural_dept, heap)
    }

    fn entry_load<'d>(&self, index: u16, param: TxTParam, parameter_heap:&'d VirtualHeapArena)-> Result<Entry<'d>> {
        let key_hash = &self.txt_bundle.core.stored[index as usize];
        let data = self.store.get(StorageClass::EntryValue, key_hash, |d|d.to_vec())?;
        let mut data_hash = Hasher::new();
        data_hash.update(&data);
        let value_hash = data_hash.finalize();

        let control_type = Serializer::serialize_fully(&param.typ,CONFIG.max_structural_dept)?;
        let expected_hash = entry_hash(&control_type,&value_hash);

        let control_hash = self.store.get(StorageClass::EntryHash, key_hash,  |d|hash_from_slice(d))?;
        if control_hash != expected_hash { return error(||"stored value had wrong type")}

        let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
        param.desc.parse_value(&mut parser, parameter_heap)
    }

    fn entry_delete(&self, index: u16) -> Result<()> {
        let del = &self.txt_bundle.core.stored[index as usize];
        self.store.delete(StorageClass::EntryValue, del)?;
        self.store.delete(StorageClass::EntryHash, del)
    }

    fn entry_store(&self, ret_entry: &Entry, ret: TxTReturn)  -> Result<()> {

        let mut s = Serializer::new(CONFIG.max_structural_dept);
        ret.desc.serialize_value(*ret_entry, &mut s)?;
        let data = s.extract();
        let mut data_hash = Hasher::new();
        data_hash.update(&data);

        let value_hash = data_hash.finalize();
        let control_type = Serializer::serialize_fully(&ret.typ,CONFIG.max_structural_dept)?;
        let expected_hash = entry_hash(&control_type, &value_hash);

        let id = unsafe {ret_entry.adt.1.get(0).expect("entry has to few fields").data.deref()}.try_into().expect("entry id has incorrect length");
        self.store.set(StorageClass::EntryHash, id, expected_hash.to_vec())?;
        self.store.set(StorageClass::EntryValue, id,  data)?;
        Ok(())
    }

    fn commit(&self)  {
        self.store.commit(StorageClass::EntryValue);
        self.store.commit(StorageClass::EntryHash);
    }

    fn revert(&self) {
        self.store.rollback(StorageClass::EntryValue);
        self.store.rollback(StorageClass::EntryHash);
    }
}