use sanskrit_common::errors::*;
use sanskrit_common::encoding::{Parser, ParserAllocator};
use model::{Transaction, ParamRef, RetType, TransactionBundle, ParamMode};
use sanskrit_common::model::Hash;
//use ed25519_dalek::*;
//use sha2::{Sha512};
use sanskrit_common::arena::*;
use sanskrit_interpreter::interpreter::{Frame, ExecutionContext};
use sanskrit_interpreter::model::{Entry, Adt, TransactionDescriptor, TxTParam, TxTReturn};
use alloc::vec::Vec;

use ::{Tracker, CONFIG};
use core::cell::RefCell;

//A struct holding context information of the current transaction
pub struct ExecutionEnvironment<'b, 'c> {
    parameter_heap:&'b VirtualHeapArena<'c>,
    max_desc_alloc:VirtualHeapArena<'c>,
    structural_arena:HeapArena<'c>,
    runtime_heap:VirtualHeapArena<'c>,
    //Caches to parse each param just once
    entry_cache:RefCell<Vec<Option<Entry<'b>>>>,
    literal_cache:RefCell<Vec<Option<Entry<'b>>>>,
    witness_cache:RefCell<Vec<Option<Entry<'b>>>>,
}

pub trait ExecutableStore {
    //get bundle Hash
    fn get_bundle_hash(&self) -> Hash;
    // get full bundle Hash
    fn get_full_bundle_hash(&self) -> Hash;
    //get the bundle
    fn get_transaction_bundle(&self) -> &TransactionBundle;
    //reads the desc
    fn read_transaction_desc<'c, A:ParserAllocator>(&self, target:&Hash, heap:&'c A) -> Result<TransactionDescriptor<'c>>;
    //loads an entry
    fn entry_load<'b>(&self, index:u16, param:TxTParam,  parameter_heap:&'b VirtualHeapArena) -> Result<Entry<'b>>;
    //deletes an entry
    fn entry_delete(&self, index:u16) -> Result<()>;
    //stores an entry
    fn entry_store(&self, entry:&Entry, ret:TxTReturn) -> Result<()>;
    //commits changes to backend
    fn commit(&self);
    //reverts changes to last commit
    fn revert(&self);
}

//Executes a transaction
pub fn execute_once<ES:ExecutableStore, L: Tracker>(exec_store:&ES, block_no:u64, heap:&Heap, tracker:&mut L) -> Result<()> {
    //Create Allocator
    //create heaps: based on bundle input
    let structural_arena = heap.new_arena(
        Heap::elems::<Entry>(exec_store.get_transaction_bundle().core.stack_elem_limit as usize)
            + Heap::elems::<Frame>(exec_store.get_transaction_bundle().core.stack_frame_limit as usize)
            + Heap::elems::<Entry>(CONFIG.return_stack)
    );

    let max_desc_alloc = heap.new_virtual_arena(exec_store.get_transaction_bundle().core.transaction_storage_heap as usize);
    let parameter_heap = heap.new_virtual_arena(exec_store.get_transaction_bundle().core.param_heap_limit as usize);
    let runtime_heap = heap.new_virtual_arena(exec_store.get_transaction_bundle().core.runtime_heap_limit as usize);

    let entry_cache = RefCell::new(alloc::vec::from_elem(Option::None, exec_store.get_transaction_bundle().core.stored.len()));
    let literal_cache = RefCell::new(alloc::vec::from_elem(Option::None, exec_store.get_transaction_bundle().core.literal.len()));
    let witness_cache = RefCell::new(alloc::vec::from_elem(Option::None,exec_store.get_transaction_bundle().witness.len()));

    let mut exec_env = ExecutionEnvironment {
        max_desc_alloc,
        structural_arena,
        parameter_heap: &parameter_heap,
        runtime_heap,
        entry_cache,
        literal_cache,
        witness_cache
    };

    for txt_section in exec_store.get_transaction_bundle().core.sections.iter() {
        tracker.section_start(txt_section);
        for txt in txt_section.txts.iter() {
            tracker.transaction_start(txt);
            match execute_transaction(&exec_env, exec_store, txt, block_no, tracker) {
                Ok(_) => {},
                Err(err) => {
                    exec_store.revert();
                    tracker.transaction_finish(txt, false);
                    tracker.section_finish(txt_section, false);
                    return Err(err);
                }
            };

            //release all the memory so it does not leak into the next transaction
            exec_env.structural_arena = exec_env.structural_arena.reuse();
            exec_env.max_desc_alloc = exec_env.max_desc_alloc.reuse();
            exec_env.runtime_heap = exec_env.runtime_heap.reuse();
            tracker.transaction_finish(txt, true);
        }

        //commit
        exec_store.commit();
        tracker.section_finish(txt_section, true);

    }
    Ok(())
}

fn execute_transaction<ES:ExecutableStore, L: Tracker>(env:&ExecutionEnvironment, exec_store:&ES, txt:&Transaction, block_no:u64, tracker:&mut L) -> Result<()>{

    //Prepare all the Memory
    //Todo: just give in index???
    let target = &exec_store.get_transaction_bundle().core.descriptors[txt.txt_desc as usize];
    let txt_desc:TransactionDescriptor = exec_store.read_transaction_desc(target,  &env.max_desc_alloc)?;

    let mut interpreter_stack = env.structural_arena.alloc_stack::<Entry>(txt_desc.max_stack as usize);
    let mut frame_stack = env.structural_arena.alloc_stack::<Frame>(txt_desc.max_frames as usize);
    let mut return_stack = env.structural_arena.alloc_stack::<Entry>(CONFIG.return_stack);

    //push everything required onto the stack
    let mut deletes = Vec::with_capacity(txt_desc.params.len());

    for (p,p_typ) in txt_desc.params.iter().zip(txt.params.iter()) {
        match p_typ {
            ParamRef::Load(ParamMode::Consume,index) => {
                //We delete at end so others can copy and in case it produces an error it must still be their
                deletes.push(index);
                let data = load_from_store(env, exec_store, *index, *p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            }
            ParamRef::Load(ParamMode::Copy, index) => {
                let data = load_from_store(env, exec_store, *index, *p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            },
            ParamRef::Load(ParamMode::Borrow, index) => {
                let data = load_from_store(env, exec_store, *index, *p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            },

            ParamRef::Provided => {
                let data = create_ctx(&env.parameter_heap, &exec_store.get_bundle_hash(), &exec_store.get_full_bundle_hash(),  block_no)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            },

            ParamRef::Literal(index) => {
                let data = load_from_literal(env, exec_store,*index,*p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            },
            ParamRef::Witness(index) => {
                let data = load_from_witness(env, exec_store, *index,*p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            },
        };
    }

    ExecutionContext::interpret(&txt_desc.functions, &mut interpreter_stack, &mut frame_stack, &mut return_stack, &env.runtime_heap)?;

    //Now that we know it succeeds we can modify the store
    for index in deletes {
        exec_store.entry_delete(*index)?;
    }

    assert_eq!(interpreter_stack.len(), txt.returns.len(), "Transaction Return Information missmatched Stack");
    assert_eq!(interpreter_stack.len(), txt_desc.returns.len(), "Transaction Description Return Information missmatched Stack");

    for ((ret_entry, r), r_typ) in interpreter_stack.as_slice().iter().zip(txt_desc.returns.iter()).zip(txt.returns.iter()) {
        match r_typ {
            RetType::Store => {
                tracker.return_value(r_typ, r, ret_entry);
                exec_store.entry_store(ret_entry, *r)?
            },
            RetType::Drop => tracker.return_value(r_typ, r, ret_entry),
            RetType::Log => tracker.return_value(r_typ, r, ret_entry),
        }
    }
    Ok(())
}


fn load_from_literal<'b,'c, ES:ExecutableStore>(env:&ExecutionEnvironment<'b, 'c>, exec_store: &ES, index:u16, param:TxTParam) -> Result<Entry<'b>> {
    let entry_copy = env.literal_cache.borrow()[index as usize];
    Ok(match entry_copy {
        None => {
            let data = exec_store.get_transaction_bundle().core.literal[index as usize];
            let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
            let entry = param.desc.parse_value(&mut parser, env.parameter_heap)?;
            env.literal_cache.borrow_mut()[index as usize] = Some(entry,);
            entry
        },
        Some(entry) => entry
    })
}

fn load_from_witness<'b,'c, ES:ExecutableStore>(env:&ExecutionEnvironment<'b, 'c>, exec_store: &ES, index:u16, param:TxTParam) -> Result<Entry<'b>> {
    let entry_copy = env.witness_cache.borrow()[index as usize];
    Ok(match entry_copy {
        None => {
            let data = exec_store.get_transaction_bundle().witness[index as usize];
            let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
            let entry = param.desc.parse_value(&mut parser, env.parameter_heap)?;
            env.witness_cache.borrow_mut()[index as usize] = Some(entry,);
            entry
        },
        Some(entry) => entry
    })
}



fn load_from_store<'b,'c, ES:ExecutableStore>(env:&ExecutionEnvironment<'b, 'c>, exec_store: &ES, index:u16, param:TxTParam) -> Result<Entry<'b>> {
    let entry_copy = env.entry_cache.borrow()[index as usize];
    Ok(match entry_copy {
        None => {
            let entry = exec_store.entry_load(index, param, env.parameter_heap)?;
            env.entry_cache.borrow_mut()[index as usize] = Some(entry);
            entry
        },
        Some(entry) => entry
    })
}


pub fn create_ctx<'a,'h>(alloc:&'a VirtualHeapArena<'h>, txt_hash:&Hash, full_hash:&Hash, block_no:u64) -> Result<Entry<'a>> {
    //Todo: construct over schema to be compiler agnostic
    Ok(Entry{adt: Adt(0,alloc.copy_alloc_slice(&[
        Entry {data: alloc.copy_alloc_slice(txt_hash)?},
        Entry {data: alloc.copy_alloc_slice(full_hash)?},
        Entry {u64: block_no},
        Entry {u64: 0}
    ])?)})
}