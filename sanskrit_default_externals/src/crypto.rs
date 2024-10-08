use alloc::format;
use sanskrit_common::hashing::{Hasher, HashingDomain};
use ed25519_consensus::*;
use sanskrit_common::model::{Hash, ValueRef};
use sanskrit_common::errors::*;
use sanskrit_chain_code::{ExecutionInterface, Value};
use sanskrit_chain_code::model::Kind;


pub fn raw_plain_hash(data:&[u8]) -> Hash {
    //Plain is an exception and allowed to be called with no domain as it is always used as data
    let mut context = Hasher::new();
    //fill the hash
    context.update(data);
    //calc the Hash
    context.finalize()
}

//a non recursive, non-structural variant that just hashes the data input -- it has no collision guarantees
// This is never used to generate collision free hashes like unique, singleton, indexes etc...
pub fn plain_hash<I:ExecutionInterface>(inter:&mut I, kind:Kind, val:ValueRef, tail:bool) -> Result<()> {
    let op1 = inter.get(val)?;
    let hash_data =  I::process_entry_slice(kind,op1, |data| raw_plain_hash(data));
    //get ownership and return
    //todo: i do not like the to_vec
    inter.push_entry(tail,inter.data_entry(hash_data.to_vec()));
    Ok(())
}

pub fn raw_join_hash(data1:&[u8], data2:&[u8], domain:HashingDomain)  -> Hash{
    let mut context = domain.get_domain_hasher();
    //fill the hash with first value
    context.update(data1);
    //fill the hash with second value
    context.update(data2);
    //calc the Hash
    context.finalize()
}

//hashes 2 inputs together
pub fn join_hash<I:ExecutionInterface>(inter:&mut I, val1:ValueRef, val2:ValueRef, domain:HashingDomain, tail:bool) -> Result<()>  {
    let data1 = inter.get(val1)?;
    let data2 = inter.get(val2)?;
    //calc the Hash
    let hash_data = raw_join_hash(data1.as_data(), data2.as_data(), domain);
    //get ownership and return
    //todo: i do not like the to_vec
    inter.push_entry(tail,inter.data_entry(hash_data.to_vec()));
    Ok(())
}

pub fn ecdsa_verify<I:ExecutionInterface>(inter:&mut I, msg:ValueRef, pk:ValueRef, sig:ValueRef, tail:bool) -> Result<()>  {
    let msg_data = inter.get(msg)?;
    let pk_data = inter.get(pk)?;
    let sig_data = inter.get(sig)?;

    if pk_data.as_data().len() != 32 {
        return owned_error(||format!("Wrong Key Size: {} vs. {}",pk_data.as_data().len(), 32));
    }

    if sig_data.as_data().len() != 64 {
        return owned_error(||format!("Wrong Signature Size: {} vs. {}", sig_data.as_data().len(), 64));
    }

    let res = match (VerificationKey::try_from(pk_data.as_data()), Signature::try_from(sig_data.as_data())) {
        (Ok(vk), Ok(sig)) => {
            match vk.verify(&sig,msg_data.as_data()) {
                Ok(_) => true,
                Err(_) => false
            }
        },
        _ => false
    };
    inter.push_entry(tail,inter.bool_entry(res));
    Ok(())
}
