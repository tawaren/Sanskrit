#[cfg(test)]

use sanskrit_common::model::*;
use sanskrit_runtime::interpreter::Frame;
use sanskrit_runtime::model::*;
use sanskrit_common::arena::Heap;
use test::Bencher;
use sanskrit_runtime::ContextEnvironment;
use sanskrit_runtime::interpreter::ExecutionContext;
use sanskrit_common::arena::VirtualHeapArena;
use sanskrit_runtime::Configuration;

pub const CONFIG: Configuration = Configuration {
    max_stack_depth:16*1024,
    max_frame_depth:512,
    max_heap_size:1024 * 1024,
    max_script_stack_size:256,
    max_code_size:512 * 1024,
    max_structural_dept:64,
    max_transaction_size:128 * 1024,
    max_store_slots: 32,
    temporary_buffer: 24 * 255 //recalc
};

fn execute_native<F>(b: &mut Bencher, v_gen:F, ops:&[Operand], do_it:bool)
    where F: for <'a> Fn(&'a VirtualHeapArena)->Vec<Vec<Ptr<'a,Object<'a>>>>
{
    execute_multi_ret_native(b, v_gen, ops, 1, do_it)
}

fn execute_multi_ret_native<F>(b: &mut Bencher, v_gen:F, ops:&[Operand], rets:usize, do_it:bool)
    where F: for <'a> Fn(&'a VirtualHeapArena)->Vec<Vec<Ptr<'a,Object<'a>>>>
{
    let heap = Heap::new(CONFIG.calc_heap_size(2), 2.0);
    //Create Allocator
    let size_interpreter_stack = Heap::elems::<Ptr<Object>>(CONFIG.max_stack_depth);
    let size_frame_stack = Heap::elems::<Frame>(CONFIG.max_frame_depth);
    let size_code_alloc = CONFIG.max_code_size;
    let size_alloc = CONFIG.max_heap_size;

    let temporary_values = heap.new_arena(CONFIG.temporary_buffer);
    let alloc = heap.new_virtual_arena(size_alloc);
    let code_alloc = heap.new_virtual_arena(size_code_alloc);
    let structural_arena = heap.new_arena(size_interpreter_stack + size_frame_stack);
    let env = ContextEnvironment {
        block_no:0,
        full_hash:[0;20],
        txt_hash:[0;20],
        code_hash:[0;20]
    };
    let vals_sets = v_gen(&alloc);
    let first = vals_sets[0].len();
    assert!(vals_sets.iter().all(|s|s.len() == first));
    let stack = 1000*rets;
    let mut code = code_alloc.slice_builder(stack).unwrap();
    let mut elems = 0;
    'outer: loop {
        for (s1,vals) in vals_sets.iter().enumerate() {
            for op in ops.iter() {
                let mut ins = code_alloc.slice_builder(vals.len()).unwrap();
                for a in 0..vals.len() {
                    ins.push(ValueRef((elems+s1*vals.len()+a) as u16));
                }
                let ins = ins.finish();
                code.push(OpCode::Invoke(FunDesc::Native(*op), ins));
                elems+=rets;
                if elems >= stack {break 'outer}
            }
        }
    }
    let code = code.finish();
    let exp = code_alloc.alloc(Exp::Ret(code,SlicePtr::empty())).unwrap();
    let funs = code_alloc.copy_alloc_slice(&[exp;1]).unwrap();

    //here bench start
    b.iter(||{
        let tmp_alloc = alloc.temp_arena().unwrap();
        let tmp = temporary_values.temp_arena();
        let tmp_struct = structural_arena.temp_arena();
        let mut value_stack = tmp_struct.alloc_stack(CONFIG.max_stack_depth);
        let mut frame_stack = tmp_struct.alloc_stack(CONFIG.max_frame_depth);
        for vals in &vals_sets {
            for v in vals {
                value_stack.push(*v).unwrap();
            }
        }

        if do_it {
            ExecutionContext::interpret(env,&funs, &mut value_stack, &mut frame_stack, &tmp_alloc, &tmp).unwrap();
        }
    })
}

//i do not like the duplication bur found no otehr way
fn execute_code<F,O>(b: &mut Bencher, v_gen:F, op:O, rets:usize, do_it:bool)
    where F: for <'a> Fn(&'a VirtualHeapArena)->Vec<Vec<Ptr<'a,Object<'a>>>>,
          O: for <'a> Fn(SlicePtr<'a,ValueRef>)->OpCode<'a>

{
    let heap = Heap::new(CONFIG.calc_heap_size(2), 2.0);
    //Create Allocator
    let size_interpreter_stack = Heap::elems::<Ptr<Object>>(CONFIG.max_stack_depth);
    let size_frame_stack = Heap::elems::<Frame>(CONFIG.max_frame_depth);
    let size_code_alloc = CONFIG.max_code_size;
    let size_alloc = CONFIG.max_heap_size;

    let temporary_values = heap.new_arena(CONFIG.temporary_buffer);
    let alloc = heap.new_virtual_arena(size_alloc);
    let code_alloc = heap.new_virtual_arena(size_code_alloc);
    let structural_arena = heap.new_arena(size_interpreter_stack + size_frame_stack);
    let env = ContextEnvironment {
        block_no:0,
        full_hash:[0;20],
        txt_hash:[0;20],
        code_hash:[0;20]
    };
    let vals_sets = v_gen(&alloc);
    let first = vals_sets[0].len();
    assert!(vals_sets.iter().all(|s|s.len() == first));
    let stack = 1000*rets;
    let mut code = code_alloc.slice_builder(stack).unwrap();
    let mut elems = 0;
    'outer: loop {
        for (s1,vals) in vals_sets.iter().enumerate() {
            let mut ins = code_alloc.slice_builder(vals.len()).unwrap();
            for a in 0..vals.len() {
                ins.push(ValueRef((elems+s1*vals.len()+a) as u16));
            }
            let ins = ins.finish();
            code.push(op(ins));
            elems+=rets;
            if elems >= stack {break 'outer}
        }
    }
    let code = code.finish();
    let exp = code_alloc.alloc(Exp::Ret(code,SlicePtr::empty())).unwrap();
    let funs = code_alloc.copy_alloc_slice(&[exp;1]).unwrap();

    //here bench start
    b.iter(||{
        let tmp_alloc = alloc.temp_arena().unwrap();
        let tmp = temporary_values.temp_arena();
        let tmp_struct = structural_arena.temp_arena();
        let mut value_stack = tmp_struct.alloc_stack(CONFIG.max_stack_depth);
        let mut frame_stack = tmp_struct.alloc_stack(CONFIG.max_frame_depth);
        for vals in &vals_sets {
            for v in vals {
                value_stack.push(*v).unwrap();
            }
        }

        if do_it {
            ExecutionContext::interpret(env,&funs, &mut value_stack, &mut frame_stack, &tmp_alloc, &tmp).unwrap();
        }
    })
}



fn deep_obj<'a>(alloc:&'a VirtualHeapArena, leaf:Object<'a>, depth:u8) -> Ptr<'a,Object<'a>> {
    let mut cur = alloc.alloc(leaf).unwrap();
    for i in 0..depth{
        let mut builder = alloc.slice_builder(2).unwrap();
        builder.push(cur);
        builder.push(alloc.alloc(leaf).unwrap());
        cur = alloc.alloc(Object::Adt(0,builder.finish())).unwrap()
    }
    cur
}

fn wide_obj<'a>(alloc:&'a VirtualHeapArena, leaf:Object<'a>, width:u8) -> Ptr<'a,Object<'a>> {
    let mut builder = alloc.slice_builder(width as usize).unwrap();
    for i in 0..width{
        builder.push(alloc.alloc(leaf).unwrap());
    }
    alloc.alloc(Object::Adt(0,builder.finish())).unwrap()
}

fn tree_obj<'a>(alloc:&'a VirtualHeapArena, leaf:Object<'a>, depth:u8) -> Ptr<'a,Object<'a>> {
    if depth == 0 {
        alloc.alloc(leaf).unwrap()
    } else {
        let mut builder = alloc.slice_builder(2).unwrap();
        builder.push(tree_obj(alloc,leaf,depth-1));
        builder.push(tree_obj(alloc,leaf,depth-1));
        alloc.alloc(Object::Adt(0,builder.finish())).unwrap()
    }
}

mod add;
mod sub;
mod div;
mod mul;
mod arith;
mod and;
mod or;
mod xor;
mod bit_ops;
mod not;
mod copy;
mod to;
mod eq ;
mod hash;
mod plain_hash;
mod to_data;
mod concat;
mod lt;
mod lte;
mod gt;
mod gte;
mod cmp;
mod set_bit;
mod get_bit;
mod gen_unique;
mod derive;
mod env;
mod unpack;