use sanskrit_common::model::*;
use sanskrit_common::errors::*;
use model::*;

use sanskrit_common::arena::*;
use byteorder::{ByteOrder};
use sanskrit_common::encoding::EncodingByteOrder;
use externals::{ExecutionInterface, RuntimeExternals};

//enum to indicate if a block had a result or an error as return
#[derive(Copy, Clone, Debug)]
pub enum Continuation<'code> {
    Next,
    Cont(&'code Exp<'code>, usize, bool),
    TryCont(&'code Exp<'code>, &'code Exp<'code>, &'code Exp<'code>, usize),
    //These are called Call instead of cont as these always are isolated frames with no access to parent
    // In theory we could differ the cont as well over an isolated flag, which would allow tail call optim
    RepCall(&'code Exp<'code>, usize, ValueRef, Tag, u8),
    RepTryCall(&'code Exp<'code>, &'code Exp<'code>, &'code Exp<'code>, usize, ValueRef, Tag, u8),
    Rollback,
}

#[derive(Copy, Clone, Debug)]
pub enum Frame<'code> {
    Continuation{
        exp:&'code Exp<'code>,
        pos:usize,
        stack_height:usize,
    },
    Try {
        succ:&'code Exp<'code>,
        fail:&'code Exp<'code>,
        stack_height:usize,
        prev:Option<usize>
    },
    Repeat {
        exp:&'code Exp<'code>,
        stack_height:usize,
        cond_value:ValueRef,
        abort_tag:Tag,
        counter:u8
    }
}

//the context in which the code is interpreted / executed
pub struct ExecutionContext<'transaction, 'code, 'interpreter, 'execution, 'heap> {
    functions: &'code [Ptr<'code,Exp<'code>>],                                                      // all the code
    frames: &'execution mut HeapStack<'interpreter,Frame<'code>>,
    stack: &'execution mut HeapStack<'interpreter, Entry<'transaction> >,                       //The current stack
    alloc: &'transaction VirtualHeapArena<'heap>,
    return_stack: &'execution mut HeapStack<'interpreter, Entry<'transaction> >,
    try_ptr:Option<usize>,
}

//creates a new literal
pub fn create_lit_object<'transaction, 'heap>(data:&[u8], typ:LitDesc, alloc:&'transaction VirtualHeapArena<'heap>) -> Result<Entry<'transaction>> {
    //find out which literal to create
    Ok(match typ {
        LitDesc::Id | LitDesc::Data => Entry { data: alloc.copy_alloc_slice(data)? },
        LitDesc::I8 => Entry { i8:data[0] as i8 },
        LitDesc::U8 => Entry { u8:data[0]},
        LitDesc::I16 => Entry {i16:EncodingByteOrder::read_i16(data)},
        LitDesc::U16 => Entry {u16:EncodingByteOrder::read_u16(data)},
        LitDesc::I32 => Entry {i32:EncodingByteOrder::read_i32(data)},
        LitDesc::U32 => Entry {u32:EncodingByteOrder::read_u32(data)},
        LitDesc::I64 => Entry {i64:EncodingByteOrder::read_i64(data)},
        LitDesc::U64 => Entry {u64:EncodingByteOrder::read_u64(data)},
        LitDesc::I128 => Entry {i128:EncodingByteOrder::read_i128(data)},
        LitDesc::U128 => Entry {u128:EncodingByteOrder::read_u128(data)},
    })
}

impl <'transaction,'code,'interpreter,'execution,'heap> ExecutionInterface<'interpreter, 'transaction, 'heap> for ExecutionContext<'transaction,'code,'interpreter, 'execution,'heap> {
    //helper to get stack elems
    fn get(&self, idx: usize) -> Result<Entry<'transaction>> {
        //calc the pos
        let pos = self.stack.len() - idx - 1;
        //get the elem
        Ok(*self.stack.get(pos)?)
    }

    //helper to get the correct stack (return or normal)
    fn get_stack(&mut self, tail:bool) -> &mut HeapStack<'interpreter, Entry<'transaction> > {
        if tail {
            self.return_stack
        } else {
            self.stack
        }
    }

    fn get_heap(&self) -> &'transaction VirtualHeapArena<'heap>{
        self.alloc
    }

    fn process_entry_slice<R:Sized,F:FnOnce(&[u8]) -> R>(kind:Kind, op1:Entry<'transaction>, proc:F) -> R {
        match kind {
            Kind::I8 => proc(&[unsafe {op1.i8} as u8]),
            Kind::U8 => proc(&[unsafe {op1.u8}]),
            Kind::I16 => {
                let mut input = [0; 2];
                EncodingByteOrder::write_i16(&mut input, unsafe {op1.i16});
                proc(&input)
            },
            Kind::U16 =>  {
                let mut input = [0; 2];
                EncodingByteOrder::write_u16(&mut input, unsafe {op1.u16});
                proc(&input)
            },
            Kind::I32 => {
                let mut input = [0; 4];
                EncodingByteOrder::write_i32(&mut input, unsafe {op1.i32});
                proc(&input)
            },
            Kind::U32 => {
                let mut input = [0; 4];
                EncodingByteOrder::write_u32(&mut input, unsafe {op1.u32});
                proc(&input)
            },
            Kind::I64 => {
                let mut input = [0; 8];
                EncodingByteOrder::write_i64(&mut input, unsafe {op1.i64});
                proc(&input)
            },
            Kind::U64 => {
                let mut input = [0; 8];
                EncodingByteOrder::write_u64(&mut input, unsafe {op1.u64});
                proc(&input)
            },
            Kind::I128 => {
                let mut input = [0; 16];
                EncodingByteOrder::write_i128(&mut input, unsafe {op1.i128});
                proc(&input)
            },
            Kind::U128 => {
                let mut input = [0; 16];
                EncodingByteOrder::write_u128(&mut input, unsafe {op1.u128});
                proc(&input)
            },
            Kind::Data => proc(unsafe {&op1.data})
        }
    }
}

impl<'transaction,'code,'interpreter,'execution,'heap> ExecutionContext<'transaction,'code,'interpreter, 'execution,'heap> {
    //Creates a new Empty context
    pub fn interpret<Ext:RuntimeExternals>(
        functions: &'code [Ptr<'code,Exp<'code>>],
        stack:&'execution mut HeapStack<'interpreter,Entry<'transaction>>,
        frames:&'execution mut HeapStack<'interpreter,Frame<'code>>,
        return_stack:&'execution mut HeapStack<'interpreter,Entry<'transaction>>,
        alloc:&'transaction VirtualHeapArena<'heap>
    ) -> Result<()>{
        //Define some reused types and capabilities
        let context = ExecutionContext {
            functions,
            frames,
            stack,
            alloc,
            return_stack,
            try_ptr:None,
        };
        context.execute_function::<Ext>((functions.len()-1) as u16)
    }

    //TExecutes a function in the current context
    fn execute_function<Ext:RuntimeExternals>(mut self, fun_idx: u16) -> Result<()> {
        //Cost: constant
        //assert params are on Stack for now
        let code = &self.functions[fun_idx as usize];

        self.frames.push(Frame::Continuation {
            exp: &code,
            pos: 0,
            stack_height: 0
        })?;

        if self.execute_exp::<Ext>()? {
            Ok(())
        } else {
            error(||"Transaction was rolled back")
        }
    }

    //Type checks an expression in the current context
    fn execute_exp<Ext:RuntimeExternals>(&mut self) -> Result<bool> {
        'outer: loop {
            match self.frames.pop() {
                Some(Frame::Continuation { exp, mut pos, stack_height }) => {
                    //let Frame { exp, pos, stack_height } = cur;
                    //Cost: relative to vals.len() |opCode will measure seperately| -- sub stack push will be accounted on push
                    //process each opcode
                    'main: while exp.0.len() > pos {
                        pos+=1;
                        let tail = exp.0.len() == pos;

                        match self.execute_op_code::<Ext>(&exp.0[pos-1], tail)? {
                            Continuation::Next => {},
                            Continuation::Cont(n_exp, n_stack_height, isolated) => {
                                if !tail {
                                    //Re-push the current Frame if we still need it later on
                                    self.frames.push(Frame::Continuation { exp, pos, stack_height })?;
                                    //Push the new Frame
                                    self.frames.push(Frame::Continuation { exp:n_exp, pos:0, stack_height:n_stack_height })?;
                                    //Execute the new Frame
                                    continue 'outer;
                                } else {
                                    //Push the new Frame
                                    self.frames.push(Frame::Continuation { exp:n_exp, pos:0, stack_height })?;
                                    //Execute the new Frame
                                    if isolated {
                                        //we no longer need the current - args are on return
                                        break 'main;
                                    } else {
                                        //we still need the current stack - args are on stack
                                        continue 'outer;
                                    }
                                }
                            }
                            Continuation::RepCall(n_exp, n_stack_height, cond_value, abort_tag, counter) => {
                                if !tail {
                                    //Re-push the current Frame if we still need it later on
                                    self.frames.push(Frame::Continuation { exp, pos, stack_height })?;
                                    //Push the new repeat frame
                                    self.frames.push(Frame::Repeat { exp:n_exp, stack_height: n_stack_height, cond_value, abort_tag, counter})?;
                                    //Execute the new repeat frame (params are already on the stack)
                                    continue 'outer;
                                } else {
                                    //Push the new repeat frame
                                    self.frames.push(Frame::Repeat { exp:n_exp, stack_height, cond_value, abort_tag, counter})?;
                                    //Execute the new repeat frame (ensure the old stack is cleaned first -- that is why we use break 'inner)
                                    break 'main;
                                }
                            }
                            Continuation::TryCont(n_exp, succ, fail, n_stack_height) => {
                                if !tail {
                                    //Re-push the current Frame if we still need it later omn
                                    self.frames.push(Frame::Continuation { exp, pos, stack_height })?;
                                    //we only revert to new stack height as the pushed continue frame will revert the rest
                                    self.frames.push(Frame::Try { succ, fail, stack_height:n_stack_height,  prev:self.try_ptr})?;
                                    //Set the try_ptr in case of a rollback
                                    self.try_ptr = Some(self.frames.len());
                                    //Push the new Frame
                                    self.frames.push(Frame::Continuation { exp:n_exp, pos:0, stack_height:n_stack_height })?;
                                    //Execute the new Frame
                                    continue 'outer;
                                } else {
                                    //Push a try Frame
                                    self.frames.push(Frame::Try { succ, fail, stack_height,  prev:self.try_ptr})?;
                                    //Set the try_ptr in case of a rollback
                                    self.try_ptr = Some(self.frames.len());
                                    //Execute the new Frame
                                    self.frames.push(Frame::Continuation { exp:n_exp, pos:0, stack_height:n_stack_height })?;
                                    //Execute the new Frame
                                    // Note: Try executes the try body with tail == false so the args are always on the stack
                                    continue 'outer;
                                }
                            }
                            Continuation::RepTryCall(n_exp, succ, fail, n_stack_height, cond_value, abort_tag, counter) => {
                                if !tail {
                                    //Re-push the current Frame if we still need it later on
                                    self.frames.push(Frame::Continuation { exp, pos, stack_height })?;
                                    //we only revert to new stack height as the pushed continue frame will revert the rest
                                    self.frames.push(Frame::Try { succ, fail, stack_height:n_stack_height,  prev:self.try_ptr})?;
                                    //Set the try_ptr in case of a rollback
                                    self.try_ptr = Some(self.frames.len());
                                    //Push the new repeat frame
                                    self.frames.push(Frame::Repeat { exp:n_exp, stack_height:n_stack_height, cond_value, abort_tag, counter})?;
                                    //Execute the new repeat frame (params are already on the stack)
                                    continue 'outer;
                                } else {
                                    //Push a try Frame
                                    self.frames.push(Frame::Try { succ, fail, stack_height,  prev:self.try_ptr})?;
                                    //Set the try_ptr in case of a rollback
                                    self.try_ptr = Some(self.frames.len());
                                    //Push the new repeat frame
                                    self.frames.push(Frame::Repeat { exp:n_exp, stack_height: n_stack_height, cond_value, abort_tag, counter})?;
                                    //Execute the new Frame
                                    // Note: Try executes the try body with tail == false so the args are always on the stack
                                    continue 'outer;
                                }
                            }
                            Continuation::Rollback => {
                                if !self.execute_rollback()? {
                                    return Ok(false)
                                }
                            }
                        }
                    }

                    //reset the stack
                    self.stack.rewind_to(stack_height)?;

                    //push the returned elems (empties return_stack)
                    self.stack.transfer_from(self.return_stack)?;
                },
                Some(Frame::Try { succ, stack_height, prev, .. }) => {
                    self.frames.push(Frame::Continuation { exp:succ, pos:0, stack_height })?;
                    self.try_ptr = prev;
                },
                Some(Frame::Repeat {exp, stack_height, cond_value:ValueRef(idx), abort_tag, counter}) => {
                    let cond_elem = self.get(idx as usize)?;
                    let tag = unsafe {cond_elem.adt.0};
                    //do we need more repetitions?
                    if tag != abort_tag.0 {
                        //can we do more iterations?
                        if counter != 0 {
                            //Re-push ourself
                            self.frames.push(Frame::Repeat { exp, stack_height, cond_value:ValueRef(idx), abort_tag, counter: counter -1})?;
                            //Push the next iteration
                            self.frames.push(Frame::Continuation { exp, pos: 0, stack_height })?;
                        } else {
                            if !self.execute_rollback()? {
                                return Ok(false)
                            }
                        }
                    }
                }
                None => return Ok(true)
            }
        }
    }

    fn execute_rollback(&mut self) -> Result<bool> {
        match self.try_ptr {
            None => Ok(false),
            Some(try_ptr) => {
                //reset frames
                self.frames.rewind_to(try_ptr)?;
                if let Some(Frame::Try { fail, stack_height, prev, .. }) = self.frames.pop() {
                    //reset the stack
                    self.stack.rewind_to(stack_height)?;
                    //push the failure as continuation
                    self.frames.push(Frame::Continuation { exp:fail, pos:0, stack_height })?;
                    //recover the previous try pointer
                    self.try_ptr = prev;
                } else {
                    unreachable!()
                }
                Ok(true)
            },
        }
    }

    //The heavy lifter that type checks op code
    fn execute_op_code<Ext:RuntimeExternals>(&mut self, code: &'code OpCode, tail:bool) -> Result<Continuation<'code>> {
        //Branch on the opcode type and check it
        match *code {
            OpCode::Void => self.void(tail),
            OpCode::Data(data) => self.lit(&data, tail),
            OpCode::SpecialLit(ref data, desc) => self.special_lit(data,desc, tail),
            OpCode::Let(ref bind) => self.let_(bind, tail),
            OpCode::Unpack(value) => self.unpack(value, tail),
            OpCode::Get(value, field) => self.get_field(value, field, tail),
            OpCode::Switch(value, ref cases) => self.switch(value, cases, tail),
            OpCode::Pack(tag, values) => self.pack(tag, &values, tail),
            OpCode::CreateSig(func,values) => self.create_sig(func, &values, tail),
            OpCode::InvokeSig(func_val, values)  => self.invoke_sig(func_val, &values, tail),
            OpCode::Invoke(func, values) => self.invoke(func, &values, tail),
            OpCode::RepeatedInvoke(func, values, cond_value, abort_tag, max_reps) => self.repeat(func, &values, cond_value, abort_tag, max_reps, tail),
            OpCode::Try(ref code,ref succ, ref fail) => self.try::<Ext>(code, succ, fail, tail),
            OpCode::Rollback => Ok(Continuation::Rollback),
            OpCode::Return(ref vals) => self._return(vals, tail),
            OpCode::And(kind, op1,op2) => self.and(kind,op1,op2, tail),
            OpCode::Or(kind, op1,op2) => self.or(kind, op1,op2, tail),
            OpCode::Xor(kind, op1,op2) => self.xor(kind, op1,op2, tail),
            OpCode::Not(kind, op) => self.not(kind, op, tail),
            OpCode::Id(op) => self.copy(op, tail),
            OpCode::Add(kind, op1,op2) => self.add(kind, op1,op2, tail),
            OpCode::Sub(kind, op1,op2) => self.sub(kind, op1,op2, tail),
            OpCode::Mul(kind, op1,op2) => self.mul(kind, op1,op2, tail),
            OpCode::Div(kind, op1,op2) => self.div(kind, op1,op2, tail),
            OpCode::Eq(kind, op1,op2) => self.eq(kind, op1,op2, tail),
            OpCode::ToData(kind, op) => self.convert_to_data(kind,op, tail),
            OpCode::FromData(kind, op) => self.convert_from_data(kind,op, tail),
            OpCode::Lt(kind, op1,op2) => self.lt(kind, op1,op2, tail),
            OpCode::Gt(kind, op1,op2) => self.gt(kind, op1,op2, tail),
            OpCode::Lte(kind, op1,op2) => self.lte(kind, op1,op2, tail),
            OpCode::Gte(kind, op1,op2) => self.gte(kind, op1,op2, tail),
            OpCode::SysInvoke(id, ref vals) => self.sys_call::<Ext>(id, vals, tail),
            OpCode::TypedSysInvoke(id, kind, ref vals) => self.kinded_sys_call::<Ext>(id, kind, vals, tail),
        }
    }

    fn void(&mut self, tail:bool) -> Result<Continuation<'code>> {
        //push it onto the stack
        let stack = self.get_stack(tail);
        //we can push whatever we want
        stack.push(Entry {u8:0})?;
        Ok(Continuation::Next)
    }

    //creates a literal
    fn lit(&mut self, data: &[u8], tail:bool) -> Result<Continuation<'code>> {
        //Cost: relative to: data.0.len(), + 1 push
        //create the literal
        let data = self.alloc.copy_alloc_slice(data)?;
        //push it onto the stack
        let stack = self.get_stack(tail);
        stack.push(Entry {data})?;
        Ok(Continuation::Next)
    }

    //creates a literal
    fn special_lit(&mut self, data: &[u8], typ: LitDesc, tail:bool) -> Result<Continuation<'code>> {
        //Cost: relative to: data.0.len(), + 1 push
        //create the literal
        let obj = create_lit_object(data, typ, self.alloc)?;
        //push it onto the stack
        let stack = self.get_stack(tail);
        stack.push(obj)?;
        Ok(Continuation::Next)
    }

    //_ as let is keyword
    // process an EXp isolated
    fn let_(&mut self, bind: &'code Exp, _tail:bool) -> Result<Continuation<'code>> {
        //Cost: constant
        //fetch the height
        let stack_height = self.stack.len();
        //execute the block
        Ok(Continuation::Cont(bind, stack_height, false))
    }

    //unpacks an adt
    fn unpack(&mut self, ValueRef(idx): ValueRef, tail:bool) -> Result<Continuation<'code>> {

        //Cost: relative to: elems.len()
        //get the input
        let Adt(_, fields) = unsafe { self.get(idx as usize)?.adt };
        //must be an adt (static guarantee)
        //push each field
        let stack = self.get_stack(tail);
        for e in fields.iter() {
            stack.push(*e)?;
        }
        Ok(Continuation::Next)
    }

    //gets a single field from an adt
    fn get_field(&mut self, ValueRef(idx): ValueRef, field:u8, tail:bool) -> Result<Continuation<'code>> {
        //Cost: constant
        //get the input
        let Adt(_, fields) = unsafe { self.get(idx as usize)?.adt };
        //must be an adt (static guarantee)
        //push the correct field
        let stack = self.get_stack(tail);
        stack.push( fields[field as usize])?;
        Ok(Continuation::Next)
    }

    //branch based on constructor
    fn switch(&mut self, ValueRef(idx): ValueRef, cases: &'code [Ptr<'code,Exp<'code>>], _tail:bool) -> Result<Continuation<'code>> {
        //Cost: relative to: elems.len()
        //get the input
        let Adt(tag, fields) = unsafe { self.get(idx as usize)?.adt };
        //must be an adt (static guarantee)
        //capture the height
        let stack_height = self.stack.len();
        //push the fields
        for e in fields.iter() {
            self.stack.push(*e)?;
        }

        //execute the right branch
        Ok(Continuation::Cont(&cases[tag as usize], stack_height, false))
    }

    //packs an adt
    fn pack(&mut self, Tag(tag): Tag, values: &[ValueRef], tail:bool) -> Result<Continuation<'code>> {
        //Cost: relative to: values.len()
        //fetch the inputs
        let mut fields = self.alloc.slice_builder(values.len())?;
        for ValueRef(idx) in values {
            let elem = self.get(*idx as usize)?;
            fields.push(elem);
        }
        //produce an adt with the fields as args
        let stack = self.get_stack(tail);
        stack.push(Entry{adt:Adt(tag, fields.finish())})?;
        Ok(Continuation::Next)
    }

    //packs an adt
    fn create_sig(&mut self, func:u16, values: &[ValueRef], tail:bool) -> Result<Continuation<'code>> {
        //Cost: relative to: values.len()
        //fetch the inputs
        let mut fields = self.alloc.slice_builder(values.len())?;
        for ValueRef(idx) in values {
            let elem = self.get(*idx as usize)?;
            fields.push(elem);
        }
        //produce an adt with the fields as args
        let stack = self.get_stack(tail);
        stack.push(Entry{func:Func(func, fields.finish())})?;
        Ok(Continuation::Next)
    }

    //call a sig function
    fn invoke_sig(&mut self, ValueRef(fun_val): ValueRef, values: &[ValueRef], tail:bool) -> Result<Continuation<'code>> {
        //get the target
        let Func(index, captures) = unsafe {self.get(fun_val as usize)?.func};
        //must be a function pointer (static guarantee)
        //Cost: relative to: values.len()
        //get the code
        let fun_code: &Exp = &self.functions[index as usize];
        //fetch the height
        let stack_height = self.stack.len();
        //push the captured arguments
        assert!(values.len()+captures.len() <= u16::max_value() as usize);
        for elem in captures.iter() {
            self.get_stack(tail).push(*elem)?;
        }
        //push the provided arguments
        for (i,ValueRef(idx)) in values.iter().enumerate() {
            let elem = if tail {
                self.get(*idx as usize)?
            } else {
                self.get(*idx as usize+(i+captures.len()))? //i+captures.len() counteracts the already pushed elements
            };
            self.get_stack(tail).push(elem)?;

        }
        //Execute the function
        Ok(Continuation::Cont(fun_code, stack_height, true))
    }

    //call a function
    fn invoke(&mut self, fun_idx: u16, values: &[ValueRef], tail:bool) -> Result<Continuation<'code>> {
        //Cost: relative to: values.len()
        //Non-Native
        //get the code
        let fun_code: &Exp = &self.functions[fun_idx as usize];
        //fetch the height
        let stack_height = self.stack.len();
        //push the arguments
        assert!(values.len() <= u16::max_value() as usize);
        for (i,ValueRef(idx)) in values.iter().enumerate() {
            let elem = if tail {
                self.get(*idx as usize)?
            } else {
                self.get(*idx as usize+i)? //i counteracts the already pushed elements
            };
            self.get_stack(tail).push(elem)?;
        }
        //Execute the function
        Ok(Continuation::Cont(fun_code, stack_height, true))
    }

    fn repeat(&mut self, fun_idx: u16, values: &[ValueRef], cond_value:ValueRef, abort_tag:Tag, max_reps:u8, tail:bool) -> Result<Continuation<'code>> {
        //Cost: relative to: values.len()
        //Non-Native
        //get the code
        let fun_code: &Exp = &self.functions[fun_idx as usize];
        //fetch the height
        let stack_height = self.stack.len();
        //push the arguments
        assert!(values.len() <= u16::max_value() as usize);
        for (i,ValueRef(idx)) in values.iter().enumerate() {
            let elem = if tail {
                self.get(*idx as usize)?
            } else {
                self.get(*idx as usize+i)? //i counteracts the already pushed elements
            };
            self.get_stack(tail).push(elem)?;
        }
        //Execute the function
        Ok(Continuation::RepCall(fun_code, stack_height, cond_value, abort_tag, max_reps))
    }

    fn try<Ext:RuntimeExternals>(&mut self,  try:&'code OpCode, succ: &'code Exp, fail: &'code Exp, _tail:bool) -> Result<Continuation<'code>> {
        //fetch the height
        let stack_height = self.stack.len();
        //Execute the try code
        match self.execute_op_code::<Ext>(try, false)? {
            //it succeeded so continue with the success case
            Continuation::Next => Ok(Continuation::Cont(succ, stack_height, false)), //This is nice as it allows to make adds & muls & ... more efficently even with try
            //it failed so continue with the failure case
            Continuation::Rollback => Ok(Continuation::Cont(fail, stack_height, false)), //This is nice as it allows to make adds & muls & ... more efficently even with try
            //it is a nested block so remember the try
            Continuation::Cont(n_exp,n_stack_height, _isolated) => {
                assert_eq!(n_stack_height,stack_height);
                Ok(Continuation::TryCont(n_exp,succ,fail,stack_height))
            },
            Continuation::RepCall(n_exp, n_stack_height, cond_value, abort_tag, max_reps) => {
                assert_eq!(n_stack_height,stack_height);
                Ok(Continuation::RepTryCall(n_exp, succ, fail, stack_height, cond_value, abort_tag, max_reps))
            }

            //Not yet supported. Needs to be supported if we have inlining optimization
            Continuation::TryCont(_,_,_,_) => error(||"Directly nested tries are not yet supports"),
            Continuation::RepTryCall(_, _, _, _, _, _, _) => error(||"Directly nested tries are not yet supports"),
        }
    }


    fn and(&mut self, kind:Kind, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val1 as usize)?;
        let op2 = self.get(val2 as usize)?;
        let res =match kind {
            Kind::I8 => Entry{i8: unsafe {op1.i8 & op2.i8}},
            Kind::U8 => Entry{u8: unsafe {op1.u8 & op2.u8}},
            Kind::I16 => Entry{i16: unsafe {op1.i16 & op2.i16}},
            Kind::U16 => Entry{u16: unsafe {op1.u16 & op2.u16}},
            Kind::I32 => Entry{i32: unsafe {op1.i32 & op2.i32}},
            Kind::U32 => Entry{u32: unsafe {op1.u32 & op2.u32}},
            Kind::I64 => Entry{i64: unsafe {op1.i64 & op2.i64}},
            Kind::U64 => Entry{u64: unsafe {op1.u64 & op2.u64}},
            Kind::I128 => Entry{i128: unsafe {op1.i128 & op2.i128}},
            Kind::U128 => Entry{u128: unsafe {op1.u128 & op2.u128}},
            Kind::Data => {
                let data1 = unsafe {op1.data};
                let data2 = unsafe {op2.data};
                let mut builder = self.alloc.slice_builder(data1.len())?;
                for i in 0..data1.len() {
                    builder.push(data1[i] & data2[i]);
                }
                Entry{ data: builder.finish() }
            }
        };
        self.get_stack(tail).push(res)?;
        Ok(Continuation::Next)
    }

    fn or(&mut self, kind:Kind, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val1 as usize)?;
        let op2 = self.get(val2 as usize)?;
        let res =match kind {
            Kind::I8 => Entry{i8: unsafe {op1.i8 | op2.i8}},
            Kind::U8 => Entry{u8: unsafe {op1.u8 | op2.u8}},
            Kind::I16 => Entry{i16: unsafe {op1.i16 | op2.i16}},
            Kind::U16 => Entry{u16: unsafe {op1.u16 | op2.u16}},
            Kind::I32 => Entry{i32: unsafe {op1.i32 | op2.i32}},
            Kind::U32 => Entry{u32: unsafe {op1.u32 | op2.u32}},
            Kind::I64 => Entry{i64: unsafe {op1.i64 | op2.i64}},
            Kind::U64 => Entry{u64: unsafe {op1.u64 | op2.u64}},
            Kind::I128 => Entry{i128: unsafe {op1.i128 | op2.i128}},
            Kind::U128 => Entry{u128: unsafe {op1.u128 | op2.u128}},
            Kind::Data => {
                let data1 = unsafe {op1.data};
                let data2 = unsafe {op2.data};
                let mut builder = self.alloc.slice_builder(data1.len())?;
                for i in 0..data1.len() {
                    builder.push(data1[i] | data2[i]);
                }
                Entry{ data: builder.finish() }
            }
        };
        self.get_stack(tail).push(res)?;
        Ok(Continuation::Next)
    }

    fn xor(&mut self, kind:Kind, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val1 as usize)?;
        let op2 = self.get(val2 as usize)?;
        let res = match kind {
            Kind::I8 => Entry{i8: unsafe {op1.i8 ^ op2.i8}},
            Kind::U8 => Entry{u8: unsafe {op1.u8 ^ op2.u8}},
            Kind::I16 => Entry{i16: unsafe {op1.i16 ^ op2.i16}},
            Kind::U16 => Entry{u16: unsafe {op1.u16 ^ op2.u16}},
            Kind::I32 => Entry{i32: unsafe {op1.i32 ^ op2.i32}},
            Kind::U32 => Entry{u32: unsafe {op1.u32 ^ op2.u32}},
            Kind::I64 => Entry{i64: unsafe {op1.i64 ^ op2.i64}},
            Kind::U64 => Entry{u64: unsafe {op1.u64 ^ op2.u64}},
            Kind::I128 => Entry{i128: unsafe {op1.i128 ^ op2.i128}},
            Kind::U128 => Entry{u128: unsafe {op1.u128 ^ op2.u128}},
            Kind::Data => {
                let data1 = unsafe {op1.data};
                let data2 = unsafe {op2.data};
                let mut builder = self.alloc.slice_builder(data1.len())?;
                for i in 0..data1.len() {
                    builder.push(data1[i] ^ data2[i]);
                }
                Entry{ data: builder.finish() }
            }
        };
        self.get_stack(tail).push(res)?;
        Ok(Continuation::Next)
    }

    fn not(&mut self, kind:Kind, ValueRef(val):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val as usize)?;
        let res = match kind {
            Kind::I8 => Entry{i8: unsafe {!op1.i8}},
            Kind::U8 => Entry{u8: unsafe {!op1.u8}},
            Kind::I16 => Entry{i16: unsafe {!op1.i16}},
            Kind::U16 => Entry{u16: unsafe {!op1.u16}},
            Kind::I32 => Entry{i32: unsafe {!op1.i32}},
            Kind::U32 => Entry{u32: unsafe {!op1.u32}},
            Kind::I64 => Entry{i64: unsafe {!op1.i64}},
            Kind::U64 => Entry{u64: unsafe {!op1.u64}},
            Kind::I128 => Entry{i128: unsafe {!op1.i128}},
            Kind::U128 => Entry{u128: unsafe {!op1.u128}},
            Kind::Data => {
                let data1 = unsafe {op1.data};
                let mut builder = self.alloc.slice_builder(data1.len())?;
                for i in 0..data1.len() {
                    builder.push(!data1[i]);
                }
                Entry{ data: builder.finish() }
            }
        };
        self.get_stack(tail).push(res)?;
        Ok(Continuation::Next)
    }

    fn copy(&mut self, ValueRef(val):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        let elem = self.get(val as usize)?;
        self.get_stack(tail).push(elem)?;
        Ok(Continuation::Next)
    }

    fn _return(&mut self,  vals:&[ValueRef], tail:bool) -> Result<Continuation<'code>> {
        for (idx,val) in vals.iter().enumerate(){
            if tail {
                self.return_stack.push(self.get(val.0 as usize)?)?
            } else {
                self.stack.push(self.get(val.0 as usize+idx)?)?;
            };
        }

        Ok(Continuation::Next)
    }

    //does an addition (a checked one, returns None in case of Over/under flow)
    fn add(&mut self, kind:Kind, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val1 as usize)?;
        let op2 = self.get(val2 as usize)?;

        let res = match kind {
            Kind::I8 => unsafe {op1.i8.checked_add(op2.i8)}.map(|r| Entry{i8: r}),
            Kind::U8 => unsafe {op1.u8.checked_add(op2.u8)}.map(|r| Entry{u8: r}),
            Kind::I16 => unsafe {op1.i16.checked_add(op2.i16)}.map(|r| Entry{i16: r}),
            Kind::U16 => unsafe {op1.u16.checked_add(op2.u16)}.map(|r| Entry{u16: r}),
            Kind::I32 => unsafe {op1.i32.checked_add(op2.i32)}.map(|r| Entry{i32: r}),
            Kind::U32 => unsafe {op1.u32.checked_add(op2.u32)}.map(|r| Entry{u32: r}),
            Kind::I64 => unsafe {op1.i64.checked_add(op2.i64)}.map(|r| Entry{i64: r}),
            Kind::U64 => unsafe {op1.u64.checked_add(op2.u64)}.map(|r| Entry{u64: r}),
            Kind::I128 => unsafe {op1.i128.checked_add(op2.i128)}.map(|r| Entry{i128: r}),
            Kind::U128 => unsafe {op1.u128.checked_add(op2.u128)}.map(|r| Entry{u128: r}),
            Kind::Data => unreachable!(),
        };

        match res {
            None => Ok(Continuation::Rollback),
            Some(r) => {
                self.get_stack(tail).push(r)?;
                Ok(Continuation::Next)
            }
        }
    }

    //does a substraction (a checked one, returns None in case of Over/under flow)
    fn sub(&mut self, kind:Kind, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val1 as usize)?;
        let op2 = self.get(val2 as usize)?;
        let res =match kind {
            Kind::I8 => unsafe {op1.i8.checked_sub(op2.i8)}.map(|r| Entry{i8: r}),
            Kind::U8 => unsafe {op1.u8.checked_sub(op2.u8)}.map(|r| Entry{u8: r}),
            Kind::I16 => unsafe {op1.i16.checked_sub(op2.i16)}.map(|r| Entry{i16: r}),
            Kind::U16 => unsafe {op1.u16.checked_sub(op2.u16)}.map(|r| Entry{u16: r}),
            Kind::I32 => unsafe {op1.i32.checked_sub(op2.i32)}.map(|r| Entry{i32: r}),
            Kind::U32 => unsafe {op1.u32.checked_sub(op2.u32)}.map(|r| Entry{u32: r}),
            Kind::I64 => unsafe {op1.i64.checked_sub(op2.i64)}.map(|r| Entry{i64: r}),
            Kind::U64 => unsafe {op1.u64.checked_sub(op2.u64)}.map(|r| Entry{u64: r}),
            Kind::I128 => unsafe {op1.i128.checked_sub(op2.i128)}.map(|r| Entry{i128: r}),
            Kind::U128 => unsafe {op1.u128.checked_sub(op2.u128)}.map(|r| Entry{u128: r}),
            Kind::Data => unreachable!(),
        };

        match res {
            None => Ok(Continuation::Rollback),
            Some(r) => {
                self.get_stack(tail).push(r)?;
                Ok(Continuation::Next)
            }
        }
    }

    //does a multiplication (a checked one, returns None in case of Over/under flow)
    fn mul(&mut self, kind:Kind, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val1 as usize)?;
        let op2 = self.get(val2 as usize)?;
        let res =match kind {
            Kind::I8 => unsafe {op1.i8.checked_mul(op2.i8)}.map(|r| Entry{i8: r}),
            Kind::U8 => unsafe {op1.u8.checked_mul(op2.u8)}.map(|r| Entry{u8: r}),
            Kind::I16 => unsafe {op1.i16.checked_mul(op2.i16)}.map(|r| Entry{i16: r}),
            Kind::U16 => unsafe {op1.u16.checked_mul(op2.u16)}.map(|r| Entry{u16: r}),
            Kind::I32 => unsafe {op1.i32.checked_mul(op2.i32)}.map(|r| Entry{i32: r}),
            Kind::U32 => unsafe {op1.u32.checked_mul(op2.u32)}.map(|r| Entry{u32: r}),
            Kind::I64 => unsafe {op1.i64.checked_mul(op2.i64)}.map(|r| Entry{i64: r}),
            Kind::U64 => unsafe {op1.u64.checked_mul(op2.u64)}.map(|r| Entry{u64: r}),
            Kind::I128 => unsafe {op1.i128.checked_mul(op2.i128)}.map(|r| Entry{i128: r}),
            Kind::U128 => unsafe {op1.u128.checked_mul(op2.u128)}.map(|r| Entry{u128: r}),
            Kind::Data => unreachable!(),
        };

        match res {
            None => Ok(Continuation::Rollback),
            Some(r) => {
                self.get_stack(tail).push(r)?;
                Ok(Continuation::Next)
            }
        }
    }

    //does a division (a checked one, returns None in case of Over/under flow or division by 0)
    fn div(&mut self, kind:Kind, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val1 as usize)?;
        let op2 = self.get(val2 as usize)?;
        let res =match kind {
            Kind::I8 => unsafe {op1.i8.checked_div(op2.i8)}.map(|r| Entry{i8: r}),
            Kind::U8 => unsafe {op1.u8.checked_div(op2.u8)}.map(|r| Entry{u8: r}),
            Kind::I16 => unsafe {op1.i16.checked_div(op2.i16)}.map(|r| Entry{i16: r}),
            Kind::U16 => unsafe {op1.u16.checked_div(op2.u16)}.map(|r| Entry{u16: r}),
            Kind::I32 => unsafe {op1.i32.checked_div(op2.i32)}.map(|r| Entry{i32: r}),
            Kind::U32 => unsafe {op1.u32.checked_div(op2.u32)}.map(|r| Entry{u32: r}),
            Kind::I64 => unsafe {op1.i64.checked_div(op2.i64)}.map(|r| Entry{i64: r}),
            Kind::U64 => unsafe {op1.u64.checked_div(op2.u64)}.map(|r| Entry{u64: r}),
            Kind::I128 => unsafe {op1.i128.checked_div(op2.i128)}.map(|r| Entry{i128: r}),
            Kind::U128 => unsafe {op1.u128.checked_div(op2.u128)}.map(|r| Entry{u128: r}),
            Kind::Data => unreachable!(),
        };

        match res {
            None => Ok(Continuation::Rollback),
            Some(r) => {
                self.get_stack(tail).push(r)?;
                Ok(Continuation::Next)
            }
        }
    }

    //compares the inputs for equality
    fn eq(&mut self, kind:Kind, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val1 as usize)?;
        let op2 = self.get(val2 as usize)?;
        self.get_stack(tail).push(match kind {
            Kind::I8 => Entry{ adt: Adt(unsafe {op1.i8 == op2.i8} as u8, SlicePtr::empty())},
            Kind::U8 => Entry{ adt: Adt(unsafe {op1.u8 == op2.u8} as u8, SlicePtr::empty())},
            Kind::I16 => Entry{ adt: Adt(unsafe {op1.i16 == op2.i16} as u8, SlicePtr::empty())},
            Kind::U16 => Entry{ adt: Adt(unsafe {op1.u16 == op2.u16} as u8, SlicePtr::empty())},
            Kind::I32 => Entry{ adt: Adt(unsafe {op1.i32 == op2.i32} as u8, SlicePtr::empty())},
            Kind::U32 => Entry{ adt: Adt(unsafe {op1.u32 == op2.u32} as u8, SlicePtr::empty())},
            Kind::I64 => Entry{ adt: Adt(unsafe {op1.i64 == op2.i64} as u8, SlicePtr::empty())},
            Kind::U64 => Entry{ adt: Adt(unsafe {op1.u64 == op2.u64} as u8, SlicePtr::empty())},
            Kind::I128 => Entry{ adt: Adt(unsafe {op1.i128 == op2.i128} as u8, SlicePtr::empty())},
            Kind::U128 => Entry{ adt: Adt(unsafe {op1.u128 == op2.u128} as u8, SlicePtr::empty())},
            Kind::Data => Entry{ adt: Adt(unsafe {op1.data == op2.data} as u8, SlicePtr::empty())},
        })?;
        Ok(Continuation::Next)
    }

    fn sys_call<Ext:RuntimeExternals>(&mut self, id:u8, vals:&[ValueRef], tail:bool) -> Result<Continuation<'code>>  {
        Ext::system_call(self,id, vals, tail)?;
        Ok(Continuation::Next)
    }

    fn kinded_sys_call<Ext:RuntimeExternals>(&mut self, id:u8, kind:Kind, vals:&[ValueRef], tail:bool) -> Result<Continuation<'code>>  {
        Ext::typed_system_call(self,id,kind, vals, tail)?;
        Ok(Continuation::Next)
    }

    //compares the inputs for less than
    fn lt(&mut self, kind:Kind, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val1 as usize)?;
        let op2 = self.get(val2 as usize)?;
        self.get_stack(tail).push(match kind {
            Kind::I8 => Entry{adt: Adt(unsafe {op1.i8 < op2.i8} as u8, SlicePtr::empty())},
            Kind::U8 => Entry{adt: Adt(unsafe {op1.u8 < op2.u8} as u8, SlicePtr::empty())},
            Kind::I16 => Entry{adt: Adt(unsafe {op1.i16 < op2.i16} as u8, SlicePtr::empty())},
            Kind::U16 => Entry{adt: Adt(unsafe {op1.u16 < op2.u16} as u8, SlicePtr::empty())},
            Kind::I32 => Entry{adt: Adt(unsafe {op1.i32 < op2.i32} as u8, SlicePtr::empty())},
            Kind::U32 => Entry{adt: Adt(unsafe {op1.u32 < op2.u32} as u8, SlicePtr::empty())},
            Kind::I64 => Entry{adt: Adt(unsafe {op1.i64 < op2.i64} as u8, SlicePtr::empty())},
            Kind::U64 => Entry{adt: Adt(unsafe {op1.u64 < op2.u64} as u8, SlicePtr::empty())},
            Kind::I128 => Entry{adt: Adt(unsafe {op1.i128 < op2.i128} as u8, SlicePtr::empty())},
            Kind::U128 => Entry{adt: Adt(unsafe {op1.u128 < op2.u128} as u8, SlicePtr::empty())},
            Kind::Data => unreachable!(),
        })?;
        Ok(Continuation::Next)
    }

    //compares the inputs for greater than
    fn gt(&mut self, kind:Kind, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val1 as usize)?;
        let op2 = self.get(val2 as usize)?;
        self.get_stack(tail).push(match kind {
            Kind::I8 => Entry{adt: Adt(unsafe {op1.i8 > op2.i8} as u8, SlicePtr::empty())},
            Kind::U8 => Entry{adt: Adt(unsafe {op1.u8 > op2.u8} as u8, SlicePtr::empty())},
            Kind::I16 => Entry{adt: Adt(unsafe {op1.i16 > op2.i16} as u8, SlicePtr::empty())},
            Kind::U16 => Entry{adt: Adt(unsafe {op1.u16 > op2.u16} as u8, SlicePtr::empty())},
            Kind::I32 => Entry{adt: Adt(unsafe {op1.i32 > op2.i32} as u8, SlicePtr::empty())},
            Kind::U32 => Entry{adt: Adt(unsafe {op1.u32 > op2.u32} as u8, SlicePtr::empty())},
            Kind::I64 => Entry{adt: Adt(unsafe {op1.i64 > op2.i64} as u8, SlicePtr::empty())},
            Kind::U64 => Entry{adt: Adt(unsafe {op1.u64 > op2.u64} as u8, SlicePtr::empty())},
            Kind::I128 => Entry{adt: Adt(unsafe {op1.i128 > op2.i128} as u8, SlicePtr::empty())},
            Kind::U128 => Entry{adt: Adt(unsafe {op1.u128 > op2.u128} as u8, SlicePtr::empty())},
            Kind::Data => unreachable!(),
        })?;
        Ok(Continuation::Next)
    }

    //compares the inputs for less than or equal
    fn lte(&mut self, kind:Kind, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val1 as usize)?;
        let op2 = self.get(val2 as usize)?;
        self.get_stack(tail).push(match kind {
            Kind::I8 => Entry{adt: Adt(unsafe {op1.i8 <= op2.i8} as u8, SlicePtr::empty())},
            Kind::U8 => Entry{adt: Adt(unsafe {op1.u8 <= op2.u8} as u8, SlicePtr::empty())},
            Kind::I16 => Entry{adt: Adt(unsafe {op1.i16 <= op2.i16} as u8, SlicePtr::empty())},
            Kind::U16 => Entry{adt: Adt(unsafe {op1.u16 <= op2.u16} as u8, SlicePtr::empty())},
            Kind::I32 => Entry{adt: Adt(unsafe {op1.i32 <= op2.i32} as u8, SlicePtr::empty())},
            Kind::U32 => Entry{adt: Adt(unsafe {op1.u32 <= op2.u32} as u8, SlicePtr::empty())},
            Kind::I64 => Entry{adt: Adt(unsafe {op1.i64 <= op2.i64} as u8, SlicePtr::empty())},
            Kind::U64 => Entry{adt: Adt(unsafe {op1.u64 <= op2.u64} as u8, SlicePtr::empty())},
            Kind::I128 => Entry{adt: Adt(unsafe {op1.i128 <= op2.i128} as u8, SlicePtr::empty())},
            Kind::U128 => Entry{adt: Adt(unsafe {op1.u128 <= op2.u128} as u8, SlicePtr::empty())},
            Kind::Data => unreachable!(),
        })?;
        Ok(Continuation::Next)
    }

    //compares the inputs for greater than or equal
    fn gte(&mut self, kind:Kind, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val1 as usize)?;
        let op2 = self.get(val2 as usize)?;
        self.get_stack(tail).push(match kind {
            Kind::I8 => Entry { adt: Adt(unsafe { op1.i8 >= op2.i8 } as u8, SlicePtr::empty())},
            Kind::U8 => Entry { adt: Adt(unsafe { op1.u8 >= op2.u8 } as u8, SlicePtr::empty())},
            Kind::I16 => Entry { adt: Adt(unsafe { op1.i16 >= op2.i16 } as u8, SlicePtr::empty())},
            Kind::U16 => Entry { adt: Adt(unsafe { op1.u16 >= op2.u16 } as u8, SlicePtr::empty())},
            Kind::I32 => Entry { adt: Adt(unsafe { op1.i32 >= op2.i32 } as u8, SlicePtr::empty())},
            Kind::U32 => Entry { adt: Adt(unsafe { op1.u32 >= op2.u32 } as u8, SlicePtr::empty())},
            Kind::I64 => Entry { adt: Adt(unsafe { op1.i64 >= op2.i64 } as u8, SlicePtr::empty())},
            Kind::U64 => Entry { adt: Adt(unsafe { op1.u64 >= op2.u64 } as u8, SlicePtr::empty())},
            Kind::I128 => Entry { adt: Adt(unsafe { op1.i128 >= op2.i128 } as u8, SlicePtr::empty())},
            Kind::U128 => Entry { adt: Adt(unsafe { op1.u128 >= op2.u128 } as u8, SlicePtr::empty())},
            Kind::Data => unreachable!(),
        })?;
        Ok(Continuation::Next)
    }

    fn entry_to_data(&mut self, kind:Kind, op1:Entry<'transaction>) -> Result<SlicePtr<'transaction, u8>> {
        Self::process_entry_slice(kind,op1, |s| self.alloc.copy_alloc_slice(s))
    }

    fn data_to_entry(&mut self, kind:Kind, op1:SlicePtr<'transaction, u8>) -> Entry<'transaction> {
        match kind {
            Kind::I8 => Entry{i8:op1[0] as i8},
            Kind::U8 => Entry{u8:op1[0]},
            Kind::I16 => Entry{i16:EncodingByteOrder::read_i16(&op1)},
            Kind::U16 => Entry{u16:EncodingByteOrder::read_u16(&op1)},
            Kind::I32 => Entry{i32:EncodingByteOrder::read_i32(&op1)},
            Kind::U32 => Entry{u32:EncodingByteOrder::read_u32(&op1)},
            Kind::I64 => Entry{i64:EncodingByteOrder::read_i64(&op1)},
            Kind::U64 => Entry{u64:EncodingByteOrder::read_u64(&op1)},
            Kind::I128 => Entry{i128:EncodingByteOrder::read_i128(&op1)},
            Kind::U128 => Entry{u128:EncodingByteOrder::read_u128(&op1)},
            Kind::Data => Entry{data:op1},
        }
    }

    //converts numeric input to data
    //uses byteorder crate for conversion where not trivial
    // conversion is little endian
    fn convert_to_data(&mut self, kind:Kind, ValueRef(val):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val as usize)?;
        let res = Entry{ data:self.entry_to_data(kind,op1)?};
        self.get_stack(tail).push(res)?;
        Ok(Continuation::Next)
    }

    //converts dat input to numerics
    //uses byteorder crate for conversion where not trivial
    // conversion is little endian
    fn convert_from_data(&mut self, kind:Kind, ValueRef(val):ValueRef, tail:bool) -> Result<Continuation<'code>> {
        let op1 = self.get(val as usize)?;
        let res = self.data_to_entry(kind,unsafe{op1.data});
        self.get_stack(tail).push(res)?;
        Ok(Continuation::Next)
    }
}
