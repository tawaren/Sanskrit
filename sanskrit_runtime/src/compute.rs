use sanskrit_common::errors::*;
use sanskrit_common::encoding::{Parser, ParserAllocator};
use model::{Transaction, ParamRef, RetType, ParamMode};
use sanskrit_common::model::{Hash, Ptr, SlicePtr};
//use ed25519_dalek::*;
//use sha2::{Sha512};
use sanskrit_common::arena::*;
use sanskrit_interpreter::interpreter::{Frame, ExecutionContext};
use sanskrit_interpreter::model::{Entry, TransactionDescriptor, TxTParam, TxTReturn, RuntimeType};
use alloc::vec::Vec;

use ::{Tracker, CONFIG};
use core::cell::RefCell;
use sanskrit_common::store::Store;
use ::{Context, TransactionBundle};
use system::SystemContext;

//A struct holding context information of the current transaction
pub struct ExecutionEnvironment<'a, 'b, 'c> {
    parameter_heap:&'b VirtualHeapArena<'c>,
    descs:SlicePtr<'a,TransactionDescriptor<'a>>,

    structural_arena:HeapArena<'c>,
    runtime_heap:VirtualHeapArena<'c>,
    //Caches to parse each param just once
    entry_cache:RefCell<Vec<Option<Entry<'b>>>>,
    literal_cache:RefCell<Vec<Option<Entry<'b>>>>,
    witness_cache:RefCell<Vec<Option<Entry<'b>>>>,
    scratch_pad:RefCell<Vec<Option<Entry<'b>>>>,

}

pub trait TransactionExecutionContext<S:Store,B:TransactionBundle> {
    fn new() -> Self;
    //reads the desc
    fn read_transaction_desc<'d, A:ParserAllocator>(&self, ctx:&Context<S,B>, target:&Hash, heap:&'d A) -> Result<TransactionDescriptor<'d>>;
    //create a provided value
    //Todo: Do we need some extra info??? in order to select
    fn create_provided_value<'a,'h>(&self, ctx:&Context<S,B>, typ:Ptr<RuntimeType>, alloc:&'a VirtualHeapArena<'h>, block_no:u64, section_no:u8,  txt_no:u8, p_num:u8) -> Result<Entry<'a>>;
    //loads an entry
    fn chain_value_load<'b>(&self, ctx:&Context<S,B>, index:u16, param:TxTParam, parameter_heap:&'b VirtualHeapArena) -> Result<Entry<'b>>;
    //deletes an entry
    fn chain_value_delete(&self, ctx:&Context<S,B>, index:u16) -> Result<()>;
    //stores an entry
    fn chain_value_store(&self, ctx:&Context<S,B>, entry:&Entry, ret:TxTReturn) -> Result<()>;
    //commits changes to backend
    fn commit(&self, ctx:&Context<S,B>);
    //reverts changes to last commit
    fn revert(&self, ctx:&Context<S,B>);
}

//Executes a transaction
pub fn execute_once<'c, L: Tracker, SYS:SystemContext<'c>>(exec_store:&SYS::EC, ctx:&Context<SYS::S, SYS::B>, block_no:u64, heap:&Heap, tracker:&mut L) -> Result<()> {
    //Create Allocator
    //create heaps: based on bundle input
    let structural_arena = heap.new_arena(
        Heap::elems::<Entry>(ctx.txt_bundle.stack_elem_limit() as usize)
            + Heap::elems::<Frame>(ctx.txt_bundle.stack_frame_limit() as usize)
            + Heap::elems::<Entry>(CONFIG.return_stack)
    );

    let parameter_heap = heap.new_virtual_arena(ctx.txt_bundle.param_heap_limit() as usize);
    let runtime_heap = heap.new_virtual_arena(ctx.txt_bundle.runtime_heap_limit() as usize);

    let entry_cache = RefCell::new(alloc::vec::from_elem(Option::None, ctx.txt_bundle.stored().len()));
    let literal_cache = RefCell::new(alloc::vec::from_elem(Option::None, ctx.txt_bundle.literal().len()));
    let witness_cache = RefCell::new(alloc::vec::from_elem(Option::None,ctx.txt_bundle.witness().len()));
    let scratch_pad = RefCell::new(alloc::vec::from_elem(Option::None,ctx.txt_bundle.scratch_pad_slots() as usize));

    //Todo: Shall we do lazy? -- currently all the txt loads count to essential cost
    let desc_alloc = heap.new_virtual_arena(ctx.txt_bundle.transaction_heap_limit() as usize);
    let mut desc_builder = desc_alloc.slice_builder(ctx.txt_bundle.descriptors().len())?;
    for desc_hash in ctx.txt_bundle.descriptors().iter() {
        desc_builder.push(exec_store.read_transaction_desc(ctx, desc_hash, &desc_alloc)?);
    }

    let mut exec_env = ExecutionEnvironment {
        descs: desc_builder.finish(),
        structural_arena,
        parameter_heap: &parameter_heap,
        runtime_heap,
        entry_cache,
        literal_cache,
        witness_cache,
        scratch_pad
    };

    let mut sec_no = 0;
    for txt_section in ctx.txt_bundle.sections().iter() {
        tracker.section_start(txt_section);
        let mut txt_no = 0;
        for txt in txt_section.txts.iter() {
            tracker.transaction_start(txt);
            match execute_transaction::<_, SYS>(&exec_env, exec_store, ctx, txt, block_no, sec_no, txt_no, tracker) {
                Ok(_) => {},
                Err(err) => {
                    exec_store.revert(ctx);
                    tracker.transaction_finish(txt, false);
                    tracker.section_finish(txt_section, false);
                    return Err(err);
                }
            };
            txt_no +=1;
            //release all the memory so it does not leak into the next transaction
            exec_env.structural_arena = exec_env.structural_arena.reuse();
            exec_env.runtime_heap = exec_env.runtime_heap.reuse();
            tracker.transaction_finish(txt, true);
        }

        //commit
        exec_store.commit(ctx);
        tracker.section_finish(txt_section, true);
        sec_no+=1;

    }
    Ok(())
}

fn execute_transaction<'c, L: Tracker, SYS:SystemContext<'c>>(env:&ExecutionEnvironment, exec_store:&SYS::EC, ctx:&Context<SYS::S, SYS::B>, txt:&Transaction, block_no:u64, sec_no:u8, txt_no:u8,  tracker:&mut L) -> Result<()>{

    //Prepare all the Memory
    let txt_desc:TransactionDescriptor = env.descs[txt.txt_desc as usize];

    let mut interpreter_stack = env.structural_arena.alloc_stack::<Entry>(txt_desc.max_stack as usize);
    let mut frame_stack = env.structural_arena.alloc_stack::<Frame>(txt_desc.max_frames as usize);
    let mut return_stack = env.structural_arena.alloc_stack::<Entry>(CONFIG.return_stack);

    //push everything required onto the stack
    let mut deletes = Vec::with_capacity(txt_desc.params.len());

    for (p_num, (p,p_typ)) in txt_desc.params.iter().zip(txt.params.iter()).enumerate() {
        match p_typ {
            ParamRef::Load(ParamMode::Consume,index) => {
                //We delete at end so others can copy and in case it produces an error it must still be their
                deletes.push(index);
                let data = load_from_store::<SYS>(env, exec_store, ctx, *index, *p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            }
            ParamRef::Load(ParamMode::Copy, index)
            | ParamRef::Load(ParamMode::Borrow, index) => {
                let data = load_from_store::<SYS>(env, exec_store, ctx, *index, *p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            },

            ParamRef::Provided => {
                let data = exec_store.create_provided_value(ctx, p.typ, &env.parameter_heap, block_no, sec_no, txt_no, p_num as u8)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            },

            ParamRef::Fetch(_, index) => {
                let data = env.scratch_pad.borrow()[*index as usize].unwrap();
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            }

            ParamRef::Literal(index) => {
                let data = load_from_literal::<SYS>(env, ctx,*index,*p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            },
            ParamRef::Witness(index) => {
                let data = load_from_witness::<SYS>(env, ctx, *index,*p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            },
        };
    }

    ExecutionContext::interpret::<SYS::RE>(&txt_desc.functions, &mut interpreter_stack, &mut frame_stack, &mut return_stack, &env.runtime_heap)?;

    //Now that we know it succeeds we can modify the store
    for index in deletes {
        exec_store.chain_value_delete(ctx, *index)?;
    }

    assert_eq!(interpreter_stack.len(), txt.returns.len(), "Transaction Return Information missmatched Stack");
    assert_eq!(interpreter_stack.len(), txt_desc.returns.len(), "Transaction Description Return Information missmatched Stack");

    for ((ret_entry, r), r_typ) in interpreter_stack.as_slice().iter().zip(txt_desc.returns.iter()).zip(txt.returns.iter()) {
        match r_typ {
            RetType::Store => {
                tracker.return_value(r_typ, r, ret_entry);
                exec_store.chain_value_store(ctx, ret_entry, *r)?
            },
            RetType::Put(index) => {
                tracker.return_value(r_typ, r, ret_entry);
                let copy = r.desc.move_value(*ret_entry, env.parameter_heap)?;
                env.scratch_pad.borrow_mut()[*index as usize] = Some(copy)
            }
            RetType::Drop => tracker.return_value(r_typ, r, ret_entry),
            RetType::Log => tracker.return_value(r_typ, r, ret_entry),
        }
    }
    Ok(())
}


fn load_from_literal<'a, 'b,'c, 'd, SYS:SystemContext<'d>>(env:&ExecutionEnvironment<'a, 'b, 'c>, ctx:&Context<SYS::S, SYS::B>, index:u16, param:TxTParam) -> Result<Entry<'b>> {
    let entry_copy = env.literal_cache.borrow()[index as usize];
    Ok(match entry_copy {
        None => {
            let data = ctx.txt_bundle.literal()[index as usize];
            let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
            let entry = param.desc.parse_value(&mut parser, env.parameter_heap)?;
            env.literal_cache.borrow_mut()[index as usize] = Some(entry,);
            entry
        },
        Some(entry) => entry
    })
}

fn load_from_witness<'a, 'b,'c, 'd, SYS:SystemContext<'d>>(env:&ExecutionEnvironment<'a, 'b, 'c>, ctx:&Context<SYS::S, SYS::B>, index:u16, param:TxTParam) -> Result<Entry<'b>> {
    let entry_copy = env.witness_cache.borrow()[index as usize];
    Ok(match entry_copy {
        None => {
            let data = ctx.txt_bundle.witness()[index as usize];
            let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
            let entry = param.desc.parse_value(&mut parser, env.parameter_heap)?;
            env.witness_cache.borrow_mut()[index as usize] = Some(entry,);
            entry
        },
        Some(entry) => entry
    })
}


fn load_from_store<'a, 'b,'c, 'd, SYS:SystemContext<'d>>(env:&ExecutionEnvironment<'a,'b, 'c>, exec_store: &SYS::EC, ctx:&Context<SYS::S, SYS::B>, index:u16, param:TxTParam) -> Result<Entry<'b>> {
    let entry_copy = env.entry_cache.borrow()[index as usize];
    Ok(match entry_copy {
        None => {
            let entry = exec_store.chain_value_load(ctx, index, param, env.parameter_heap)?;
            env.entry_cache.borrow_mut()[index as usize] = Some(entry);
            entry
        },
        Some(entry) => entry
    })
}