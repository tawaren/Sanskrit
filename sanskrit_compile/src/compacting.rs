//!
//! This is a simple module/function compiler that does some Optimisation but only straight forward ones:
//!   1: It strips away type information as they are not needed at runtime (descriptors still contain type information)
//!   2: It eliminates space extensive information like Module Hashes
//!   3: It eliminates copies, moves, drops and frees as the substructural meaning of them is irrelevant at runtime
//!   4: It eliminates some operation that just change the type but not the value
//!   5: It eliminates functions that do not compute anything and are just needed to for the opaque and substructural types to work
//!   6: Eliminates wrapper types (new type pattern) as they are just a type change and can se the same runtime representation

use sanskrit_core::resolver::Context;
use alloc::prelude::*;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use sanskrit_core::model::*;
use sanskrit_core::model::resolved::ResolvedErr;
use sanskrit_core::model::linking::Ref;
use sanskrit_runtime::model::OpCode as ROpCode;
use sanskrit_runtime::model::Exp as RExp;
use sanskrit_core::model::resolved::*;
use sanskrit_common::errors::*;
use sanskrit_common::model::*;
use sanskrit_common::store::*;
use sanskrit_runtime::model::Error;
use sanskrit_runtime::model::LitDesc;
use sanskrit_runtime::model::FunDesc;
use sanskrit_runtime::model::Operand;
use core::mem;
use sanskrit_common::arena::*;
use sanskrit_common::encoding::ParserAllocator;

pub struct Compactor<'b,'h> {
    //number of elements on the stack at runtime
    manifested_stack:usize,
    //elements on the stack at compiletime and where to find them on the runtime stack
    stack:Vec<usize>,
    //all the embedded functions and where to find them at runtime
    fun_mapping:BTreeMap<(Hash,u8),(u16,u8)>,
    //all the embeded errors
    err_mapping:BTreeMap<(Rc<ModuleLink>,u8),Error>,
    //the code of all the embedded function
    functions:Vec<Option<Ptr<'b,RExp<'b>>>>,
    //allocator
    alloc:&'b HeapArena<'h>
}

impl<'b,'h> Compactor<'b,'h> {
    //Start a new compactor
    pub fn new(alloc:&'b HeapArena<'h>) -> Self {
        Compactor {
            manifested_stack: 0,
            stack: Vec::new(),
            fun_mapping: BTreeMap::new(),
            err_mapping: BTreeMap::new(),
            functions: Vec::new(),
            alloc
        }
    }

    //Strip a error from the Module Hash and assign it a number
    pub fn convert_err(&mut self, err:Rc<ResolvedErr>) -> Result<Error> {
        match *err {
            ResolvedErr::Import { offset,  ref module} => {
                match self.err_mapping.get(&(module.clone(),offset)) {
                    None => {
                        //find the next free number
                        let next_idx = self.err_mapping.len();
                        //ensure we do not go over the limit
                        if next_idx > u16::max_value() as usize {return size_limit_exceeded_error()}
                        //generate the runtime error from the number
                        let res = Error::Custom(next_idx as u16);
                        //remember it
                        self.err_mapping.insert((module.clone(),offset), res);
                        Ok(res)
                    },
                    //if we already have a number return it
                    Some(ref target) => Ok(**target),
                }

            },
            //Native errors are already compact
            ResolvedErr::Native { err } => Ok(Error::Native(err)),
        }


    }

    //Strip a function from the Module Hash and assign it a number
    // Additionally does compact the function and stores it in the transaction (needed to strip it from Hash)
    pub fn emit_func<S:Store>(&mut self, fun:&FunctionComponent, module:&Hash, offset:u8, context:&Context<S>) -> Result<(u16,u8)> {
        match self.fun_mapping.get(&(module.clone(),offset)) {
            None => {
                //find the next free number
                let next_idx = self.functions.len();
                //ensure we do not go over the limit
                if next_idx > u16::max_value() as usize {return size_limit_exceeded_error()}
                //get the infos needed during compaction (number, number of returns)
                let res = (next_idx as u16,fun.returns.len() as u8);
                //remember the info
                self.fun_mapping.insert((module.clone(),offset), res);
                //reserve the Slot in the function code array (it is res.0)
                self.functions.push(None);
                //compact the function
                let processed = self.process_func(fun, context)?;
                //fill the slot with the compacted function
                self.functions[next_idx] = Some(processed);
                Ok(res)
            },
            //if we already have a number return it
            Some(target) => Ok(*target),
        }
    }

    //compacts a function
    fn process_func<S:Store>(&mut self, fun:&FunctionComponent, context:&Context<S>) -> Result<Ptr<'b,RExp<'b>>> {
        //Prepare a new Stack (Save old one)
        let old_manifest = self.manifested_stack;
        let mut old_stack = Vec::new();
        mem::swap(&mut self.stack, &mut old_stack);
        self.manifested_stack = 0;
        //push initial params to the runtime and compiletime stack
        for _ in 0..fun.params.len() {
            let pos = self.manifested_stack;
            self.stack.push(pos);
            self.manifested_stack+=1;
        }
        //compact body
        let body = self.process_exp(&fun.code, 0, 0, context);
        //restore old Stack
        mem::swap(&mut old_stack, &mut self.stack);
        self.manifested_stack = old_manifest;
        //return body
        body
    }

    //compacts an expression (block)
    fn process_exp<S:Store>(&mut self, exp:&Exp, start_stack:usize, start_manifest:usize, context:&Context<S>) -> Result<Ptr<'b,RExp<'b>>>{
        //differentiate between Return and Throw
        self.alloc.alloc(match *exp {
            Exp::Ret(ref opcodes, ref returns, ref _drops) => {
                //in case of a return we need to find out which opcodes we can eliminate
                let mut new_opcodes = self.alloc.slice_builder(opcodes.0.len())?;
                for code in &opcodes.0 {
                    //process the opcode
                    match self.process_opcode(code, context)? {
                        None => {}, //can be eliminated
                        Some(n_code) => new_opcodes.push(n_code),   //still needed but some things are stripped Aaway
                    }
                }
                //find the runtime positions of the blocks result
                let adapted = self.alloc.iter_alloc_slice(returns.iter().map(|val|self.translate_ref(*val)))?;
                //Unwind the runtime and compiletime stack
                self.manifested_stack = start_manifest;
                while start_stack < self.stack.len() { self.stack.pop(); }
                //push the results on both stacks
                for _ in 0..returns.len() {
                    let pos = self.manifested_stack;
                    self.stack.push(pos);
                    self.manifested_stack+=1;
                }
                //Generate and return the optimized Expression
                RExp::Ret(new_opcodes.finish(),adapted)
            },
            Exp::Throw(err) => {
                //Unwind the runtime and compiletime stack (todo: still necessary???)
                self.manifested_stack = start_manifest;
                while start_stack < self.stack.len() { self.stack.pop(); }
                //Generate and return the optimized Throw
                RExp::Throw(self.convert_err(err.fetch(context)?)?)
            },
        })
    }

    //compact or even eliminate an opcode
    pub fn process_opcode<S:Store>(&mut self, opcode:&OpCode, context:&Context<S>) -> Result<Option<ROpCode<'b>>>{
        //delegate each opcode to a dedicated function
        match *opcode {
            OpCode::Lit(ref data, typ) => self.lit(data,typ, context),
            OpCode::Let(ref exp) => self.let_(exp, context),
            OpCode::CopyFetch(val) => self.copy(val),
            OpCode::Fetch(val) =>  self.copy(val),
            OpCode::BorrowFetch(val) => self.copy(val),
            OpCode::Free(_) => Ok(None),
            OpCode::Drop(_) => Ok(None),
            OpCode::BorrowUnpack(val, typ) => self.unpack(val,typ,None, context),
            OpCode::Unpack(val, typ) => self.unpack(val,typ,None, context),
            OpCode::Field(val, typ, field) => self.get_field(val, typ, field, context),
            OpCode::CopyField(val, typ, field) => self.get_field(val,typ,field, context),
            OpCode::BorrowField(val, typ, field) => self.get_field(val,typ,field, context),
            OpCode::BorrowSwitch(val, typ, ref exps) => self.switch(val, typ, exps, context),
            OpCode::Switch(val, typ, ref exps) => self.switch(val, typ, exps, context),
            OpCode::BorrowPack(typ, tag, ref values) => self.pack(typ, tag,values, context),
            OpCode::Pack(typ, tag, ref values) => self.pack(typ, tag,values, context),
            OpCode::CopyPack(typ, tag, ref values) => self.pack(typ, tag,values, context),
            OpCode::Invoke(func, ref values) => self.invoke(func,values, context),
            OpCode::Try(ref try, ref catches) => self.try(try,catches, context),
        }
    }

    //helper to get a value from compiletime stack over a ValueRef
    fn get(&self, ValueRef(val):ValueRef) -> usize {
        let pos = self.stack.len() - val as usize -1;
        self.stack[pos]
    }

    //Takes a compiletime value ref and makes a new runtime value ref
    fn translate_ref(&self, val:ValueRef) -> ValueRef {
        //the pos on the runtime stack
        let pos = self.get(val);
        //the distance from the top of the stack
        let n_index = self.manifested_stack - pos -1;
        //assert we are not to far away
        assert!(n_index <= u16::max_value() as usize);
        //generate the result
        ValueRef(n_index as u16)
    }

    fn lit<S:Store>(&mut self, data:&LargeVec<u8>, typ:TypeRef, context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //Extract the kind of lit. This increase runtime rep but speeds up arithmetic
        let lit_desc = match *typ.fetch(context)? {
            ResolvedType::Native { typ:NativeType::Data(_), .. } => LitDesc::Data,
            ResolvedType::Native { typ:NativeType::SInt(1), .. } => LitDesc::I8,
            ResolvedType::Native { typ:NativeType::UInt(1), .. } => LitDesc::U8,
            ResolvedType::Native { typ:NativeType::SInt(2), .. } => LitDesc::I16,
            ResolvedType::Native { typ:NativeType::UInt(2), .. } => LitDesc::U16,
            ResolvedType::Native { typ:NativeType::SInt(4), .. } => LitDesc::I32,
            ResolvedType::Native { typ:NativeType::UInt(4), .. } => LitDesc::U32,
            ResolvedType::Native { typ:NativeType::SInt(8), .. } => LitDesc::I64,
            ResolvedType::Native { typ:NativeType::UInt(8), .. } => LitDesc::U64,
            ResolvedType::Native { typ:NativeType::SInt(16), .. } => LitDesc::I128,
            ResolvedType::Native { typ:NativeType::UInt(16), .. } => LitDesc::U128,
            _ => unreachable!()
        };

        //push the lit on both stacks
        let pos = self.manifested_stack;
        self.stack.push(pos);
        self.manifested_stack+=1;

        //create the runtime op
        Ok(Some(ROpCode::Lit(self.alloc.copy_alloc_slice(&data.0)?,lit_desc)))
    }

    fn let_<S:Store>(&mut self, exp:&Exp, context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //capture current stack positions
        let stack_depth = self.stack.len();
        let manifest_depth = self.manifested_stack;
        //process the nested expression
        let n_exp = self.process_exp(exp, stack_depth, manifest_depth, context)?;
        //generate the let
        Ok(Some(ROpCode::Let(n_exp)))
    }

    fn copy(&mut self, val:ValueRef) -> Result<Option<ROpCode<'b>>> {
        //just push the compile time stack as the elem already is on the runtime stack
        let pos = self.get(val);
        self.stack.push(pos);
        //copy can be eliminated
        Ok(None)
    }

    fn unpack<S:Store>(&mut self, val:ValueRef, typ:TypeRef, tag:Option<Tag>, context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //get the str information
        let r_ctr = context.get_ctrs(typ, context.store)?;
        //check if it i a wrapper
        if r_ctr.len() == 1 && r_ctr[0].len() == 1 {
            //if a wrapper just push the compile time stack as the elem already is on the runtime stack
            let pos = self.get(val);
            self.stack.push(pos);
            //eliminate the unpack
            Ok(None)
        } else {
            //find the runtime pos
            let new_ref = self.translate_ref(val);
            //find the tag
            let tag = match tag {
                None => 0 as u8,        //todo: maybe have a special struct for these later that spare the tag
                Some(Tag(t)) => t,
            };

            //push all fields from the ctr to both stacks
            for _ in 0..r_ctr[tag as usize].len(){
                let pos = self.manifested_stack;
                self.stack.push(pos);
                self.manifested_stack+=1;
            }
            //generate the runtime code
            Ok(Some(ROpCode::Unpack(new_ref)))
        }

    }

    fn get_field<S:Store>(&mut self, val:ValueRef, typ:TypeRef, field:u8, context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //get the str information
        let ctrs = context.get_ctrs(typ, context.store)?;
        //check if it i a wrapper
        if ctrs.len() == 1 && ctrs[0].len() == 1 {
            //if a wrapper just push the compile time stack as the elem already is on the runtime stack
            let pos = self.get(val);
            self.stack.push(pos);
            //eliminate the unpack
            Ok(None)
        } else {
            //find the runtime pos
            let new_ref = self.translate_ref(val);
            //push the field onto the stack
            let pos = self.manifested_stack;
            self.stack.push(pos);
            self.manifested_stack+=1;
            //generate the runtime code
            Ok(Some(ROpCode::Get(new_ref, field)))
        }
    }

    fn pack<S:Store>(&mut self, typ:TypeRef, tag:Tag, vals:&[ValueRef], context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //get the str information
        let ctrs = context.get_ctrs(typ, context.store)?;
        //check if it i a wrapper
        if ctrs.len() == 1 && ctrs[0].len() == 1 {
            //if a wrapper just push the compile time stack as the elem already is on the runtime stack
            let pos = self.get(vals[0]);
            self.stack.push(pos);
            //eliminate the unpack
            Ok(None)
        } else {
            //find the input fields position at runtime
            let adapted = self.alloc.iter_alloc_slice(vals.iter().map(|val|self.translate_ref(*val)))?;
            //push the packed element ot both stacks
            let pos = self.manifested_stack;
            self.stack.push(pos );
            self.manifested_stack+=1;
            //generate the runtime code
            Ok(Some(ROpCode::Pack(tag,adapted)))
        }
    }

    fn switch<S:Store>(&mut self, val:ValueRef, typ:TypeRef, exps:&[Exp], context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //find the inputs runtime position
        let new_ref = self.translate_ref(val);
        //capture the stack
        let stack_depth = self.stack.len();
        let manifest_depth = self.manifested_stack;

        //process the branches
        let mut new_exps = self.alloc.slice_builder(exps.len())?;
        for (tag,exp) in exps.iter().enumerate() {
            //eliminate the stack effects of the previous branch
            self.manifested_stack = manifest_depth;
            while stack_depth < self.stack.len() { self.stack.pop(); }
            assert!(tag <= u8::max_value() as usize);
            //a branch body is a unpack followed by the branch code (reuse the existing function)
            let _ignore = self.unpack(val, typ, Some(Tag(tag as u8)), context)?;
            //process the branch body
            new_exps.push(self.process_exp(exp, stack_depth, manifest_depth, context)?);
        }
        //generate the runtime code
        Ok(Some(ROpCode::Switch(new_ref,new_exps.finish())))
    }

    fn try<S:Store>(&mut self, try:&Exp, catches:&[(ErrorRef, Exp)], context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //capture the stack
        let stack_depth = self.stack.len();
        let manifest_depth = self.manifested_stack;
        //process the try block body
        let new_try = self.process_exp(try, stack_depth, manifest_depth, context)?;
        //process the catches
        let mut new_catches = self.alloc.slice_builder(catches.len())?;
        for (err,exp) in catches {
            //eliminate the stack effects of the previous branch
            self.manifested_stack = manifest_depth;
            while stack_depth < self.stack.len() { self.stack.pop();}
            //process the catch block
            let new_catch = self.process_exp(exp, stack_depth, manifest_depth, context)?;
            //convert the error to a runtime error
            let new_err = self.convert_err(err.fetch(context)?)?;
            //record the mapping
            new_catches.push((new_err,new_catch))
        }
        //generate the runtime code
        Ok(Some(ROpCode::Try(new_try, new_catches.finish())))
    }

    fn invoke<S:Store>(&mut self, fun:FuncRef, vals:&[ValueRef], context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //diferentiate between native and Custom
        let (f_desc, rets) = match *fun.fetch(context)? {
            ResolvedFunction::Import { ref module, offset, .. } =>{
                //load the called function from the store
                let fun_cache = context.store.get_func(&*module, offset)?;
                let fun_comp = fun_cache.retrieve();
                //if the function does not have an impact omit it (no returns & no risk will not change anything)
                if fun_comp.returns.is_empty() && fun_comp.risk.is_empty() {
                    return Ok(None)
                }
                //extract the module Hash (needed by emit & context)
                let hash = module.to_hash();
                //get the context of the new function
                let new_ctx = Context::from_store_func(fun_comp, hash, &context.store)?;
                //produce it
                match self.emit_func(fun_comp, &hash,offset,&new_ctx)? {
                    (index,rets) => (FunDesc::Custom(index),rets) //return the essential info
                }
            },
            //if it is a native do a case by case transformation (most just map 1 to 1)
            ResolvedFunction::Native { typ, ref applies } => match typ {
                NativeFunc::And => (FunDesc::Native(Operand::And),1),
                NativeFunc::Or => (FunDesc::Native(Operand::Or),1),
                NativeFunc::Xor => (FunDesc::Native(Operand::Xor),1),
                NativeFunc::Not => (FunDesc::Native(Operand::Not),1),
                //The casts are represented differently at runtime, the compiletime rep was choosen to have Extend not throwing an error
                NativeFunc::SignCast  | NativeFunc::Cut  | NativeFunc::Extend => match *applies[1] { //the type 1 is the target type
                    //runtime is split into signed & unsigned conversions
                    ResolvedType::Native {  typ: NativeType::SInt(size), .. } => (FunDesc::Native(Operand::ToI(size)),1),
                    ResolvedType::Native {  typ: NativeType::UInt(size), .. } => (FunDesc::Native(Operand::ToU(size)),1),
                    _ => unreachable!()
                },
                NativeFunc::Add => (FunDesc::Native(Operand::Add),1),
                NativeFunc::Sub => (FunDesc::Native(Operand::Sub),1),
                NativeFunc::Mul => (FunDesc::Native(Operand::Mul),1),
                NativeFunc::Div => (FunDesc::Native(Operand::Div),1),
                NativeFunc::Eq => (FunDesc::Native(Operand::Eq),1),
                NativeFunc::Hash => (FunDesc::Native(Operand::Hash), 1),
                NativeFunc::PlainHash => (FunDesc::Native(Operand::PlainHash),1),
                NativeFunc::Lt => (FunDesc::Native(Operand::Lt),1),
                NativeFunc::Gt => (FunDesc::Native(Operand::Gt),1),
                NativeFunc::Lte => (FunDesc::Native(Operand::Lte),1),
                NativeFunc::Gte => (FunDesc::Native(Operand::Gte),1),
                //to Data can be a no op depending on the types
                NativeFunc::ToData =>  match *applies[0] {
                    //For ints it is needed
                    ResolvedType::Native {  typ: NativeType::UInt(_), .. }
                    | ResolvedType::Native {  typ: NativeType::SInt(_), .. } => (FunDesc::Native(Operand::ToData),1),
                    //the rest are no ops --  same bit representation --
                    _  => {
                        //is a NoOp, as Unique & Singleton have same repr: Data(20) & are unique (prevails uniqueness)
                        //push the compiletime stack
                        let pos = self.get(vals[0]);
                        self.stack.push(pos);
                        //return eliminated indicator
                        return Ok(None)
                    }
                },
                NativeFunc::Concat => (FunDesc::Native(Operand::Concat),1),
                NativeFunc::GetBit => (FunDesc::Native(Operand::GetBit),1),
                NativeFunc::SetBit => (FunDesc::Native(Operand::SetBit),1),
                //is a NoOp, as Unique & Singleton & Manifest & Index & Ref have same repr: Data(20)
                NativeFunc::ToUnique => {
                    //push the compiletime stack
                    let pos = self.get(vals[0]);
                    self.stack.push(pos);
                    //return eliminated indicator
                    return Ok(None)
                }
                NativeFunc::GenUnique => (FunDesc::Native(Operand::GenUnique),2),
                NativeFunc::FullHash => (FunDesc::Native(Operand::FullHash),1),
                NativeFunc::TxTHash => (FunDesc::Native(Operand::TxTHash),1),
                NativeFunc::CodeHash => (FunDesc::Native(Operand::CodeHash),1),
                NativeFunc::BlockNo => (FunDesc::Native(Operand::BlockNo),1),
                //Index and Ref have the same runtime behaviour and representation, the difference is only in the type and allowed usage
                NativeFunc::GenIndex |  NativeFunc::ToRef  => match *applies[0] {
                    //this is just hashing - but in the key domain
                    ResolvedType::Native {  typ: NativeType::Data(_), .. } => (FunDesc::Native(Operand::GenIndex),1),
                    //these are no ops --  same bit representation -- |Index is for toref, others are for genIndex|
                    ResolvedType::Native {  typ: NativeType::Unique, .. }
                    | ResolvedType::Native {  typ: NativeType::Singleton, .. }
                    | ResolvedType::Native {  typ: NativeType::Index, .. } => {
                        //push the compiletime stack
                        let pos = self.get(vals[0]);
                        self.stack.push(pos);
                        //return eliminated indicator
                        return Ok(None)
                    }
                    _ => unreachable!()
                },
                NativeFunc::Derive => (FunDesc::Native(Operand::Derive), 1),
            }
        };

        //find the params runtime pos
        let adapted = self.alloc.iter_alloc_slice(vals.iter().map(|val|self.translate_ref(*val)))?;

        //push all the results to both stacks
        for _ in 0..rets{
            let pos = self.manifested_stack;
            self.stack.push(pos);
            self.manifested_stack+=1;
        }
        //generate the runtime code
        Ok(Some(ROpCode::Invoke(f_desc,adapted)))
    }

    //extract all the produced code
    pub fn extract_functions(self) -> Result<SlicePtr<'b, Ptr<'b,RExp<'b>>>> {
        self.alloc.iter_alloc_slice(self.functions.into_iter().map(|c|c.unwrap()))
    }
}
