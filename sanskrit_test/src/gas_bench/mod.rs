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
use sanskrit_runtime::script_stack::StackEntry;
use sanskrit_common::linear_stack::Elem;
use sanskrit_runtime::elem_store::CacheSlotMap;
use sanskrit_runtime::script_stack::LinearScriptStack;
use sanskrit_runtime::script_interpreter::Executor;
use sanskrit_memory_store::BTreeMapStore;
use sanskrit_runtime::elem_store::ElemStore;
use sanskrit_common::linear_stack::LinearStack;

pub const CONFIG: Configuration = Configuration {
    max_stack_depth:16*1024,
    max_frame_depth:512,
    max_heap_size:1024 * 1024,
    max_script_stack_size:2 * 1024,
    max_code_size:16 * 1024 * 1024,
    max_structural_dept:64,
    max_transaction_size:16 * 1024 * 1024,
    max_store_slots: 4,
    temporary_buffer: 24 * 255 //recalc
};

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum Operand {
    And,        //Deploys a logical and on bools or bitwise on ints
    Or,         //Deploys a logical or on bools or bitwise on ints
    Xor,        //Deploys a logical xor on bools or bitwise on ints
    Not,        //Deploys a logical not on bools or bitwise on ints
    ToU(u8),    //cast to an Unsigned Integer with u8 Bytes
    ToI(u8),    //cast to an Signed Integer with u8 Bytes
    Add,        //Does an arithmetic addition of two ints (throws on under or overflow)
    Sub,        //Does an arithmetic subtraction of two ints (throws on under or overflow)
    Mul,        //Does an arithmetic multiplication of two ints (throws on under or overflow)
    Div,        //Does an arithmetic dividation of two ints (throws on a division by zero)
    Eq,         //Compares two values for equality
    Hash,       //Calculates the hash of a value
    PlainHash,  //Calculates a plain hash for a data input (not structurally encoded)
    Lt,         //Compares two values to decide if one is less than the other
    Gt,         //Compares two values to decide if one is greater than the other
    Lte,        //Compares two values to decide if one is less than or equal the other
    Gte,        //Compares two values to decide if one is greater or equal than the other
    ToData,     //Transforms Integers & Uniques to data
    Concat,     //Concats two data values
    SetBit,     //sets a bit in a data value
    GetBit,     //queries a bit from a data value
    GenUnique,  //generates a new unique value (needs context for that)
    FullHash,   //gets the full hash of the current transactoion (needs context for that)
    TxTHash,    //gets the transaction hash (no witnesses) of the current transactoion (needs context for that)
    CodeHash,   //gets the hash of the currents transactions code  (needs context for that)
    BlockNo,    //gets the blockno in which the transaction is included
    GenIndex,   //generates a new storage index fro data or uniques
    Derive,     //derives a new index or referenz from two others
    Id,
}

impl Operand {
    fn opcode<'a>(&self, ptr:SlicePtr<'a,ValueRef>) -> OpCode<'a> {
        match *self {
            Operand::And => OpCode::And(ptr[0],ptr[1]),
            Operand::Or => OpCode::Or(ptr[0],ptr[1]),
            Operand::Xor => OpCode::Xor(ptr[0],ptr[1]),
            Operand::Not => OpCode::Not(ptr[0]),
            Operand::ToU(size) => OpCode::ToU(size, ptr[0]),
            Operand::ToI(size) => OpCode::ToI(size, ptr[0]),
            Operand::Add => OpCode::Add(ptr[0],ptr[1]),
            Operand::Sub => OpCode::Sub(ptr[0],ptr[1]),
            Operand::Mul => OpCode::Mul(ptr[0],ptr[1]),
            Operand::Div => OpCode::Div(ptr[0],ptr[1]),
            Operand::Eq => OpCode::Eq(ptr[0],ptr[1]),
            Operand::Hash => OpCode::Hash(ptr[0]),
            Operand::PlainHash => OpCode::PlainHash(ptr[0]),
            Operand::Lt => OpCode::Lt(ptr[0],ptr[1]),
            Operand::Gt => OpCode::Gt(ptr[0],ptr[1]),
            Operand::Lte => OpCode::Lte(ptr[0],ptr[1]),
            Operand::Gte => OpCode::Gte(ptr[0],ptr[1]),
            Operand::ToData => OpCode::ToData(ptr[0]),
            Operand::Concat => OpCode::Concat(ptr[0],ptr[1]),
            Operand::SetBit => OpCode::SetBit(ptr[0],ptr[1], ptr[2]),
            Operand::GetBit => OpCode::GetBit(ptr[0],ptr[1]),
            Operand::GenUnique => OpCode::GenUnique(ptr[0]),
            Operand::FullHash => OpCode::FullHash,
            Operand::TxTHash => OpCode::TxTHash,
            Operand::CodeHash => OpCode::CodeHash,
            Operand::BlockNo => OpCode::BlockNo,
            Operand::GenIndex => OpCode::GenIndex(ptr[0]),
            Operand::Derive => OpCode::Derive(ptr[0],ptr[1]),
            Operand::Id => OpCode::Id(ptr[0]),
        }
    }
}


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
                //todo: we do not need slice ptr, --  iter is ok
                let mut ins = code_alloc.slice_builder(vals.len()).unwrap();
                for a in 0..vals.len() {
                    ins.push(ValueRef((elems+s1*vals.len()+vals.len() - a - 1) as u16));
                }
                let ins = ins.finish();
                code.push(op.opcode(ins));
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
          O: for <'a> FnMut(&'a VirtualHeapArena, SlicePtr<'a,ValueRef>)->OpCode<'a>
{
    execute_code_with_extra_fun(b,v_gen,op,rets,|_|vec![],do_it)
}

//i do not like the duplication bur found no otehr way
fn execute_code_with_extra_fun<F,O,E>(b: &mut Bencher, v_gen:F, mut op:O, rets:usize, mut extra:E, do_it:bool)
    where F: for <'a> Fn(&'a VirtualHeapArena)->Vec<Vec<Ptr<'a,Object<'a>>>>,
          O: for <'a> FnMut(&'a VirtualHeapArena, SlicePtr<'a,ValueRef>)->OpCode<'a>,
          E: for <'a> FnMut(&'a VirtualHeapArena) -> Vec<Ptr<'a,Exp<'a>>>
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
    let mut code = code_alloc.slice_builder(1000).unwrap();
    let mut elems = 0;
    'outer: for _ in 0..1000 { //do at most 1000 (necessary if rets == 0)
        for (s1,vals) in vals_sets.iter().enumerate() {
            let mut ins = code_alloc.slice_builder(vals.len()).unwrap();
            for a in 0..vals.len() {
                ins.push(ValueRef((elems+s1*vals.len()+vals.len() - a - 1) as u16));
            }
            let ins = ins.finish();
            code.push(op(&code_alloc, ins));
            elems+=rets;
            if elems >= stack && rets != 0 {break 'outer} //rets != 0 necessary or we can not test that
        }
    }
    let code = code.finish();
    let exp = code_alloc.alloc(Exp::Ret(code,SlicePtr::empty())).unwrap();
    let efs = extra(&code_alloc);
    let mut funs = code_alloc.slice_builder(efs.len()+1).unwrap();
    funs.push(exp);
    for ef in efs {
        funs.push(ef)
    }
    let funs = funs.finish();

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
            ExecutionContext::interpret(env,&funs, &mut value_stack, &mut frame_stack, &tmp_alloc, &tmp).unwrap()
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

fn execute_script<F,O>(b: &mut Bencher, v_gen:F, mut code_gen:O, rets:usize, do_it:bool)
    where F: for <'a> Fn(&'a VirtualHeapArena)->Vec<Vec<StackEntry<'a>>>,
          O: for <'a> FnMut(&'a VirtualHeapArena, SlicePtr<'a,ValueRef>)->ScriptCode<'a>
{

    let store = BTreeMapStore::new();
    let heap = Heap::new(CONFIG.calc_heap_size(2), 2.0);
    //Create Allocator
    let size_script_stack = Heap::elems::<Elem<StackEntry,SlicePtr<usize>>>(CONFIG.max_script_stack_size);
    let size_code_alloc = CONFIG.max_code_size;
    let size_alloc = CONFIG.max_heap_size;
    let size_interpreter_stack = Heap::elems::<Ptr<Object>>(CONFIG.max_stack_depth);
    let size_frame_stack = Heap::elems::<Frame>(CONFIG.max_frame_depth);

    let txt_alloc = heap.new_virtual_arena(CONFIG.max_transaction_size); //todo: will be static conf later (or block consensus)
    let temporary_values = heap.new_arena(CONFIG.temporary_buffer);
    let alloc = heap.new_virtual_arena(size_alloc);
    let code_alloc = heap.new_virtual_arena(size_code_alloc);
    let structural_arena = heap.new_arena(size_interpreter_stack + size_frame_stack + size_script_stack);

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
    let mut script_code = txt_alloc.slice_builder(1000).unwrap();
    let mut elems = 0;
    'outer: for _ in 0..1000 { //do at most 1000 (necessary if rets == 0)
        for (s1,vals) in vals_sets.iter().enumerate() {
            let mut ins = code_alloc.slice_builder(vals.len()).unwrap();
            for a in 0..vals.len() {
                ins.push(ValueRef((elems+s1*vals.len()+vals.len() - a - 1) as u16));
            }
            let ins = ins.finish();
            script_code.push(txt_alloc.alloc( code_gen(&txt_alloc, ins)).unwrap());
            elems+=rets;
            if elems >= stack && rets != 0 {break 'outer} //rets != 0 necessary or we can not test that
        }
    }
    let script_code = script_code.finish();
    b.iter(||{
        let sub_alloc = alloc.temp_arena().unwrap();
        let sub_structural = structural_arena.temp_arena();
        let slot_map = CacheSlotMap::new(&sub_structural, CONFIG.max_store_slots,(0,0,0)).unwrap();
        let script_stack = sub_structural.alloc_stack(CONFIG.max_script_stack_size);

        let mut stack = LinearScriptStack::new(&sub_alloc,script_stack,&[], &[]).unwrap();

        for vals in &vals_sets {
            for v in vals {
                stack.provide(*v).unwrap();
            }
        }

        let mut exec = Executor{
            accounts: SlicePtr::empty(),
            witness: SlicePtr::empty(),
            newtypes: SlicePtr::empty(),
            imports: SlicePtr::empty(),
            stack,
            env,
            store:ElemStore::new(&store, slot_map),
            alloc: &sub_alloc,
            code_alloc: &code_alloc,
            stack_alloc: &structural_arena,
        };
        //execute the transaction
        if do_it {
            exec.execute(&script_code, &temporary_values).unwrap();
        };
    })
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
mod pack;
mod get;
mod lit;
mod _let;
mod try;
mod invoke;
mod switch;

mod script_lit;