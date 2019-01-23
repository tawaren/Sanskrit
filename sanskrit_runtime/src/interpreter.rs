use alloc::prelude::*;
use sanskrit_common::model::*;
use sanskrit_common::errors::*;
use model::*;

use byteorder::{LittleEndian, ByteOrder};
use num_traits::ToPrimitive;
use blake2_rfc::blake2b::{Blake2b};
use script_interpreter::unique_hash;
use script_interpreter::UniqueDomain;
use ContextEnvironment;
use sanskrit_common::arena::*;

//enum to indicate if a block had a result or an error as return
#[derive(Copy, Clone, Debug)]
pub enum Continuation<'code> {
    Next,
    Throw(Error),
    Cont(&'code Exp<'code>, usize),
    Try(&'code Exp<'code>, &'code [(Error, Ptr<'code,Exp<'code>>)], usize)

}

#[derive(Copy, Clone, Debug)]
pub enum Frame<'code> {
    Exp{exp:&'code Exp<'code>, pos:usize, stack_height:usize},
    Catch{catches:&'code [(Error, Ptr<'code,Exp<'code>>)], stack_height:usize},
    Throw(Error),
}

//the context in which the code is interpreted / executed
pub struct ExecutionContext<'script,'code, 'interpreter, 'execution, 'heap> {
    env:ContextEnvironment,                                             //Environment with txt and block infos
    functions: &'code [Ptr<'code,Exp<'code>>],                          // all the code
    frames: &'execution mut HeapStack<'interpreter,Frame<'code>>,
    stack: &'execution mut HeapStack<'interpreter,Ptr<'script ,Object<'script>>>,       //The current stack
    alloc: &'script VirtualHeapArena<'heap>,
    temporary_values: &'interpreter HeapArena<'heap>,                   // helper to spare allocations, todo: use temporary

}

//creates a new literal
pub fn create_lit_object<'script, 'heap>(data:&[u8], typ:LitDesc, alloc:&'script VirtualHeapArena<'heap>) -> Result<Ptr<'script,Object<'script>>> {
    //find out which literal to create
    alloc.alloc(match typ {
        LitDesc::Ref | LitDesc::Data => Object::Data(alloc.copy_alloc_slice(data)?),
        LitDesc::I8 => Object::I8(data[0] as i8),
        LitDesc::U8 => Object::U8(data[0]),
        LitDesc::I16 => Object::I16(LittleEndian::read_i16(data)),
        LitDesc::U16 => Object::U16(LittleEndian::read_u16(data)),
        LitDesc::I32 => Object::I32(LittleEndian::read_i32(data)),
        LitDesc::U32 => Object::U32(LittleEndian::read_u32(data)),
        LitDesc::I64 => Object::I64(LittleEndian::read_i64(data)),
        LitDesc::U64 => Object::U64(LittleEndian::read_u64(data)),
        LitDesc::I128 => Object::I128(LittleEndian::read_i128(data)),
        LitDesc::U128 => Object::U128(LittleEndian::read_u128(data)),
    })
}

//Helper to hash objects structurally
//2 Bytes encode length of Data or Num of fields
fn object_hash(obj:&Object, context: &mut Blake2b) {
    //cost: relative to: Object size
    match *obj {
        Object::I8(data) => {
            let mut input = [0; 3];
            LittleEndian::write_u16(&mut input[0..], 1);
            input[2] = data as u8;
            context.update(&input);
        },
        Object::U8(data) => {
            let mut input = [0; 3];
            LittleEndian::write_u16(&mut input[0..], 1);
            input[2] = data;
            context.update(&input);
        },
        Object::I16(data) => {
            let mut input = [0; 4];
            LittleEndian::write_u16(&mut input[0..], 2);
            LittleEndian::write_i16(&mut input[2..], data);
            context.update(&input);
        },
        Object::U16(data) => {
            let mut input = [0; 4];
            LittleEndian::write_u16(&mut input[0..], 2);
            LittleEndian::write_u16(&mut input[2..], data);
            context.update(&input);
        },
        Object::I32(data) => {
            let mut input = [0; 6];
            LittleEndian::write_u16(&mut input[0..], 4);
            LittleEndian::write_i32(&mut input[2..], data);
            context.update(&input);
        },
        Object::U32(data) => {
            let mut input = [0; 6];
            LittleEndian::write_u16(&mut input[0..], 4);
            LittleEndian::write_u32(&mut input[2..], data);
            context.update(&input);
        },
        Object::I64(data) => {
            let mut input = [0; 10];
            LittleEndian::write_u16(&mut input[0..], 8);
            LittleEndian::write_i64(&mut input[2..], data);
            context.update(&input);
        },
        Object::U64(data) | Object::Context(data) => {
            let mut input = [0; 10];
            LittleEndian::write_u16(&mut input[0..], 8);
            LittleEndian::write_u64(&mut input[2..], data);
            context.update(&input);
        },
        Object::I128(data) => {
            let mut input = [0; 18];
            LittleEndian::write_u16(&mut input[0..], 16);
            LittleEndian::write_i128(&mut input[2..], data);
            context.update(&input);
        },
        Object::U128(data) => {
            let mut input = [0; 18];
            LittleEndian::write_u16(&mut input[0..], 16);
            LittleEndian::write_u128(&mut input[2..], data);
            context.update(&input);
        },
        Object::Data(data) => {
            let mut prefix = [0; 2];
            LittleEndian::write_u16(&mut prefix, data.len() as u16);
            context.update(&prefix);
            context.update(&data);
        },
        Object::Adt(tag, nested) => {
            let mut prefix = [0; 3];
            LittleEndian::write_u16(&mut prefix, nested.len() as u16);
            prefix[2] = tag;
            context.update(&prefix);
            for d in nested.iter() {
                object_hash(d, context)
            }
        },
    };
}

impl<'script,'code,'interpreter,'execution,'heap> ExecutionContext<'script,'code,'interpreter, 'execution,'heap> {
    //Creates a new Empty context
    pub fn interpret(env:ContextEnvironment, functions: &'code [Ptr<'code,Exp<'code>>], stack:&'execution mut HeapStack<'interpreter,Ptr<'script, Object<'script>>>, frames:&'execution mut HeapStack<'interpreter,Frame<'code>>, alloc:&'script VirtualHeapArena<'heap>, temporary_values:&'interpreter HeapArena<'heap>) -> Result<()>{
        //Define some reused types and capabilities
        let context = ExecutionContext {
            env,
            functions,
            frames,
            stack,
            alloc,
            temporary_values
        };
        context.execute_function(0)
    }

    //TExecutes a function in the current context
    fn execute_function(mut self, fun_idx: u16) -> Result<()> {
        //Cost: constant
        //assert params are on Stack for now
        let code = &self.functions[fun_idx as usize];
        self.frames.push(Frame::Exp {
            exp: &code,
            pos: 0,
            stack_height: 0
        })?;

        if self.execute_exp()? {
            Ok(())
        } else {
            interpreter_error()
        }
    }

    //Type checks an expression in the current context
    fn execute_exp(&mut self) -> Result<bool> {
        'outer: loop {
            match self.frames.pop() {
                None => return Ok(true),
                Some(Frame::Catch {..}) => { },
                Some(Frame::Exp { exp, pos, stack_height }) => {
                    //Find out if the expression returns a result or a failure
                    match *exp {
                        //If a return process the blocks opcode
                        Exp::Ret(ref op_seq, vals) => {
                            //Cost: relative to vals.len() |opCode will measure seperately| -- sub stack push will be accounted on push
                            //process each opcode
                            let mut pos = pos;
                            while op_seq.len() > pos {
                                pos+=1;
                                match self.execute_op_code(&op_seq[pos-1])? {
                                    Continuation::Next => {},
                                    //if it is an error propagate it
                                    Continuation::Cont(n_exp,n_stack_hight) => {
                                        self.frames.push(Frame::Exp { exp, pos, stack_height })?;
                                        self.frames.push(Frame::Exp { exp:n_exp, pos:0, stack_height:n_stack_hight })?;
                                        continue 'outer;
                                    }

                                    Continuation::Try(n_exp,catches,n_stack_hight) => {
                                        self.frames.push(Frame::Exp { exp, pos, stack_height })?;
                                        self.frames.push(Frame::Catch { catches, stack_height:n_stack_hight })?;
                                        self.frames.push(Frame::Exp { exp:n_exp, pos:0, stack_height:n_stack_hight })?;
                                        continue 'outer;
                                    }
                                    Continuation::Throw(error) => {
                                        self.frames.push(Frame::Throw(error))?;
                                        continue 'outer;
                                    },
                                }
                            }

                            let tmp = self.temporary_values.temp_arena();
                            //capture each return
                            let workbench = tmp.iter_result_alloc_slice( vals.iter().map(|ValueRef(idx)|self.get(*idx)))?;
                            //reset the stack
                            self.stack.rewind_to(stack_height)?;

                            //push the captured elems (empties workbench)
                            for w in workbench.iter() {
                                self.stack.push(*w)?;
                            }
                        },

                        //If a throw check that it is declared
                        Exp::Throw(error) => self.frames.push(Frame::Throw(error))?
                    }
                },
                Some(Frame::Throw(error))  => {
                    loop {
                        match self.frames.pop() {
                            None => return Ok(false),
                            Some(Frame::Catch {catches, stack_height}) => {
                                match self.catch(catches,error,stack_height)? {
                                    Continuation::Next => {},
                                    //if it is an error propagate it
                                    Continuation::Cont(n_exp,n_stack_height) => {
                                        self.frames.push(Frame::Exp { exp:n_exp, pos:0, stack_height:n_stack_height })?;
                                        continue 'outer;
                                    }

                                    Continuation::Try(n_exp,catches,n_stack_height) => {
                                        self.frames.push(Frame::Catch { catches, stack_height:n_stack_height })?;
                                        self.frames.push(Frame::Exp { exp:n_exp, pos:0, stack_height:n_stack_height })?;
                                        continue 'outer;
                                    },

                                    Continuation::Throw(error) => {
                                        self.frames.push(Frame::Throw(error))?;
                                        continue 'outer;
                                    },
                                }
                            },
                            Some(_) => {}
                        }
                    }
                }
            }
        }
    }

    //The heavy lifter that type checks op code
    fn execute_op_code(&mut self, code: &'code OpCode) -> Result<Continuation<'code>> {
        //Branch on the opcode type and check it
        match *code {
            OpCode::Lit(data, typ) => self.lit(&data, typ),
            OpCode::Let(ref bind) => self.let_(bind),
            OpCode::Unpack(value) => self.unpack(value),
            OpCode::Get(value, field) => self.get_field(value, field),
            OpCode::Switch(value, ref cases) => self.switch(value, cases),
            OpCode::Pack(tag, values) => self.pack(tag, &values),
            OpCode::Invoke(func, values) => self.invoke(func, &values),
            OpCode::Try(ref try, ref catches) => self.try(try, catches),
            OpCode::And(op1,op2) => self.and(op1,op2),
            OpCode::Or(op1,op2) => self.or(op1,op2),
            OpCode::Xor(op1,op2) => self.xor(op1,op2),
            OpCode::Not(op) => self.not(op),
            OpCode::Id(op) => self.copy(op),
            OpCode::ToU(n_size, op) => self.to_u(n_size, op),
            OpCode::ToI(n_size, op) => self.to_i(n_size, op),
            OpCode::Add(op1,op2) => self.add(op1,op2),
            OpCode::Sub(op1,op2) => self.sub(op1,op2),
            OpCode::Mul(op1,op2) => self.mul(op1,op2),
            OpCode::Div(op1,op2) => self.div(op1,op2),
            OpCode::Eq(op1,op2) => self.eq(op1,op2),
            OpCode::Hash(op) => self.hash(op,0), //Todo: Constant
            OpCode::PlainHash(op) => self.plain_hash(op),
            OpCode::ToData(op) => self.to_data(op),
            OpCode::Concat(op1,op2) => self.concat(op1,op2),
            OpCode::Lt(op1,op2) => self.lt(op1,op2),
            OpCode::Gt(op1,op2) => self.gt(op1,op2),
            OpCode::Lte(op1,op2) => self.lte(op1,op2),
            OpCode::Gte(op1,op2) => self.gte(op1,op2),
            OpCode::SetBit(op1,op2, op3) => self.set_bit(op1,op2, op3),
            OpCode::GetBit(op1,op2) => self.get_bit(op1,op2),
            OpCode::GenUnique(op) => self.gen_unique(op),
            OpCode::GenIndex(op) => self.hash(op,1), //Todo: Constant
            OpCode::Derive(op1,op2) => self.join_hash(op1, op2, 2), //Todo: Constant
            OpCode::FullHash => self.fetch_full_hash(),
            OpCode::TxTHash => self.fetch_txt_hash(),
            OpCode::CodeHash => self.fetch_code_hash(),
            OpCode::BlockNo => self.fetch_block_no(),
        }
    }

    //creates a literal
    fn lit(&mut self, data: &[u8], typ: LitDesc) -> Result<Continuation<'code>> {
        //Cost: relative to: data.0.len(), + 1 push
        //create the literal
        let obj = create_lit_object(data, typ, self.alloc)?;
        //push it onto the stack
        self.stack.push(obj)?;
        Ok(Continuation::Next)
    }

    //_ as let is keyword
    // process an EXp isolated
    fn let_(&mut self, bind: &'code Exp) -> Result<Continuation<'code>> {
        //Cost: constant
        //fetch the height
        let stack_height = self.stack.len();
        //execute the block
        Ok(Continuation::Cont(bind, stack_height))
    }

    //helper to get stack elems
    fn get(&self, idx: u16) -> Result<Ptr<'script,Object<'script>>> {
        //calc the pos
        let pos = self.stack.len() - idx as usize - 1;
        //get the elem
        Ok(*self.stack.get(pos)?)
    }

    //unpacks an adt
    fn unpack(&mut self, ValueRef(idx): ValueRef) -> Result<Continuation<'code>> {
        //Cost: relative to: elems.len()
        //get the input
        let elem = self.get(idx)?;
        //must be an adt (static guarantee)
        if let Object::Adt(_, elems) = *elem {
            //push each field
            for e in elems.iter() {
                self.stack.push(*e)?;
            }
            Ok(Continuation::Next)
        } else { unreachable!() }
    }

    //gets a single field from an adt
    fn get_field(&mut self, ValueRef(idx): ValueRef, field:u8) -> Result<Continuation<'code>> {
        //Cost: constant
        //get the input
        let elem = self.get(idx)?;
        //must be an adt (static guarantee)
        if let Object::Adt(_, elems) = &*elem {
            //push the correct field
            self.stack.push(elems[field as usize].clone())?;
            Ok(Continuation::Next)
        } else { unreachable!() }
    }

    //branch based on constructor
    fn switch(&mut self, ValueRef(idx): ValueRef, cases: &'code [Ptr<'code,Exp<'code>>]) -> Result<Continuation<'code>> {
        //Cost: relative to: elems.len()
        //get the input
        let elem = self.get(idx)?;
        //must be an adt (static guarantee)
        if let Object::Adt(tag, elems) = *elem {
            //capture the height
            let stack_height = self.stack.len();
            //push the fields
            for e in elems.iter() {
                self.stack.push(e.clone())?;
            }
            //execute the right branch
            Ok(Continuation::Cont(&cases[tag as usize], stack_height))
        } else { unreachable!() }
    }

    //packs an adt
    fn pack(&mut self, Tag(tag): Tag, values: &[ValueRef]) -> Result<Continuation<'code>> {
        //Cost: relative to: values.len()
        //fetch the inputs
        let mut fields = self.alloc.slice_builder(values.len())?;
        for ValueRef(idx) in values {
            let elem = self.get(*idx)?;
            fields.push(elem);
        }
        //produce an adt with the fields as args
        self.stack.push(self.alloc.alloc(Object::Adt(tag, fields.finish()))?)?;
        Ok(Continuation::Next)
    }

    //create a try block
    fn try(&mut self, try: &'code Exp<'code>, catches: &'code [(Error, Ptr<'code,Exp<'code>>)]) -> Result<Continuation<'code>> {
        //fetch the hight
        let stack_height = self.stack.len();
        //execute the code
        Ok(Continuation::Try(try, catches, stack_height))
    }

    //executes the catch on the way back
    fn catch(&mut self, catches: &'code [(Error, Ptr<'code,Exp<'code>>)], err:Error, start_height:usize) -> Result<Continuation<'code>>{
        //Cost: relative to: catches.len()
        for (e, exp) in catches {
            //if this branche catches it execute it
            if *e == err {
                self.stack.rewind_to(start_height)?;
                return Ok(Continuation::Cont(exp, start_height));
            }
        }
        Ok(Continuation::Next)
    }

    //call a function
    fn invoke(&mut self, fun_idx: u16, values: &[ValueRef]) -> Result<Continuation<'code>> {
        //Cost: relative to: values.len()
        //Non-Native
        //get the code
        let fun_code: &Exp = &self.functions[fun_idx as usize];
        //fetch the height
        let stack_height = self.stack.len();
        //push the arguments
        assert!(values.len() <= u16::max_value() as usize);
        for (i,ValueRef(idx)) in values.iter().enumerate() {
            let elem = self.get(*idx+(i as u16))?; //i counteracts the already pushed elements
            self.stack.push(elem)?;
        }
        //Execute the function
        Ok(Continuation::Cont(fun_code, stack_height))
    }

    fn and(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        //get the arguments and use & on them
        self.stack.push(self.alloc.alloc(match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::I8(op1), Object::I8(op2)) => Object::I8(op1 & op2),
            (Object::U8(op1), Object::U8(op2)) => Object::U8(op1 & op2),
            (Object::I16(op1), Object::I16(op2)) => Object::I16(op1 & op2),
            (Object::U16(op1), Object::U16(op2)) => Object::U16(op1 & op2),
            (Object::I32(op1), Object::I32(op2)) => Object::I32(op1 & op2),
            (Object::U32(op1), Object::U32(op2)) => Object::U32(op1 & op2),
            (Object::I64(op1), Object::I64(op2)) => Object::I64(op1 & op2),
            (Object::U64(op1), Object::U64(op2)) => Object::U64(op1 & op2),
            (Object::I128(op1), Object::I128(op2)) => Object::I128(op1 & op2),
            (Object::U128(op1), Object::U128(op2)) => Object::U128(op1 & op2),
            (Object::Data(op1), Object::Data(op2)) => {
                let mut builder = self.alloc.slice_builder(op1.len()+op2.len())?;
                for i in 0..op1.len() {
                    builder.push(op1[i] & op2[i]);
                }
                Object::Data(builder.finish())
            },
            //adts and the tag (could be a boolean)
            (Object::Adt(op1, _), Object::Adt(op2, _)) => Object::Adt(op1 & op2, SlicePtr::empty()),
            _ => unreachable!()
        })?)?;

        Ok(Continuation::Next)
    }

    fn or(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        //get the arguments and use | on them
        self.stack.push(self.alloc.alloc(match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::I8(op1), Object::I8(op2)) => Object::I8(op1 | op2),
            (Object::U8(op1), Object::U8(op2)) => Object::U8(op1 | op2),
            (Object::I16(op1), Object::I16(op2)) => Object::I16(op1 | op2),
            (Object::U16(op1), Object::U16(op2)) => Object::U16(op1 | op2),
            (Object::I32(op1), Object::I32(op2)) => Object::I32(op1 | op2),
            (Object::U32(op1), Object::U32(op2)) => Object::U32(op1 | op2),
            (Object::I64(op1), Object::I64(op2)) => Object::I64(op1 | op2),
            (Object::U64(op1), Object::U64(op2)) => Object::U64(op1 | op2),
            (Object::I128(op1), Object::I128(op2)) => Object::I128(op1 | op2),
            (Object::U128(op1), Object::U128(op2)) => Object::U128(op1 | op2),
            (Object::Data(op1), Object::Data(op2)) => {
                let mut builder = self.alloc.slice_builder(op1.len()+op2.len())?;
                for i in 0..op1.len() {
                    builder.push(op1[i] | op2[i]);
                }
                Object::Data(builder.finish())
            },
            //adts or the tag (could be a boolean)
            (Object::Adt(op1, _), Object::Adt(op2, _)) => Object::Adt(op1 | op2, SlicePtr::empty()),
            _ => unreachable!()
        })?)?;

        Ok(Continuation::Next)
    }

    fn xor(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        //get the arguments and use ^ on them
        self.stack.push(self.alloc.alloc(match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::I8(op1), Object::I8(op2)) => Object::I8(op1 ^ op2),
            (Object::U8(op1), Object::U8(op2)) => Object::U8(op1 ^ op2),
            (Object::I16(op1), Object::I16(op2)) => Object::I16(op1 ^ op2),
            (Object::U16(op1), Object::U16(op2)) => Object::U16(op1 ^ op2),
            (Object::I32(op1), Object::I32(op2)) => Object::I32(op1 ^ op2),
            (Object::U32(op1), Object::U32(op2)) => Object::U32(op1 ^ op2),
            (Object::I64(op1), Object::I64(op2)) => Object::I64(op1 ^ op2),
            (Object::U64(op1), Object::U64(op2)) => Object::U64(op1 ^ op2),
            (Object::I128(op1), Object::I128(op2)) => Object::I128(op1 ^ op2),
            (Object::U128(op1), Object::U128(op2)) => Object::U128(op1 ^ op2),
            (Object::Data(op1), Object::Data(op2)) => {
                let mut builder = self.alloc.slice_builder(op1.len()+op2.len())?;
                for i in 0..op1.len() {
                    builder.push(op1[i] ^ op2[i]);
                }
                Object::Data(builder.finish())
            },
            //adts xor the tag (could be a boolean)
            (Object::Adt(op1, _), Object::Adt(op2, _)) => Object::Adt(op1 ^ op2, SlicePtr::empty()),
            _ => unreachable!()
        })?)?;

        Ok(Continuation::Next)
    }

    fn not(&mut self, ValueRef(val):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        //get the argument and use ! on them
        self.stack.push(self.alloc.alloc(match &*self.get(val)? {
            Object::I8(op) => Object::I8(!*op),
            Object::U8(op) => Object::U8(!*op),
            Object::I16(op) => Object::I16(!*op),
            Object::U16(op) => Object::U16(!*op),
            Object::I32(op) => Object::I32(!*op),
            Object::U32(op) => Object::U32(!*op),
            Object::I64(op) => Object::I64(!*op),
            Object::U64(op) => Object::U64(!*op),
            Object::I128(op) => Object::I128(!*op),
            Object::U128(op) => Object::U128(!*op),
            //Data is noted byte per byte
            Object::Data(op) => {
                let mut builder = self.alloc.slice_builder(op.len())?;
                for o in op.iter() {
                    builder.push(!*o);
                }
                Object::Data(builder.finish())
            },
            //adts not the tag (could be a boolean)
            Object::Adt(op, _) => Object::Adt(!*op, SlicePtr::empty()),
            _ => unreachable!()
        })?)?;

        Ok(Continuation::Next)
    }

    fn copy(&mut self, ValueRef(val):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        //get the argument and use ! on them
        self.stack.push(self.alloc.alloc(match &*self.get(val)? {
            Object::I8(op) => Object::I8(*op),
            Object::U8(op) => Object::U8(*op),
            Object::I16(op) => Object::I16(*op),
            Object::U16(op) => Object::U16(*op),
            Object::I32(op) => Object::I32(*op),
            Object::U32(op) => Object::U32(*op),
            Object::I64(op) => Object::I64(*op),
            Object::U64(op) => Object::U64(*op),
            Object::I128(op) => Object::I128(*op),
            Object::U128(op) => Object::U128(*op),
            //Data is noted byte per byte
            Object::Data(op) => {
                let mut builder = self.alloc.slice_builder(op.len())?;
                for o in op.iter() {
                    builder.push(*o);
                }
                Object::Data(builder.finish())
            },
            //adts not the tag (could be a boolean)
            Object::Adt(op, _) => Object::Adt(*op, SlicePtr::empty()),
            _ => unreachable!()
        })?)?;

        Ok(Continuation::Next)
    }


    //converts the input to a unsigned int with a width on n_size bytes
    fn to_u(&mut self, n_size: u8, ValueRef(val):ValueRef) -> Result<Continuation<'code>> {
        fn to_u<'script,T: ToPrimitive>(prim: &T, n_size: u8) -> Option<Object<'script>> {
            //cost: relative to: Object size
            //use to a method from another trait to do the conversion
            match n_size {
                1 => prim.to_u8().map(Object::U8),
                2 => prim.to_u16().map(Object::U16),
                4 => prim.to_u32().map(Object::U32),
                8 => prim.to_u64().map(Object::U64),
                16 => prim.to_u128().map(Object::U128),
                _ => unreachable!()
            }
        }

        //cost: relative to: Object size
        let res = match &*self.get(val)? {
            Object::I8(op) => to_u(&*op, n_size),
            Object::U8(op) => to_u(&*op, n_size),
            Object::I16(op) => to_u(&*op, n_size),
            Object::U16(op) => to_u(&*op, n_size),
            Object::I32(op) => to_u(&*op, n_size),
            Object::U32(op) => to_u(&*op, n_size),
            Object::I64(op) => to_u(&*op, n_size),
            Object::U64(op) => to_u(&*op, n_size),
            Object::I128(op) => to_u(&*op, n_size),
            Object::U128(op) => to_u(&*op, n_size),
            _ => unreachable!()
        };

        match res {
            None => Ok(Continuation::Throw(Error::Native(NativeError::NumericError))),
            Some(r) => {
                self.stack.push(self.alloc.alloc(r)?)?;
                Ok(Continuation::Next)
            }
        }

    }

    //converts the input to a signed int with a width on n_size bytes
    fn to_i(&mut self, n_size: u8, ValueRef(val):ValueRef) -> Result<Continuation<'code>> {
        fn to_i<'script,T: ToPrimitive>(prim: &T, n_size: u8) -> Option<Object<'script>> {
            //cost: relative to: Object size
            //use to a method from another trait to do the conversion
            match n_size {
                1 => prim.to_i8().map(Object::I8),
                2 => prim.to_i16().map(Object::I16),
                4 => prim.to_i32().map(Object::I32),
                8 => prim.to_i64().map(Object::I64),
                16 => prim.to_i128().map(Object::I128),
                _ => unreachable!()
            }
        }
        //cost: relative to: Object size
        let res = match &*self.get(val)? {
            Object::I8(op) => to_i(&*op, n_size),
            Object::U8(op) => to_i(&*op, n_size),
            Object::I16(op) => to_i(&*op, n_size),
            Object::U16(op) => to_i(&*op, n_size),
            Object::I32(op) => to_i(&*op, n_size),
            Object::U32(op) => to_i(&*op, n_size),
            Object::I64(op) => to_i(&*op, n_size),
            Object::U64(op) => to_i(&*op, n_size),
            Object::I128(op) => to_i(&*op, n_size),
            Object::U128(op) => to_i(&*op, n_size),
            _ => unreachable!()
        };

        match res {
            None => Ok(Continuation::Throw(Error::Native(NativeError::NumericError))),
            Some(r) => {
                self.stack.push(self.alloc.alloc(r)?)?;
                Ok(Continuation::Next)
            }
        }
    }

    //does an addition (a checked one, returns None in case of Over/under flow)
    fn add(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        let res = match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::I8(op1), Object::I8(op2)) => op1.checked_add(*op2).map(Object::I8),
            (Object::U8(op1), Object::U8(op2)) => op1.checked_add(*op2).map(Object::U8),
            (Object::I16(op1), Object::I16(op2)) => op1.checked_add(*op2).map(Object::I16),
            (Object::U16(op1), Object::U16(op2)) => op1.checked_add(*op2).map(Object::U16),
            (Object::I32(op1), Object::I32(op2)) => op1.checked_add(*op2).map(Object::I32),
            (Object::U32(op1), Object::U32(op2)) => op1.checked_add(*op2).map(Object::U32),
            (Object::I64(op1), Object::I64(op2)) => op1.checked_add(*op2).map(Object::I64),
            (Object::U64(op1), Object::U64(op2)) => op1.checked_add(*op2).map(Object::U64),
            (Object::I128(op1), Object::I128(op2)) => op1.checked_add(*op2).map(Object::I128),
            (Object::U128(op1), Object::U128(op2)) => op1.checked_add(*op2).map(Object::U128),
            _ => unreachable!()
        };

        match res {
            None => Ok(Continuation::Throw(Error::Native(NativeError::NumericError))),
            Some(r) => {
                //Note: Alloc is approx 20ns
                self.stack.push(self.alloc.alloc(r)?)?;
                Ok(Continuation::Next)
            }
        }
    }

    //does a substraction (a checked one, returns None in case of Over/under flow)
    fn sub(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        let res = match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::I8(op1), Object::I8(op2)) => op1.checked_sub(*op2).map(Object::I8),
            (Object::U8(op1), Object::U8(op2)) => op1.checked_sub(*op2).map(Object::U8),
            (Object::I16(op1), Object::I16(op2)) => op1.checked_sub(*op2).map(Object::I16),
            (Object::U16(op1), Object::U16(op2)) => op1.checked_sub(*op2).map(Object::U16),
            (Object::I32(op1), Object::I32(op2)) => op1.checked_sub(*op2).map(Object::I32),
            (Object::U32(op1), Object::U32(op2)) => op1.checked_sub(*op2).map(Object::U32),
            (Object::I64(op1), Object::I64(op2)) => op1.checked_sub(*op2).map(Object::I64),
            (Object::U64(op1), Object::U64(op2)) => op1.checked_sub(*op2).map(Object::U64),
            (Object::I128(op1), Object::I128(op2)) => op1.checked_sub(*op2).map(Object::I128),
            (Object::U128(op1), Object::U128(op2)) => op1.checked_sub(*op2).map(Object::U128),
            _ => unreachable!()
        };

        match res {
            None => Ok(Continuation::Throw(Error::Native(NativeError::NumericError))),
            Some(r) => {
                self.stack.push(self.alloc.alloc(r)?)?;
                Ok(Continuation::Next)
            }
        }
    }

    //does a multiplication (a checked one, returns None in case of Over/under flow)
    fn mul(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        let res = match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::I8(op1), Object::I8(op2)) => op1.checked_mul(*op2).map(Object::I8),
            (Object::U8(op1), Object::U8(op2)) => op1.checked_mul(*op2).map(Object::U8),
            (Object::I16(op1), Object::I16(op2)) => op1.checked_mul(*op2).map(Object::I16),
            (Object::U16(op1), Object::U16(op2)) => op1.checked_mul(*op2).map(Object::U16),
            (Object::I32(op1), Object::I32(op2)) => op1.checked_mul(*op2).map(Object::I32),
            (Object::U32(op1), Object::U32(op2)) => op1.checked_mul(*op2).map(Object::U32),
            (Object::I64(op1), Object::I64(op2)) => op1.checked_mul(*op2).map(Object::I64),
            (Object::U64(op1), Object::U64(op2)) => op1.checked_mul(*op2).map(Object::U64),
            (Object::I128(op1), Object::I128(op2)) => op1.checked_mul(*op2).map(Object::I128),
            (Object::U128(op1), Object::U128(op2)) => op1.checked_mul(*op2).map(Object::U128),
            _ => unreachable!()
        };

        match res {
            None => Ok(Continuation::Throw(Error::Native(NativeError::NumericError))),
            Some(r) => {
                self.stack.push(self.alloc.alloc(r)?)?;
                Ok(Continuation::Next)
            }
        }
    }

    //does a division (a checked one, returns None in case of Over/under flow or division by 0)
    fn div(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        let res = match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::I8(op1), Object::I8(op2)) => op1.checked_div(*op2).map(Object::I8),
            (Object::U8(op1), Object::U8(op2)) => op1.checked_div(*op2).map(Object::U8),
            (Object::I16(op1), Object::I16(op2)) => op1.checked_div(*op2).map(Object::I16),
            (Object::U16(op1), Object::U16(op2)) => op1.checked_div(*op2).map(Object::U16),
            (Object::I32(op1), Object::I32(op2)) => op1.checked_div(*op2).map(Object::I32),
            (Object::U32(op1), Object::U32(op2)) => op1.checked_div(*op2).map(Object::U32),
            (Object::I64(op1), Object::I64(op2)) => op1.checked_div(*op2).map(Object::I64),
            (Object::U64(op1), Object::U64(op2)) => op1.checked_div(*op2).map(Object::U64),
            (Object::I128(op1), Object::I128(op2)) => op1.checked_div(*op2).map(Object::I128),
            (Object::U128(op1), Object::U128(op2)) => op1.checked_div(*op2).map(Object::U128),
            _ => unreachable!()
        };

        match res {
            None => Ok(Continuation::Throw(Error::Native(NativeError::NumericError))),
            Some(r) => {
                self.stack.push(self.alloc.alloc(r)?)?;
                Ok(Continuation::Next)
            }
        }
    }

    //compares the inputs for equality
    fn eq(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        //Note: cost calc is hard without a custom Eq impl as we do not have appropriate info -- make eq similar to hash
        self.stack.push(self.alloc.alloc(Object::Adt(if self.get(val1)? == self.get(val2)? { 1 } else { 0 }, SlicePtr::empty()))?)?;
        Ok(Continuation::Next)
    }

    //hashes the input recursively
    fn hash(&mut self, ValueRef(val):ValueRef, domain:u8) -> Result<Continuation<'code>>  {
        //cost: constant |relative Part in object_hash|
        let top = self.get(val)?;
        //Make a 20 byte digest hascher
        let mut context = Blake2b::new(20);
        //Domain Marker
        context.update(&[domain]);
        //fill the hash
        object_hash(&top, &mut context);
        //calc the Hash
        let hash = context.finalize();
        //generate a array to the hash
        let hash_data_ref = array_ref!(hash.as_bytes(),0,20);
        //get ownership and return
        self.stack.push(self.alloc.alloc(Object::Data(self.alloc.copy_alloc_slice(hash_data_ref)?))?)?;
        Ok(Continuation::Next)
    }

    //hashes the input recursively
    fn join_hash(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, domain:u8) -> Result<Continuation<'code>>  {
        //Get the first value
        let val = self.get(val1)?;
        let data1 = match *val {
            Object::Data(ref data) => data,
            _ => unreachable!()
        };
        //Get the second value
        let val = self.get(val2)?;
        let data2 = match *val {
            Object::Data(ref data) => data,
            _ => unreachable!()
        };

        let mut context = Blake2b::new(20);
        //push the domain
        context.update(&[domain]);
        //fill the hash with first value
        context.update(&data1);
        //fill the hash with second value
        context.update(&data2);
        //calc the Hash
        let hash = context.finalize();
        //generate a array to the hash
        let hash_data_ref = array_ref!(hash.as_bytes(),0,20);
        //get ownership and return
        self.stack.push(self.alloc.alloc(Object::Data(self.alloc.copy_alloc_slice(hash_data_ref)?))?)?;
        Ok(Continuation::Next)
    }

    //a non recursive, non-structural variant that just hashes the data input
    fn plain_hash(&mut self, ValueRef(val):ValueRef) -> Result<Continuation<'code>> {
        let val = self.get(val)?;
        let data = match *val {
            Object::Data(ref data) => data,
            _ => unreachable!()
        };
        let mut context = Blake2b::new(20);
        //fill the hash
        context.update(&data);
        //calc the Hash
        let hash = context.finalize();
        //generate a array to the hash
        let hash_data_ref = array_ref!(hash.as_bytes(),0,20);
        //get ownership and return
        self.stack.push(self.alloc.alloc(Object::Data(self.alloc.copy_alloc_slice(hash_data_ref)?))?)?;
        Ok(Continuation::Next)
    }

    //concats the data inputs
    fn concat(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //get the args
        self.stack.push(self.alloc.alloc(match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::Data(ref op1), Object::Data(ref op2)) => {
                //build a new data vector from the inputs
                let mut conc = Vec::with_capacity(op1.len()+op2.len());
                conc.extend(op1.iter());
                conc.extend(op2.iter());
                Object::Data(self.alloc.copy_alloc_slice(&conc)?)
            },
            _ => unreachable!()
        })?)?;
        Ok(Continuation::Next)
    }

    //compares the inputs for less than
    fn lt(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        self.stack.push(self.alloc.alloc(match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::I8(op1), Object::I8(op2)) => Object::Adt(if op1 < op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U8(op1), Object::U8(op2)) => Object::Adt(if op1 < op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I16(op1), Object::I16(op2)) => Object::Adt(if op1 < op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U16(op1), Object::U16(op2)) => Object::Adt(if op1 < op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I32(op1), Object::I32(op2)) => Object::Adt(if op1 < op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U32(op1), Object::U32(op2)) => Object::Adt(if op1 < op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I64(op1), Object::I64(op2)) => Object::Adt(if op1 < op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U64(op1), Object::U64(op2)) => Object::Adt(if op1 < op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I128(op1), Object::I128(op2)) => Object::Adt(if op1 < op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U128(op1), Object::U128(op2)) => Object::Adt(if op1 < op2 { 1 } else { 0 }, SlicePtr::empty()),
            _ => unreachable!()
        })?)?;
        Ok(Continuation::Next)
    }

    //compares the inputs for greater than
    fn gt(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        self.stack.push(self.alloc.alloc(match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::I8(op1), Object::I8(op2)) => Object::Adt(if op1 > op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U8(op1), Object::U8(op2)) => Object::Adt(if op1 > op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I16(op1), Object::I16(op2)) => Object::Adt(if op1 > op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U16(op1), Object::U16(op2)) => Object::Adt(if op1 > op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I32(op1), Object::I32(op2)) => Object::Adt(if op1 > op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U32(op1), Object::U32(op2)) => Object::Adt(if op1 > op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I64(op1), Object::I64(op2)) => Object::Adt(if op1 > op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U64(op1), Object::U64(op2)) => Object::Adt(if op1 > op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I128(op1), Object::I128(op2)) => Object::Adt(if op1 > op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U128(op1), Object::U128(op2)) => Object::Adt(if op1 > op2 { 1 } else { 0 }, SlicePtr::empty()),
            _ => unreachable!()
        })?)?;
        Ok(Continuation::Next)
    }

    //compares the inputs for less than or equal
    fn lte(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        self.stack.push(self.alloc.alloc(match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::I8(op1), Object::I8(op2)) => Object::Adt(if op1 <= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U8(op1), Object::U8(op2)) => Object::Adt(if op1 <= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I16(op1), Object::I16(op2)) => Object::Adt(if op1 <= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U16(op1), Object::U16(op2)) => Object::Adt(if op1 <= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I32(op1), Object::I32(op2)) => Object::Adt(if op1 <= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U32(op1), Object::U32(op2)) => Object::Adt(if op1 <= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I64(op1), Object::I64(op2)) => Object::Adt(if op1 <= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U64(op1), Object::U64(op2)) => Object::Adt(if op1 <= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I128(op1), Object::I128(op2)) => Object::Adt(if op1 <= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U128(op1), Object::U128(op2)) => Object::Adt(if op1 <= op2 { 1 } else { 0 }, SlicePtr::empty()),
            _ => unreachable!()
        })?)?;
        Ok(Continuation::Next)
    }

    //compares the inputs for greater than or equal
    fn gte(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        self.stack.push(self.alloc.alloc(match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::I8(op1), Object::I8(op2)) => Object::Adt(if op1 >= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U8(op1), Object::U8(op2)) => Object::Adt(if op1 >= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I16(op1), Object::I16(op2)) => Object::Adt(if op1 >= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U16(op1), Object::U16(op2)) => Object::Adt(if op1 >= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I32(op1), Object::I32(op2)) => Object::Adt(if op1 >= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U32(op1), Object::U32(op2)) => Object::Adt(if op1 >= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I64(op1), Object::I64(op2)) => Object::Adt(if op1 >= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U64(op1), Object::U64(op2)) => Object::Adt(if op1 >= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::I128(op1), Object::I128(op2)) => Object::Adt(if op1 >= op2 { 1 } else { 0 }, SlicePtr::empty()),
            (Object::U128(op1), Object::U128(op2)) => Object::Adt(if op1 >= op2 { 1 } else { 0 }, SlicePtr::empty()),
            _ => unreachable!()
        })?)?;
        Ok(Continuation::Next)
    }

    //converts numeric input to data
    //uses byteorder crate for conversion where not trivial
    // conversion is little endian
    fn to_data(&mut self, ValueRef(val):ValueRef) -> Result<Continuation<'code>> {
        //cost: relative to: Object size
        self.stack.push(self.alloc.alloc(match &*self.get(val)? {
            Object::I8(data) => Object::Data(self.alloc.copy_alloc_slice(&[*data as u8])?),
            Object::U8(data) => Object::Data(self.alloc.copy_alloc_slice(&[*data])?),
            Object::I16(data) => {
                let mut input = vec![0; 2];
                LittleEndian::write_i16(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::U16(data) => {
                let mut input = vec![0; 2];
                LittleEndian::write_u16(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::I32(data) => {
                let mut input = vec![0; 4];
                LittleEndian::write_i32(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::U32(data) => {
                let mut input = vec![0; 4];
                LittleEndian::write_u32(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::I64(data) => {
                let mut input = vec![0; 8];
                LittleEndian::write_i64(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::U64(data) => {
                let mut input = vec![0; 8];
                LittleEndian::write_u64(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::I128(data) => {
                let mut input = vec![0; 16];
                LittleEndian::write_i128(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::U128(data) => {
                let mut input = vec![0; 16];
                LittleEndian::write_u128(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            _ => unreachable!(),
        })?)?;
        Ok(Continuation::Next)
    }

    //gets a bit in a data value (as boolean)
    fn get_bit(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef) -> Result<Continuation<'code>> {
        //helper that gets a bit in a vector
        fn get_inner_bit<'script>(v: &[u8], idx: u16) -> Option<Object<'script>> {
            //reverse the index (todo: is this really necessary??)
            let rev_index = (idx / 8) as usize;
            // check if it is in range
            if rev_index >= v.len() { return None }
            //calculate the byte position
            let byte_pos = v.len() - rev_index - 1;
            //calculate the bit position
            let bit_pos = idx % 8;
            //create the mask needed to probe the bit
            let bit_mask = 1u8 << (bit_pos as u8);
            //probe the bit
            if v[byte_pos] & bit_mask != 0 {
                Some(Object::Adt(1, SlicePtr::empty()))
            } else {
                Some(Object::Adt(0, SlicePtr::empty()))
            }
        }
        //cost: constant
        //extract vector and index and get bit
        let res = match (&*self.get(val1)?, &*self.get(val2)?) {
            (Object::Data(op1), Object::U8(op2)) => get_inner_bit(op1, *op2 as u16),
            (Object::Data(op1), Object::U16(op2)) => get_inner_bit(op1, *op2),
            _ => unreachable!()
        };

        match res {
            None => Ok(Continuation::Throw(Error::Native(NativeError::IndexError))),
            Some(r) => {
                self.stack.push(self.alloc.alloc(r)?)?;
                Ok(Continuation::Next)
            }
        }
    }

    //sets a bit in a data value (as boolean)
    fn set_bit(&mut self, ValueRef(val1):ValueRef, ValueRef(val2):ValueRef, ValueRef(val3):ValueRef) -> Result<Continuation<'code>> {
        //helper that sets a bit in a vector
        fn set_inner_bit<'script,'heap>(v: &[u8], idx: u16, val: bool, alloc:&'script VirtualHeapArena<'heap>) -> Result<Option<Object<'script>>> {
            //reverse the index (todo: is this really necessary??)
            let rev_index = (idx / 8) as usize;
            // check if it is in range
            if rev_index >= v.len() { return Ok(None) }
            //calculate the byte position
            let byte_pos = v.len() - rev_index - 1;
            //calculate the bit position
            let bit_pos = idx % 8;
            //create the mask needed to set the bit
            let bit_mask = 1u8 << (bit_pos as u8);
            //prepare a new vector for the result
            let mut new_v = alloc.copy_alloc_mut_slice(v)?;
            //set the bit in the new vector
            if val {
                new_v[byte_pos as usize] = new_v[byte_pos as usize] | bit_mask;
            } else {
                new_v[byte_pos as usize] = new_v[byte_pos as usize] & !bit_mask;
            }
            Ok(Some(Object::Data(new_v.freeze())))
        }
        //cost: relative to data size
        //extract vector and index and set bit
        let res = match (&*self.get(val1)?, &*self.get(val2)?, &*self.get(val3)?) {
            (Object::Data(op1), Object::U8(op2), Object::Adt(tag, _)) => set_inner_bit(op1, *op2 as u16, *tag == 1, self.alloc),
            (Object::Data(op1), Object::U16(op2), Object::Adt(tag, _)) => set_inner_bit(op1, *op2, *tag == 1, self.alloc),
            _ => unreachable!()
        }?;

        match res {
            None => Ok(Continuation::Throw(Error::Native(NativeError::IndexError))),
            Some(r) => {
                self.stack.push(self.alloc.alloc(r)?)?;
                Ok(Continuation::Next)
            }
        }
    }


    //sreate a unique data value from the context
    fn gen_unique(&mut self, ValueRef(val):ValueRef) -> Result<Continuation<'code>> {
        //cost: constant
        match &*self.get(val)? {
            Object::Context(num) => {
                self.stack.push(self.alloc.alloc(Object::Context(num + 1))?)?;       //increase the context so a new value is generated next time
                //derive the value
                self.stack.push(self.alloc.alloc(unique_hash(&self.env.txt_hash, UniqueDomain::Unique, *num, self.alloc)?)?)?;
            },
            _ => unreachable!()
        };
        Ok(Continuation::Next)
    }

    fn fetch_full_hash(&mut self) -> Result<Continuation<'code>> {
        let val = &self.env.full_hash;
        self.stack.push(self.alloc.alloc(Object::Data(self.alloc.copy_alloc_slice(val)?))?)?;
        Ok(Continuation::Next)
    }

    fn fetch_txt_hash(&mut self) -> Result<Continuation<'code>> {
        let val = &self.env.txt_hash;
        self.stack.push(self.alloc.alloc(Object::Data(self.alloc.copy_alloc_slice(val)?))?)?;
        Ok(Continuation::Next)
    }

    fn fetch_code_hash(&mut self) -> Result<Continuation<'code>> {
        let val = &self.env.code_hash;
        self.stack.push(self.alloc.alloc(Object::Data(self.alloc.copy_alloc_slice(val)?))?)?;
        Ok(Continuation::Next)
    }

    fn fetch_block_no(&mut self) -> Result<Continuation<'code>> {
        let val = self.env.block_no;
        self.stack.push(self.alloc.alloc(Object::U64(val))?)?;
        Ok(Continuation::Next)
    }
}