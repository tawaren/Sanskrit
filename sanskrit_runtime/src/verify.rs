
use sanskrit_common::errors::*;
use sanskrit_common::encoding::ParserAllocator;
use model::{Transaction, ParamRef, RetType, ParamMode, SectionType};
use sanskrit_common::model::{Hash, Ptr, SlicePtr};
use sanskrit_common::arena::*;
use sanskrit_interpreter::model::{TransactionDescriptor, TxTReturn, RuntimeType, TxTParam};

use alloc::collections::BTreeSet;
use core::cell::{RefCell, Cell};
use alloc::vec::Vec;
use ::CONFIG;
use sanskrit_common::store::Store;
use system::SystemContext;
use ::{Context, TransactionBundle};


//A struct holding context information of the current transaction
pub struct VerificationEnvironment<'a> {
    //Caches to parse each param just once
    descs:SlicePtr<'a,TransactionDescriptor<'a>>,
    entry_types:RefCell<Vec<Option<Ptr<'a, RuntimeType<'a>>>>>,
    literal_types:RefCell<Vec<Option<Ptr<'a, RuntimeType<'a>>>>>,
    witness_types:RefCell<Vec<Option<Ptr<'a, RuntimeType<'a>>>>>,
    scratch_pad_types:RefCell<Vec<Option<Ptr<'a, RuntimeType<'a>>>>>,
    //indicates the number of scratch pad entries that lack drop
    num_non_drop_scratch_pad_entries:Cell<u8>,
    param_heap:Cell<u32>,
}


pub trait TransactionVerificationContext<S:Store, B:TransactionBundle> {
    fn new() -> Self;
    //reads the desc and accounts for its potential gas usage
    fn read_transaction_desc<'b, A:ParserAllocator>(&self, ctx:&Context<S,B>, target:&Hash, heap:&'b A) -> Result<TransactionDescriptor<'b>>;
    //accounts for the loading of an entry
    fn account_for_chain_value_load(&self, ctx:&Context<S,B>, param:TxTParam, first_access:bool);
    //accounts for the deletion of an entry
    fn account_for_chain_value_delete(&self, ctx:&Context<S,B>, param:TxTParam, first_access:bool);
    //accounts for the insertion of an entry
    fn account_for_chain_value_store(&self, ctx:&Context<S,B>, ret:TxTReturn);
    //informs the Accounter of a Section end
    // which responds with the sections gas cost
    fn store_access_gas(&self, ctx:&Context<S,B>) -> u64;
    //checks if a type represents a chain storable entry
    fn is_chain_value(&self, ctx:&Context<S,B>, typ:Ptr<RuntimeType>) -> bool;
    //checks if a type represents a providable entry (returns gas to create & size on haep)
    fn verify_providable(&self, ctx:&Context<S,B>, typ:Ptr<RuntimeType>, section_no:u8,  txt_no:u8) -> Result<(u64,u32)>;

}


//TODO: Make cmplete rework of the fast stuff and see if with the witnesses we can get it in order
//      In storage / provider mode do not execute payment txts (for now)
//

//Todo: We need evaluation Mode
//      Takes Bundle but with limits & ev: witnesses set to 0/null
//      Then it evaluates them and returns the filled bundle data


pub fn verify_repeated<'c, SYS:SystemContext<'c>>(ctx:&Context<SYS::S,SYS::B>,  block_no:u64) -> Result<()> {
    //check that it is in window
    if block_no < ctx.txt_bundle.earliest_block() || block_no >= ctx.txt_bundle.earliest_block() + CONFIG.block_inclusion_window {
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
pub fn verify_once<'c, SYS:SystemContext<'c>>(acc_ctx:&SYS::VC, ctx:&Context<SYS::S, SYS::B>, heap:&Heap) -> Result<()> {
    //Calculate the payment information
    //Starts with the parsing costs which are already done mostly but this is inevitable (a miner could have a size limit)
    //This includes encoding the parameter witnesses

    let entry_types = RefCell::new(alloc::vec::from_elem(Option::None, ctx.txt_bundle.stored().len()));
    let literal_types = RefCell::new(alloc::vec::from_elem(Option::None, ctx.txt_bundle.literal().len()));
    let witness_types  = RefCell::new(alloc::vec::from_elem(Option::None,ctx.txt_bundle.witness().len()));
    let scratch_pad_types  = RefCell::new(alloc::vec::from_elem(Option::None,ctx.txt_bundle.scratch_pad_slots() as usize));

    if CONFIG.max_txt_alloc < ctx.txt_bundle.transaction_heap_limit() as usize {
        return error(||"Transaction Descriptors use to much memory")
    }

    //Todo: Shall we do lazy? -- currently all the txt loads count to essential cost
    let desc_alloc = heap.new_virtual_arena(ctx.txt_bundle.transaction_heap_limit() as usize);
    let mut desc_builder = desc_alloc.slice_builder(ctx.txt_bundle.descriptors().len())?;
    for desc_hash in ctx.txt_bundle.descriptors().iter() {
        desc_builder.push(acc_ctx.read_transaction_desc(ctx, desc_hash, &desc_alloc)?);
    }


    let verify_env = VerificationEnvironment {
        descs: desc_builder.finish(),
        //Input type checks
        entry_types,
        literal_types,
        witness_types,
        scratch_pad_types,
        num_non_drop_scratch_pad_entries: Cell::new(0),
        param_heap: Cell::new(0)
    };

    let mut required_gas = CONFIG.bundle_base_cost + CONFIG.parsing_cost.compute(ctx.txt_bundle.byte_size() as u64);
    let mut essential_gas = required_gas;

    let mut is_in_essential = true;

    let mut sec_no = 0;
    //todo: Check that first section is essential or modify essential const comp for case where 0 essentials
    for txt_section in ctx.txt_bundle.sections().iter() {
        let mut txt_no = 0;
        for txt in txt_section.txts.iter() {
            required_gas += verify_transaction::<SYS>(&verify_env, acc_ctx, ctx, txt, txt_section.typ, sec_no, txt_no)? as u64;
            if txt_no == u8::max_value() {
                //Check Txt Limit
                return error(||"to many transactions in a section only 256 are allowed")
            }
            txt_no+=1;
        }

        if sec_no == u8::max_value() {
            //Check Section Limit
            return error(||"to many sections in a bundle only 256 are allowed")
        }
        sec_no+=1;

        if verify_env.num_non_drop_scratch_pad_entries.get() != 0 {
            //Ensure that on  rollback we do not violate substructural type integrety
            return error(||"At the end of a section scratch pad can only contain values that can be dropped")
        }

        required_gas += acc_ctx.store_access_gas(ctx);

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
    if essential_gas > ctx.txt_bundle.essential_gas_cost() {
        return error(||"Bundle has declared wrong essential gas cost")
    }

    if required_gas > ctx.txt_bundle.total_gas_cost() {
        return error(||"Bundle has declared wrong total gas cost")
    }

    if verify_env.param_heap.get() > ctx.txt_bundle.param_heap_limit() as u32 {
        return error(||"Bundle has not reserved enough parameter heap space")
    }

    Ok(())
}



fn verify_transaction<'c, SYS:SystemContext<'c>>(env:&VerificationEnvironment, acc_ctx:&SYS::VC, ctx:&Context<SYS::S, SYS::B>, txt:&Transaction, sec_typ:SectionType, sec_no:u8, txt_no:u8) -> Result<u64>{
    //Prepare all the Memory
    if  env.descs.len() <= txt.txt_desc as usize { return error(||"Descriptor index out of range")  }
    let txt_desc = env.descs[txt.txt_desc as usize];

    if txt_desc.max_frames > ctx.txt_bundle.stack_frame_limit() {return error(||"Bundle has not reserved enough frame space")}
    if txt_desc.max_stack > ctx.txt_bundle.stack_elem_limit() {return error(||"Bundle has not reserved enough stack space")}
    if txt_desc.max_mem > ctx.txt_bundle.runtime_heap_limit() {return error(||"Bundle has not reserved enough heap space")}

    //push everything required onto the stack
    let mut lock_set = BTreeSet::new();
    let mut scratch_lock_set = alloc::vec::from_elem(false, ctx.txt_bundle.scratch_pad_slots() as usize);

    let mut gas = txt_desc.gas_cost as u64;

    for (p,p_typ) in txt_desc.params.iter().zip(txt.params.iter()) {
        match p_typ {
            ParamRef::Load(ParamMode::Consume,index) => {
                let first_access = check_store_type(env, *index, *p)?;
                //This will account for gas, depending on how the used storage works
                acc_ctx.account_for_chain_value_load(ctx, *p, first_access);
                acc_ctx.account_for_chain_value_delete(ctx, *p, first_access);
                if !p.consumes && !p.drop { return error(||"A owned store value must be consumed or dropped") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if !acc_ctx.is_chain_value(ctx, p.typ) { return error(|| "Value parameter must be an entry") }
                if ctx.txt_bundle.stored().len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = &ctx.txt_bundle.stored()[*index as usize];
                if lock_set.contains(hash) { return error(||"An entry can only be fetched once"); }
                lock_set.insert(hash.clone());
            }

            ParamRef::Load(ParamMode::Copy, index) => {
                let first_access = check_store_type(env, *index, *p)?;
                //This will account for gas, depending on how the used storage works
                acc_ctx.account_for_chain_value_load(ctx, *p, first_access);
                if !p.copy { return error(||"A Copied store value must allow copy") }
                if !p.consumes && !p.drop { return error(||"A Copied store value must be consumed or dropped") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if !acc_ctx.is_chain_value(ctx,p.typ) { return error(|| "Value parameter must be an entry") }
                if ctx.txt_bundle.stored().len() <= *index as usize { return error(||"Value index out of range")  }
            },

            ParamRef::Load(ParamMode::Borrow, index) => {
                let first_access = check_store_type(env, *index, *p)?;
                //This will account for gas, depending on how the used storage works
                acc_ctx.account_for_chain_value_load(ctx, *p, first_access);
                if p.consumes { return error(||"A Borrowed store value can not be consumed") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if !acc_ctx.is_chain_value(ctx,p.typ) { return error(|| "Value parameter must be an entry") }
                if ctx.txt_bundle.stored().len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = &ctx.txt_bundle.stored()[*index as usize];
                if lock_set.contains(hash) { return error(||"An entry can only be fetched once per transaction"); }
                lock_set.insert(hash.clone());
            },

            ParamRef::Fetch(ParamMode::Consume,index) => {
                if ctx.txt_bundle.scratch_pad_slots() <= *index { return error(||"Scratch pad value index out of range")  }
                check_scratch_pad_type(env, *index, *p)?;
                //This will account for gas, depending on how the used storage works
                if !p.consumes && !p.drop { return error(||"A owned scratch pad value must be consumed or dropped") }
                //Consume the entry
                env.scratch_pad_types.borrow_mut()[*index as usize] = None;
                if !p.drop {
                    //One non droppable entry less
                    env.num_non_drop_scratch_pad_entries.set(env.num_non_drop_scratch_pad_entries.get() -1)
                }
            }

            ParamRef::Fetch(ParamMode::Copy, index) => {
                if ctx.txt_bundle.scratch_pad_slots() <= *index { return error(||"Scratch pad value index out of range")  }
                check_scratch_pad_type(env, *index, *p)?;
                //This will account for gas, depending on how the used storage works
                if !p.copy { return error(||"A copied scratch pad value must allow copy") }
                if !p.consumes && !p.drop { return error(||"A copied scratch pad value  must be consumed or dropped") }
            },

            ParamRef::Fetch(ParamMode::Borrow, index) => {
                if ctx.txt_bundle.scratch_pad_slots() <= *index { return error(||"Scratch pad value index out of range")  }
                check_scratch_pad_type(env, *index, *p)?;
                if p.consumes { return error(||"A borrowed scratch pad value can not be consumed") }
                if scratch_lock_set[*index as usize] { return error(||"A scratch pad entry can only be fetched once per transaction"); }
                scratch_lock_set[*index as usize] = true;
            },

            ParamRef::Provided => {
                let (cost,size) = acc_ctx.verify_providable(ctx, p.typ, sec_no, txt_no)?;
                gas += cost;
                env.param_heap.set(env.param_heap.get() + size);
            },
            ParamRef::Literal(index) => {
                if !p.primitive { return error(||"Literals must be of primitive type") }
                if ctx.txt_bundle.literal().len() <= *index as usize { return error(||"Value index out of range")  }
                if check_literal_type(env, *index, *p)? {
                    //Todo: Shall we use real size instead? We do now
                    gas += CONFIG.parsing_cost.compute(p.desc.max_runtime_size()? as u64)
                }
            },
            ParamRef::Witness(index) => {
                if sec_typ != SectionType::Essential { return error(||"Witnesses can only be used in essential sections") }
                if !p.primitive { return error(||"Witnesses must be of primitive type") }
                if ctx.txt_bundle.witness().len() <= *index as usize { return error(||"Value index out of range")  }
                if check_witness_type(env, *index, *p)? {
                    //Todo: Shall we use real size instead? We do now
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
                        acc_ctx.account_for_chain_value_store(ctx, *r);
                        if primitive {return error(||"Can not store primitives") }
                        if !acc_ctx.is_chain_value(ctx,typ) { return error(||"Stored return must be an entry") }
                    },
                    RetType::Put(index) => {
                        if ctx.txt_bundle.scratch_pad_slots() <= *index { return error(||"Scratch pad value index out of range")  }
                        if env.scratch_pad_types.borrow()[*index as usize] != None {
                            return error(||"Scratch pad slot index already occupied")
                        }
                        env.scratch_pad_types.borrow_mut()[*index as usize] = Some(r.typ);
                        if !r.drop {
                            env.num_non_drop_scratch_pad_entries.set(env.num_non_drop_scratch_pad_entries.get()+1)
                        }
                        let runtime_size = r.desc.max_runtime_size()?;
                        gas += CONFIG.copy_cost.compute(runtime_size as u64);
                        env.param_heap.set(env.param_heap.get() + runtime_size as u32);
                    }

                    RetType::Drop => {
                        if !drop { return error(||"Returns without drop capability must be stored or scratched") }
                    },
                    RetType::Log => {
                        //Todo: Logs can be costly we should charge
                        if !drop { return error(||"Returns without drop capability must be stored or scratched") }
                    },
                }
            },
        }
    }
    Ok(gas)
}

fn check_scratch_pad_type<'a>(env:&VerificationEnvironment<'a>, index:u8, param:TxTParam<'a>) -> Result<()> {
    let entry_copy = env.scratch_pad_types.borrow()[index as usize];
    match entry_copy {
        None => return error(|| "Scratch pad entry was not available"),
        Some(expected_typ) => {
            if param.typ != expected_typ {
                return error(|| "A scratch pad value is referred to over different types")
            }
        }
    }
    Ok(())
}

fn check_literal_type<'a>(env:&VerificationEnvironment<'a>, index:u16, param:TxTParam<'a>) -> Result<bool> {
    let entry_copy = env.literal_types.borrow()[index as usize];
    match entry_copy {
        None => {
            env.param_heap.set(env.param_heap.get() + param.desc.max_runtime_size()? as u32);
            env.literal_types.borrow_mut()[index as usize] = Some(param.typ);
            Ok(true)
        },
        Some(expected_typ) => {
            if param.typ != expected_typ { return error(|| "A single literal value is referred to over different types") }
            Ok(false)
        }
    }
}

fn check_witness_type<'a>(env:&VerificationEnvironment<'a>, index:u16, param:TxTParam<'a>) -> Result<bool> {
    let entry_copy = env.witness_types.borrow()[index as usize];
    match entry_copy {
        None => {
            env.param_heap.set(env.param_heap.get() + param.desc.max_runtime_size()? as u32);
            env.witness_types.borrow_mut()[index as usize] = Some(param.typ);
            Ok(true)
        },
        Some(expected_typ) => {
            if param.typ != expected_typ { return error(|| "A single witness value is referred to over different types") }
            Ok(false)
        }
    }
}


fn check_store_type<'a>(env:&VerificationEnvironment<'a>,index:u16, param:TxTParam<'a>) -> Result<bool> {
    let entry_copy = env.entry_types.borrow()[index as usize];
    match entry_copy {
        None => {
            env.param_heap.set(env.param_heap.get() + param.desc.max_runtime_size()? as u32);
            env.entry_types.borrow_mut()[index as usize] = Some(param.typ);
            Ok(true)
        },
        Some(expected_typ) => {
            if param.typ != expected_typ { return error(||"A single store value is referred to over different types")}
            Ok(false)
        }
    }
}