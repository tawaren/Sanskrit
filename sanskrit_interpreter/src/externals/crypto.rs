use externals::ExecutionInterface;
use sanskrit_common::hashing::{Hasher, HashingDomain};
use model::{Entry, Kind, Adt};
use ed25519_dalek::{PublicKey, Verifier};
use ed25519_dalek::ed25519::signature::Signature;
use sanskrit_common::model::{ValueRef, SlicePtr};
use sanskrit_common::errors::*;
use sanskrit_common::arena::VirtualHeapArena;


pub fn raw_plain_hash<'a,'h>(data:&[u8], alloc_heap:&'a VirtualHeapArena<'h>) -> Result<SlicePtr<'a,u8>> {
    //Plain is an exception and allowed to be called with no domain as it is always used as data
    let mut context = Hasher::new();
    //fill the hash
    context.update(data);
    //calc the Hash
    context.alloc_finalize(alloc_heap)
}

//a non recursive, non-structural variant that just hashes the data input -- it has no collision guarantees
// This is never used to generate collision free hashes like unique, singleton, indexes etc...
pub fn plain_hash<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(inter:&mut I, kind:Kind, ValueRef(val):ValueRef, tail:bool) -> Result<()> {
    let op1 = inter.get(val as usize)?;
    let alloc_heap = inter.get_heap();
    let hash_data =  I::process_entry_slice(kind,op1, |data| raw_plain_hash(data, alloc_heap))?;
    //get ownership and return
    inter.get_stack(tail).push(Entry{data:hash_data})?;
    Ok(())
}

pub fn raw_join_hash<'a,'h>(data1:&[u8], data2:&[u8], domain:HashingDomain, alloc_heap:&'a VirtualHeapArena<'h>)  -> Result<SlicePtr<'a,u8>>{
    let mut context = domain.get_domain_hasher();
    //fill the hash with first value
    context.update(data1);
    //fill the hash with second value
    context.update(data2);
    //calc the Hash
    context.alloc_finalize(alloc_heap)
}

//hashes 2 inputs together
pub fn join_hash<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(inter:&mut I, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, domain:HashingDomain, tail:bool) -> Result<()>  {
    let data1 = unsafe {inter.get(val1 as usize)?.data};
    let data2 = unsafe {inter.get(val2 as usize)?.data};
    //calc the Hash
    let hash_data = raw_join_hash(&data1, &data2, domain, &inter.get_heap())?;
    //get ownership and return
    inter.get_stack(tail).push(Entry{data:hash_data})?;
    Ok(())
}

pub fn ecdsa_verify<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(inter:&mut I, ValueRef(msg):ValueRef, ValueRef(pk):ValueRef, ValueRef(sig):ValueRef, tail:bool) -> Result<()>  {
    let msg_data = unsafe {inter.get(msg as usize)?.data};
    let pk_data = unsafe {inter.get(pk as usize)?.data};
    let sig_data = unsafe {inter.get(sig as usize)?.data};

    let res = match (PublicKey::from_bytes(&pk_data), Signature::from_bytes(&sig_data)) {
        (Ok(pk), Ok(sig)) => {
            match pk.verify(&msg_data, &sig) {
                Ok(_) => 1,
                Err(_) => 0
            }
        },
        _ => 0
    };
    //this is false, true would be 1
    inter.get_stack(tail).push(Entry{ adt: Adt(res, SlicePtr::empty())})?;
    Ok(())
}