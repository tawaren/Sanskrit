
use sanskrit_common::errors::*;
use sanskrit_common::encoding::{ParserAllocator, Serializer, VirtualSize};
use model::{Transaction, ParamRef, RetType, TransactionBundle, ParamMode, SectionType};
use sanskrit_common::model::{Hash, Ptr};
use sanskrit_common::arena::*;
use sanskrit_interpreter::model::{Entry, TransactionDescriptor, TxTReturn, RuntimeType, TxTParam};

use alloc::collections::BTreeSet;
use system::System;
use core::cell::{RefCell, Cell};
use alloc::vec::Vec;
use sanskrit_common::hashing::HashingDomain;
use ::CONFIG;


//A struct holding context information of the current transaction
pub struct VerificationEnvironment<'a> {
    max_desc_alloc:VirtualHeapArena<'a>,
    //Caches to parse each param just once
    entry_types:RefCell<Vec<Option<Hash>>>,
    literal_types:RefCell<Vec<Option<Hash>>>,
    witness_types:RefCell<Vec<Option<Hash>>>,
    param_heap:Cell<u32>,
}

pub trait TransactionAccountingContext {
    //get the bundle
    fn get_transaction_bundle(&self) -> &TransactionBundle;
    //reads the desc and accounts for its potential gas usage
    fn read_transaction_desc<'b, A:ParserAllocator>(&self, target:&Hash, heap:&'b A) -> Result<TransactionDescriptor<'b>>;
    //accounts for the loading of an entry
    fn account_for_entry_load(&self, param:TxTParam, first_access:bool);
    //accounts for the deletion of an entry
    fn account_for_entry_delete(&self, param:TxTParam, first_access:bool);
    //accounts for the insertion of an entry
    fn account_for_entry_store(&self, ret:TxTReturn);
    //informs the Accounter of a Section end
    // which responds with the sections gas cost
    fn store_access_gas(&self) -> u64;
}


//TODO: Make cmplete rework of the fast stuff and see if with the witnesses we can get it in order
//      In storage / provider mode do not execute payment txts (for now)
//

//Todo: We need evaluation Mode
//      Takes Bundle but with limits & ev: witnesses set to 0/null
//      Then it evaluates them and returns the filled bundle data


pub fn verify_repeated(txt_bundle:&TransactionBundle,  block_no:u64) -> Result<()> {
    //check that it is in window
    if block_no < txt_bundle.core.earliest_block || block_no >= txt_bundle.core.earliest_block + CONFIG.block_inclusion_window {
        return error(||"Transaction not allowed in current block")
    }

    /*
    //Todo: This is only needed in Stateless modes
    //check that the witnesses are still ok (may have changed)

    if txt_bundle.byte_size.unwrap() - txt_bundle.core.byte_size.unwrap() - 4 /*the -4 is the 2 sizes*/ > txt_bundle.core.witness_bytes_limit as usize {
        return error(||"Transaction Bundle witness size exceeds wittness limit")
    }

    if txt_bundle.store_witness.len() != txt_bundle.core.stored.len() {
        return error(||"Not enough witnesses for stored values")
    }
    */


    Ok(())
}

//Executes a transaction
pub fn verify_once<V: TransactionAccountingContext, SYS:System>(acc_ctx:&V, sys:&SYS, heap:&Heap) -> Result<()> {
    //Calculate the payment information
    //Starts with the parsing costs which are already done mostly but this is inevitable (a miner could have a size limit)
    //This includes encoding the parameter witnesses


    let max_desc_alloc = heap.new_virtual_arena(acc_ctx.get_transaction_bundle().core.transaction_storage_heap as usize);

    let entry_types = RefCell::new(alloc::vec::from_elem(Option::None, acc_ctx.get_transaction_bundle().core.stored.len()));
    let literal_types = RefCell::new(alloc::vec::from_elem(Option::None, acc_ctx.get_transaction_bundle().core.literal.len()));
    let witness_types  = RefCell::new(alloc::vec::from_elem(Option::None,acc_ctx.get_transaction_bundle().witness.len()));

    let mut verify_env = VerificationEnvironment {
        max_desc_alloc,
        //Input type checks
        entry_types,
        literal_types,
        witness_types,
        param_heap: Cell::new(0)
    };

    let mut required_gas = CONFIG.parsing_cost.compute(acc_ctx.get_transaction_bundle().byte_size.unwrap() as u64);
    let mut essential_gas = required_gas;

    let mut is_in_essential = true;

    for txt_section in acc_ctx.get_transaction_bundle().core.sections.iter() {
        for txt in txt_section.txts.iter() {
            required_gas += verify_transaction(&verify_env, acc_ctx, sys, txt, txt_section.typ)? as u64;
            //release all the memory so it does not leak into the next transaction
            verify_env.max_desc_alloc = verify_env.max_desc_alloc.reuse();
        }

        required_gas += acc_ctx.store_access_gas();

        if is_in_essential {
            if txt_section.typ == SectionType::Essential {
                essential_gas = required_gas;
            }
            is_in_essential = false;
        } else{
            if txt_section.typ == SectionType::Essential {
                return error(||"Essential Section must be at the beginning of a bundle and only one is allowed")
            }
        }
    }

    //Todo: Check that this is in the limit specified by the miner (or globally agreed on)
    //      If miner the check needs to be done specially in order that sender does not get removed from network layer
    if essential_gas > acc_ctx.get_transaction_bundle().core.essential_gas_cost {
        return error(||"Bundle has declared wrong essential gas cost")
    }

    if required_gas > acc_ctx.get_transaction_bundle().core.total_gas_cost {
        return error(||"Bundle has declared wrong total gas cost")
    }

    if verify_env.param_heap.get() > acc_ctx.get_transaction_bundle().core.param_heap_limit as u32 {
        return error(||"Bundle has not reserved enough parameter heap space")
    }

    Ok(())
}



fn verify_transaction<V: TransactionAccountingContext, SYS:System>(env:&VerificationEnvironment, acc_ctx:&V, sys:&SYS, txt:&Transaction, sec_typ:SectionType) -> Result<u64>{
    //Prepare all the Memory
    if acc_ctx.get_transaction_bundle().core.descriptors.len() <= txt.txt_desc as usize { return error(||"Descriptor index out of range")  }
    let target = &acc_ctx.get_transaction_bundle().core.descriptors[txt.txt_desc as usize];

    let txt_desc = acc_ctx.read_transaction_desc(target, &env.max_desc_alloc)?;

    if txt_desc.max_frames > acc_ctx.get_transaction_bundle().core.stack_frame_limit {return error(||"Bundle has not reserved enough frame space")}
    if txt_desc.max_stack > acc_ctx.get_transaction_bundle().core.stack_elem_limit {return error(||"Bundle has not reserved enough stack space")}
    if txt_desc.max_mem > acc_ctx.get_transaction_bundle().core.runtime_heap_limit {return error(||"Bundle has not reserved enough heap space")}

    //push everything required onto the stack
    let mut lock_set = BTreeSet::new();

    let mut gas = txt_desc.gas_cost as u64;

    for (p,p_typ) in txt_desc.params.iter().zip(txt.params.iter()) {
        match p_typ {
            ParamRef::Load(ParamMode::Consume,index) => {
                let first_access = check_store_type(env, *index, *p)?;
                //This will account for gas, depending on how the used storage works
                acc_ctx.account_for_entry_load(*p, first_access);
                acc_ctx.account_for_entry_delete(*p, first_access);
                if !p.consumes && !p.drop { return error(||"A owned store value must be consumed or dropped") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if !sys.is_entry(p.typ) { return error(|| "Value parameter must be an entry") }
                if acc_ctx.get_transaction_bundle().core.stored.len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = &acc_ctx.get_transaction_bundle().core.stored[*index as usize];
                if lock_set.contains(hash) { return error(||"An entry can only be fetched once"); }
                lock_set.insert(hash.clone());
                check_store_type(env, *index, *p)?;
            }
            ParamRef::Load(ParamMode::Copy, index) => {
                let first_access = check_store_type(env, *index, *p)?;
                //This will account for gas, depending on how the used storage works
                acc_ctx.account_for_entry_load(*p, first_access);
                if !p.copy { return error(||"A Copied store value must allow copy") }
                if !p.consumes && !p.drop { return error(||"A Copied store value must be consumed or dropped") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if !sys.is_entry(p.typ) { return error(|| "Value parameter must be an entry") }
                if acc_ctx.get_transaction_bundle().core.stored.len() <= *index as usize { return error(||"Value index out of range")  }
            },
            ParamRef::Load(ParamMode::Borrow, index) => {
                let first_access = check_store_type(env, *index, *p)?;
                //This will account for gas, depending on how the used storage works
                acc_ctx.account_for_entry_load(*p, first_access);
                if p.consumes { return error(||"A Borrowed store value can not be consumed") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if !sys.is_entry(p.typ) { return error(|| "Value parameter must be an entry") }
                if acc_ctx.get_transaction_bundle().core.stored.len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = &acc_ctx.get_transaction_bundle().core.stored[*index as usize];
                if lock_set.contains(hash) { return error(||"An entry can only be fetched once"); }
                lock_set.insert(hash.clone());
                check_store_type(env, *index, *p)?;
            },

            ParamRef::Provided => {
                //Todo: can we reuse -- only alloc once in the beginning?;
                //Todo: where do we account for gas?
                env.param_heap.set(env.param_heap.get() + ctx_size());
                if !sys.is_context(p.typ) { return error(||"Provided value parameter must be of a supported type") }
            },
            ParamRef::Literal(index) => {
                if !p.primitive { return error(||"Literals must be of primitive type") }
                if acc_ctx.get_transaction_bundle().core.literal.len() <= *index as usize { return error(||"Value index out of range")  }
                if check_literal_type(env, *index, *p)? {
                    //Todo: Shalle use real size instead? We do now
                    gas += CONFIG.parsing_cost.compute(p.desc.max_runtime_size()? as u64)
                }
            },
            ParamRef::Witness(index) => {
                if sec_typ != SectionType::Essential { return error(||"Witnesses can only be used in essential sections") }
                if !p.primitive { return error(||"Witnesses must be of primitive type") }
                if acc_ctx.get_transaction_bundle().witness.len() <= *index as usize { return error(||"Value index out of range")  }
                if check_witness_type(env, *index, *p)? {
                    //Todo: Shalle use real size instead? We do now
                    gas += CONFIG.parsing_cost.compute(p.desc.max_runtime_size()? as u64)
                }
            },
        };
    }

    for (r, r_typ) in txt_desc.returns.iter().zip(txt.returns.iter()) {
        match *r {
            TxTReturn{ primitive, drop, typ,  .. } => {
                match r_typ {
                    RetType::Store => {
                        //This will account for gas, depending on how the used storage works
                        acc_ctx.account_for_entry_store(*r);
                        if primitive {return error(||"Can not store primitives") }
                        if !sys.is_entry(typ) { return error(||"Stored return must be an entry") }
                    },
                    RetType::Drop => {
                        if !drop { return error(||"Returns without drop capability must be stored") }
                    },
                    RetType::Log => {
                        //Todo: Logs can be costly we should charge
                        if !drop { return error(||"Returns without drop capability must be stored") }
                    },
                }
            },
        }
    }
    Ok(gas)
}


//Helper to calc the key for a storage slot
fn type_hash(typ:Ptr<RuntimeType>) -> Result<Hash> {

    let control_type = Serializer::serialize_fully(&typ,CONFIG.max_structural_dept)?;

    //Todo: This wastes CPU Cycles
    //      We could instead store control_type
    //       But that would use unpredicatble memory
    //       Luckely this is implementation detail here
    //Make a 20 byte digest hascher
    let mut context = HashingDomain::Entry.get_domain_hasher();
    //push the data into it
    context.update(&control_type);
    //calc the Hash
    Ok(context.finalize())
}

fn check_literal_type(env:&VerificationEnvironment, index:u16, param:TxTParam) -> Result<bool> {
    let entry_copy = env.literal_types.borrow()[index as usize];
    let control_hash = type_hash(param.typ)?;
    match entry_copy {
        None => {
            env.param_heap.set(env.param_heap.get() + param.desc.max_runtime_size()? as u32);
            env.literal_types.borrow_mut()[index as usize] = Some(control_hash);
            Ok(true)
        },
        Some(expected_hash) => {
            if control_hash != expected_hash { return error(|| "A single literal value is referred to over different types") }
            Ok(false)
        }
    }
}

fn check_witness_type(env:&VerificationEnvironment, index:u16, param:TxTParam) -> Result<bool> {
    let entry_copy = env.witness_types.borrow()[index as usize];
    let control_hash = type_hash(param.typ)?;
    match entry_copy {
        None => {
            env.param_heap.set(env.param_heap.get() + param.desc.max_runtime_size()? as u32);
            env.witness_types.borrow_mut()[index as usize] = Some(control_hash);
            Ok(true)
        },
        Some(expected_hash) => {
            if control_hash != expected_hash { return error(|| "A single witness value is referred to over different types") }
            Ok(false)
        }
    }
}


fn check_store_type(env:&VerificationEnvironment,index:u16, param:TxTParam) -> Result<bool> {
    let entry_copy = env.entry_types.borrow()[index as usize];
    let control_hash = type_hash(param.typ)?;
    match entry_copy {
        None => {
            env.param_heap.set(env.param_heap.get() + param.desc.max_runtime_size()? as u32);
            env.entry_types.borrow_mut()[index as usize] = Some(control_hash);
            Ok(true)
        },
        Some(expected_hash) => {
            if control_hash != expected_hash { return error(||"A single store value is referred to over different types")}
            Ok(false)
        }
    }
}

pub fn ctx_size<'a,'h>() -> u32 {
    //Todo: construct over schema to be compiler agnostic
    return (2*Hash::SIZE + 4*Entry::SIZE) as u32;

}