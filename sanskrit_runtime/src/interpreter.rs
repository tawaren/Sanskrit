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

                            let tmp = self.temporary_values.temp_arena()?;
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
    fn invoke(&mut self, func: FunDesc, values: &[ValueRef]) -> Result<Continuation<'code>> {
        match func {
            //Cost: constant
            //execute it if native
            FunDesc::Native(native_fun) => self.execute_native(native_fun, values),
            FunDesc::Custom(fun_idx) => {
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
            },
        }
    }

    //execute a native function
    fn execute_native(&mut self, op: Operand, values: &[ValueRef]) -> Result<Continuation<'code>> {
        //get the return (multi returns handle one inline)
        //A none means an error that must be thrown
        let res = match op {
            //cost: op specific
            Operand::And => self.and(values)?,
            Operand::Or => self.or(values)?,
            Operand::Xor => self.xor(values)?,
            Operand::Not => self.not(values)?,
            Operand::ToU(n_size) => match self.to_u(n_size, values)? {
                None => return Ok(Continuation::Throw(Error::Native(NativeError::NumericError))),
                Some(obj) => obj,
            },
            Operand::ToI(n_size) => match self.to_i(n_size, values)? {
                None => return Ok(Continuation::Throw(Error::Native(NativeError::NumericError))),
                Some(obj) => obj,
            },
            Operand::Add => match self.add(values)? {
                None => return Ok(Continuation::Throw(Error::Native(NativeError::NumericError))),
                Some(obj) => obj,
            },
            Operand::Sub => match self.sub(values)? {
                None => return Ok(Continuation::Throw(Error::Native(NativeError::NumericError))),
                Some(obj) => obj,
            },
            Operand::Mul => match self.mul(values)? {
                None => return Ok(Continuation::Throw(Error::Native(NativeError::NumericError))),
                Some(obj) => obj,
            },
            Operand::Div => match self.div(values)? {
                None => return Ok(Continuation::Throw(Error::Native(NativeError::NumericError))),
                Some(obj) => obj,
            },
            Operand::Eq => self.eq(values)?,
            Operand::Hash => self.hash(values,0)?, //Todo: Constant
            Operand::PlainHash => self.plain_hash(values)?,
            Operand::ToData => self.to_data(values)?,
            Operand::Concat => self.concat(values)?,
            Operand::Lt => self.lt(values)?,
            Operand::Gt => self.gt(values)?,
            Operand::Lte => self.lte(values)?,
            Operand::Gte => self.gte(values)?,
            Operand::SetBit => match self.set_bit(values)? {
                None => return Ok(Continuation::Throw(Error::Native(NativeError::IndexError))),
                Some(obj) => obj,
            },
            Operand::GetBit => match self.get_bit(values)? {
                None => return Ok(Continuation::Throw(Error::Native(NativeError::IndexError))),
                Some(obj) => obj,
            },
            Operand::GenUnique => match self.gen_unique(values)? {
                (ctx, unique) => {
                    self.stack.push(self.alloc.alloc(ctx)?)?;   //we can only return 1 push so do the second manually
                    unique
                }
            },
            Operand::GenIndex => self.hash(values,1)?, //Todo: Constant
            Operand::Derive => self.hash(values, 2)?, //Todo: Constant
            //Cost: constant but op specific
            Operand::FullHash => Object::Data(self.alloc.copy_alloc_slice(&self.env.full_hash)?),
            Operand::TxTHash => Object::Data(self.alloc.copy_alloc_slice(&self.env.txt_hash)?),
            Operand::CodeHash => Object::Data(self.alloc.copy_alloc_slice(&self.env.code_hash)?),
            Operand::BlockNo => Object::U64(self.env.block_no),

        };
        let elem = self.alloc.alloc(res)?;
        self.stack.push(elem)?;
        Ok(Continuation::Next)
    }

    fn and(&self, values: &[ValueRef]) -> Result<Object<'script>> {
        //cost: relative to: Object size
        //get the arguments and use & on them
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
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
                //Data is anded byte for byte
                Object::Data(self.alloc.iter_alloc_slice(op1.iter().zip(op2.iter()).map(|(a, b)| *a & *b))?)
            },
            //adts and the tag (could be a boolean)
            (Object::Adt(op1, _), Object::Adt(op2, _)) => Object::Adt(op1 & op2, SlicePtr::empty()),
            _ => unreachable!()
        })
    }

    fn or(&self, values: &[ValueRef]) -> Result<Object<'script>> {
        //cost: relative to: Object size
        //get the arguments and use | on them
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
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
                //Data is ored byte for byte
                Object::Data(self.alloc.iter_alloc_slice(op1.iter().zip(op2.iter()).map(|(a, b)| *a | *b))?)
            },
            //adts or the tag (could be a boolean)
            (Object::Adt(op1, _), Object::Adt(op2, _)) => Object::Adt(op1 | op2, SlicePtr::empty()),
            _ => unreachable!()
        })
    }

    fn xor(&self, values: &[ValueRef]) -> Result<Object<'script>> {
        //cost: relative to: Object size
        //get the arguments and use ^ on them
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
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
                //Data is xored byte for byte
                Object::Data(self.alloc.iter_alloc_slice(op1.iter().zip(op2.iter()).map(|(a, b)| *a ^ *b))?)
            },
            //adts xor the tag (could be a boolean)
            (Object::Adt(op1, _), Object::Adt(op2, _)) => Object::Adt(op1 ^ op2, SlicePtr::empty()),
            _ => unreachable!()
        })
    }

    fn not(&self, values: &[ValueRef]) -> Result<Object<'script>> {
        //cost: relative to: Object size
        //get the argument and use ! on them
        Ok(match &*self.get(values[0].0)? {
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
            Object::Data(op) => Object::Data(self.alloc.iter_alloc_slice(op.iter().map(|a| !a))?),
            //adts not the tag (could be a boolean)
            Object::Adt(op, _) => Object::Adt(!*op, SlicePtr::empty()),
            _ => unreachable!()
        })
    }

    //converts the input to a unsigned int with a width on n_size bytes
    fn to_u(&self, n_size: u8, values: &[ValueRef]) -> Result<Option<Object<'script>>> {
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
        Ok(match &*self.get(values[0].0)? {
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
        })
    }

    //converts the input to a signed int with a width on n_size bytes
    fn to_i(&self, n_size: u8, values: &[ValueRef]) -> Result<Option<Object<'script>>> {
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
        Ok(match &*self.get(values[0].0)? {
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
        })
    }

    //does an addition (a checked one, returns None in case of Over/under flow)
    fn add(&self, values: &[ValueRef]) -> Result<Option<Object<'script>>> {
        //cost: relative to: Object size
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
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
        })
    }

    //does a substraction (a checked one, returns None in case of Over/under flow)
    fn sub(&self, values: &[ValueRef]) -> Result<Option<Object<'script>>> {
        //cost: relative to: Object size
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
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
        })
    }

    //does a multiplication (a checked one, returns None in case of Over/under flow)
    fn mul(&self, values: &[ValueRef]) -> Result<Option<Object<'script>>> {
        //cost: relative to: Object size
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
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
        })
    }

    //does a division (a checked one, returns None in case of Over/under flow or division by 0)
    fn div(&self, values: &[ValueRef]) -> Result<Option<Object<'script>>> {
        //cost: relative to: Object size
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
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
        })
    }

    //compares the inputs for equality
    fn eq(&self, values: &[ValueRef]) -> Result<Object<'script>> {
        //cost: relative to: Object size
        //Note: cost calc is hard without a custom Eq impl as we do not have appropriate info -- make eq similar to hash
        Ok(Object::Adt(if self.get(values[0].0)? == self.get(values[1].0)? { 1 } else { 0 }, SlicePtr::empty()))
    }

    //hashes the input recursively
    fn hash(&self, values: &[ValueRef], domain:u8) -> Result<Object<'script>>  {
        //cost: constant |relative Part in object_hash|
        let top = self.get(values[0].0)?;
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
        Ok(Object::Data(self.alloc.copy_alloc_slice(hash_data_ref)?))
    }

    //a non recursive, non-structural variant that just hashes the data input
    fn plain_hash(&self, values: &[ValueRef]) -> Result<Object<'script>> {
        let val = self.get(values[0].0)?;
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
        Ok(Object::Data(self.alloc.copy_alloc_slice(hash_data_ref)?))
    }

    //concats the data inputs
    fn concat(&self,values:&[ValueRef]) -> Result<Object<'script>> {
        //get the args
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
            (Object::Data(ref op1), Object::Data(ref op2)) => {
                //build a new data vector from the inputs
                Object::Data(self.alloc.merge_alloc_slice(op1, op2)?)
            },
            _ => unreachable!()
        })
    }

    //compares the inputs for less than
    fn lt(&self, values: &[ValueRef]) -> Result<Object<'script>> {
        //cost: relative to: Object size
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
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
        })
    }

    //compares the inputs for greater than
    fn gt(&self, values: &[ValueRef]) -> Result<Object<'script>> {
        //cost: relative to: Object size
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
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
        })
    }

    //compares the inputs for less than or equal
    fn lte(&self, values: &[ValueRef]) -> Result<Object<'script>> {
        //cost: relative to: Object size
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
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
        })
    }

    //compares the inputs for greater than or equal
    fn gte(&self, values: &[ValueRef]) -> Result<Object<'script>> {
        //cost: relative to: Object size
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
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
        })
    }

    //converts numeric input to data
    //uses byteorder crate for conversion where not trivial
    // conversion is little endian
    fn to_data(&self, values: &[ValueRef]) -> Result<Object<'script>> {
        //cost: relative to: Object size
        Ok(match &*self.get(values[0].0)? {
            Object::I8(data) => Object::Data(self.alloc.copy_alloc_slice(&[*data as u8])?),
            Object::U8(data) => Object::Data(self.alloc.copy_alloc_slice(&[*data])?),
            Object::I16(data) => {
                let mut input = [0; 2];
                LittleEndian::write_i16(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::U16(data) => {
                let mut input = [0; 2];
                LittleEndian::write_u16(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::I32(data) => {
                let mut input = [0; 4];
                LittleEndian::write_i32(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::U32(data) => {
                let mut input = [0; 4];
                LittleEndian::write_u32(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::I64(data) => {
                let mut input = [0; 8];
                LittleEndian::write_i64(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::U64(data) => {
                let mut input = [0; 8];
                LittleEndian::write_u64(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::I128(data) => {
                let mut input = [0; 16];
                LittleEndian::write_i128(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            Object::U128(data) => {
                let mut input = [0; 16];
                LittleEndian::write_u128(&mut input, *data);
                Object::Data(self.alloc.copy_alloc_slice(&input)?)
            },
            _ => unreachable!(),
        })
    }

    //gets a bit in a data value (as boolean)
    fn get_bit(&self, values: &[ValueRef]) -> Result<Option<Object<'script>>> {
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
        Ok(match (&*self.get(values[0].0)?, &*self.get(values[1].0)?) {
            (Object::Data(op1), Object::U8(op2)) => get_inner_bit(op1, *op2 as u16),
            (Object::Data(op1), Object::U16(op2)) => get_inner_bit(op1, *op2),
            _ => unreachable!()
        })
    }

    //sets a bit in a data value (as boolean)
    fn set_bit(&self, values: &[ValueRef]) -> Result<Option<Object<'script>>> {
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
            let mut new_v = v.to_vec();
            //set the bit in the new vector
            if val {
                new_v[byte_pos as usize] = new_v[byte_pos as usize] | bit_mask;
            } else {
                new_v[byte_pos as usize] = new_v[byte_pos as usize] & !bit_mask;
            }
            Ok(Some(Object::Data(alloc.copy_alloc_slice(&new_v)?)))
        }
        //cost: relative to data size
        //extract vector and index and set bit
        match (&*self.get(values[0].0)?, &*self.get(values[1].0)?, &*self.get(values[2].0)?) {
            (Object::Data(op1), Object::U8(op2), Object::Adt(tag, _)) => set_inner_bit(op1, *op2 as u16, *tag == 1, self.alloc),
            (Object::Data(op1), Object::U16(op2), Object::Adt(tag, _)) => set_inner_bit(op1, *op2, *tag == 1, self.alloc),
            _ => unreachable!()
        }
    }


    //sreate a unique data value from the context
    fn gen_unique(&self, values: &[ValueRef]) -> Result<(Object<'script>, Object<'script>)> {
        //cost: constant
        Ok(match &*self.get(values[0].0)? {
            Object::Context(num) => (
                Object::Context(num + 1),       //increase the context so a new value is generated next time
                //derive the value
                unique_hash(&self.env.txt_hash, UniqueDomain::Unique, *num, self.alloc)?
            ),
            _ => unreachable!()
        })
    }
}