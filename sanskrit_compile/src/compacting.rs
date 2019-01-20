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
use sanskrit_runtime::model::Object;
use core::mem;
use sanskrit_common::arena::*;
use sanskrit_common::encoding::ParserAllocator;
use sanskrit_common::encoding::VirtualSize;
use gas_table::gas;

struct State {
    //the gas used in this trace
    gas:usize,
    //the maximal number of gas used
    max_gas:usize,
    //number of active frames at runtime
    frames:usize,
    //maximal number of active frames
    max_frames:usize,
    //number of elements on the stack at runtime
    manifested_stack:usize,
    //maximal number of elements on the stack at runtime
    max_manifest_stack:usize,
    //elements on the stack at compiletime and where to find them on the runtime stack
    stack:Vec<usize>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CallResources {
    pub max_gas:usize,
    pub max_manifest_stack: usize,
    pub max_frames: usize,
}

pub struct Compactor<'b,'h> {
    //state
    state:State,
    //all the embedded functions and where to find them at runtime
    fun_mapping:BTreeMap<(Hash,u8),(u16,u8,CallResources)>,
    //all the embeded errors
    err_mapping:BTreeMap<(Rc<ModuleLink>,u8),Error>,
    //the code of all the embedded function
    functions:Vec<Option<Ptr<'b,RExp<'b>>>>,
    //allocator
    alloc:&'b HeapArena<'h>
}

type BranchPoint = (usize,usize);
type ReturnPoint = (usize,usize);

//todo: Comment
impl State {
    fn new() -> Self {
        State {
            gas: 0,
            max_gas: 0,
            frames: 0,
            max_frames: 0,
            manifested_stack: 0,
            max_manifest_stack: 0,
            stack: Vec::new(),
        }
    }

    fn push_real(&mut self) {
        let pos = self.manifested_stack;
        self.stack.push(pos);
        self.manifested_stack+=1;
        if self.manifested_stack > self.max_manifest_stack {
            self.max_manifest_stack = self.manifested_stack;
        }
    }

    fn push_alias(&mut self, alias:usize) {
        self.stack.push(alias);
    }

    fn add_frame(&mut self) {
        self.frames += 1;
        if self.frames > self.max_frames {
            self.max_frames = self.frames
        }
    }

    //todo: can we do the drop over ret pointers???
    fn drop_frame(&mut self) {
        self.frames -= 1;
    }

    fn use_gas(&mut self, gas:usize) {
        self.gas += gas;
        if self.gas > self.max_gas {
            self.max_gas = self.gas
        }
    }

    fn extract_call_resources(self) -> CallResources {
        CallResources {
            max_gas: self.max_gas,
            max_manifest_stack: self.max_manifest_stack,
            max_frames: self.max_frames,
        }
    }

    fn include_call_resources(&mut self, res:CallResources) {
        self.use_gas(res.max_gas);
        self.max_manifest_stack = self.max_manifest_stack.max(self.manifested_stack + res.max_manifest_stack);
        self.max_frames = self.max_frames.max(self.frames + res.max_frames)
    }

    //todo: how to do for gas and mem, it is not that easy as it must be reapplied
    // idea: basically make a new one like fun call - then at the end: apply the max back into the original
    // needs something like return point but setting original to 0 & then on reapply reverses the role and applies the current to the captured and make the combined the new one
    // is done for catches and switches but not over the let & try

    fn start_branching(&mut self) -> BranchPoint {
        let res = (self.gas, self.max_gas);
        self.gas = 0;
        self.max_gas = 0;
        res
    }

    fn next_branch(&mut self) {
        self.gas = 0;
    }

    fn end_branching(&mut self, (gas, max_gas):BranchPoint) {
        self.gas = (gas + self.max_gas);
        self.max_gas = self.gas.max(max_gas);
    }

    fn return_point(&self) -> ReturnPoint{
        (self.manifested_stack, self.stack.len())
    }

    fn rewind(&mut self, (manifested_stack, stack_size):ReturnPoint){
        self.manifested_stack = manifested_stack;
        while stack_size < self.stack.len() {
            self.stack.pop();
        }
    }
}

impl<'b,'h> Compactor<'b,'h> {
    //Start a new compactor
    pub fn new(alloc:&'b HeapArena<'h>) -> Self {
        Compactor {
            state:State::new(),
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
    pub fn emit_func<S:Store>(&mut self, fun:&FunctionComponent, module:&Hash, offset:u8, context:&Context<S>) -> Result<(u16,u8,CallResources)> {
        match self.fun_mapping.get(&(module.clone(),offset)) {
            None => {
                //find the next free number
                let next_idx = self.functions.len();
                //ensure we do not go over the limit
                if next_idx > u16::max_value() as usize {return size_limit_exceeded_error()}
                //reserve the Slot in the function code array (it is res.0)
                self.functions.push(None);
                //compact the function
                let (processed, resources) = self.process_func(fun, context)?;
                //get the infos needed during compaction (number, number of returns)
                let res = (next_idx as u16,fun.returns.len() as u8, resources);
                //remember the info
                let old = self.fun_mapping.insert((module.clone(),offset), res);
                //  for the case that someone gets the idea to allow recursion
                assert_eq!(old,None);
                //fill the slot with the compacted function
                self.functions[next_idx] = Some(processed);
                Ok(res)
            },
            //if we already have a number return it
            Some(target) => Ok(*target),
        }
    }

    //compacts a function
    fn process_func<S:Store>(&mut self, fun:&FunctionComponent, context:&Context<S>) -> Result<(Ptr<'b,RExp<'b>>,CallResources)> {
        //Prepare a new Stack (Save old one)
        let mut state = State::new();
        mem::swap(&mut self.state, &mut state);
        //ret point
        let ret_point = self.state.return_point();
        //push initial params to the runtime and compiletime stack
        for _ in 0..fun.params.len() {
            self.state.push_real();
        }
        //compact body
        let body = self.process_exp(&fun.code, ret_point, context)?;
        //restore old Stack
        mem::swap(&mut state, &mut &mut self.state);
        //return body & Ressource infos
        Ok((body, state.extract_call_resources()))
    }

    //compacts an expression (block)
    fn process_exp<S:Store>(&mut self, exp:&Exp, ret_point:ReturnPoint, context:&Context<S>) -> Result<Ptr<'b,RExp<'b>>>{
        //account for the frame
        self.state.use_gas(gas::frame_process());
        //differentiate between Return and Throw
        Ok(self.alloc.alloc(match *exp {
            Exp::Ret(ref opcodes, ref returns, ref _drops) => {
                // add the frame
                self.state.add_frame();
                //in case of a return we need to find out which opcodes we can eliminate
                let mut new_opcodes = self.alloc.slice_builder(opcodes.0.len())?;
                for code in &opcodes.0 {
                    //process the opcode
                    match self.process_opcode(code, context)? {
                        None => {}, //can be eliminated
                        Some(n_code) => {
                            self.state.use_gas(gas::op_process());
                            new_opcodes.push(n_code)  //still needed but some things are stripped Aaway
                        },
                    }
                }
                //find the runtime positions of the blocks result
                let adapted = self.alloc.iter_alloc_slice(returns.iter().map(|val|self.translate_ref(*val)))?;
                //Unwind the runtime and compiletime stack
                self.state.rewind(ret_point);
                //push the results on both stacks
                for _ in 0..returns.len() {
                    self.state.push_real();
                }
                // drop the frame
                self.state.drop_frame();
                //Generate and return the optimized Expression
                RExp::Ret(new_opcodes.finish(),adapted)
            },
            Exp::Throw(err) => {
                //Unwind the runtime and compiletime stack (todo: still necessary???)
                self.state.rewind(ret_point);
                //Generate and return the optimized Throw
                RExp::Throw(self.convert_err(err.fetch(context)?)?)
            },
        }))
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
        let pos = self.state.stack.len() - val as usize -1;
        self.state.stack[pos]
    }

    //Takes a compiletime value ref and makes a new runtime value ref
    fn translate_ref(&self, val:ValueRef) -> ValueRef {
        //the pos on the runtime stack
        let pos = self.get(val);
        //the distance from the top of the stack
        let n_index = self.state.manifested_stack - pos -1;
        //assert we are not to far away
        assert!(n_index <= u16::max_value() as usize);
        //generate the result
        ValueRef(n_index as u16)
    }

    fn lit<S:Store>(&mut self, data:&LargeVec<u8>, typ:TypeRef, context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //Extract the kind of lit. This increase runtime rep but speeds up arithmetic
        let r_typ = typ.fetch(context)?;
        let lit_desc = match *r_typ {
            ResolvedType::Native { typ:NativeType::Data(_), .. } => LitDesc::Data,
            ResolvedType::Native { typ:NativeType::Ref, .. } => LitDesc::Ref,
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

        //process the lit
        self.state.use_gas(gas::lit(r_typ));
        //push the lit on both stacks
        self.state.push_real();
        //create the runtime op
        Ok(Some(ROpCode::Lit(self.alloc.copy_alloc_slice(&data.0)?,lit_desc)))
    }

    fn let_<S:Store>(&mut self, exp:&Exp, context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //All cost for let are in process_exp/op_process
        //capture current stack positions
        let ret_point = self.state.return_point();
        //process the nested expression
        let n_exp = self.process_exp(exp, ret_point, context)?;
        //generate the let
        Ok(Some(ROpCode::Let(n_exp)))
    }

    fn copy(&mut self, val:ValueRef) -> Result<Option<ROpCode<'b>>> {
        //just push the compile time stack as the elem already is on the runtime stack
        let pos = self.get(val);
        self.state.push_alias(pos);
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
            self.state.push_alias(pos);
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

            if r_ctr[tag as usize].len() == 0 {
                //eliminate the unpack it produces nothing
                Ok(None)
            }  else {
                //account for the gas
                self.state.use_gas(gas::unpack(r_ctr[tag as usize].len()));
                //push all fields from the ctr to both stacks
                for _ in 0..r_ctr[tag as usize].len(){
                    self.state.push_real()
                }
                //generate the runtime code
                Ok(Some(ROpCode::Unpack(new_ref)))
            }
        }

    }

    fn get_field<S:Store>(&mut self, val:ValueRef, typ:TypeRef, field:u8, context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //get the str information
        let ctrs = context.get_ctrs(typ, context.store)?;
        //check if it i a wrapper
        if ctrs.len() == 1 && ctrs[0].len() == 1 {
            //if a wrapper just push the compile time stack as the elem already is on the runtime stack
            let pos = self.get(val);
            self.state.push_alias(pos);
            //eliminate the unpack
            Ok(None)
        } else {
            //account for the gas
            self.state.use_gas(gas::field());
            //find the runtime pos
            let new_ref = self.translate_ref(val);
            //push the field onto the stack
            self.state.push_real();
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
            self.state.push_alias(pos);
            //eliminate the unpack
            Ok(None)
        } else {
            //account for the gas
            self.state.use_gas(gas::pack(vals.len()));
            //find the input fields position at runtime
            let adapted = self.alloc.iter_alloc_slice(vals.iter().map(|val|self.translate_ref(*val)))?;
            //push the packed element ot both stacks
            self.state.push_real();
            //generate the runtime code
            Ok(Some(ROpCode::Pack(tag,adapted)))
        }
    }

    fn switch<S:Store>(&mut self, val:ValueRef, typ:TypeRef, exps:&[Exp], context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //account for the gas
        self.state.use_gas(gas::switch());
        //find the inputs runtime position
        let new_ref = self.translate_ref(val);
        //capture the stack
        let ret_point = self.state.return_point();
        //process the branches
        let branch_point = self.state.start_branching();
        let mut new_exps = self.alloc.slice_builder(exps.len())?;
        for (tag,exp) in exps.iter().enumerate() {
            //eliminate the stack effects of the previous branch
            self.state.rewind(ret_point);
            assert!(tag <= u8::max_value() as usize);
            //a branch body is a unpack followed by the branch code (reuse the existing function)
            let _ignore = self.unpack(val, typ, Some(Tag(tag as u8)), context)?;
            //process the branch body
            new_exps.push(self.process_exp(exp, ret_point, context)?);
            //ready for the next branch if their is one or not is irrelevant
            self.state.next_branch();
        }
        //finish branching
        self.state.end_branching(branch_point);
        //generate the runtime code
        Ok(Some(ROpCode::Switch(new_ref,new_exps.finish())))
    }

    fn try<S:Store>(&mut self, try:&Exp, catches:&[(ErrorRef, Exp)], context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //account for the gas
        self.state.use_gas(gas::try(catches.len()));
        //tries need a extra frame at runtime to keep track of the catch block
        self.state.add_frame();
        //capture the stack
        let ret_point = self.state.return_point();
        //process the try block body
        let new_try = self.process_exp(try, ret_point, context)?;
        //the extra frame is only needed for the catch part
        self.state.drop_frame();
        //process the catches
        let branch_point = self.state.start_branching();
        let mut new_catches = self.alloc.slice_builder(catches.len())?;
        for (err,exp) in catches {
            //eliminate the stack effects of the previous branch
            self.state.rewind(ret_point);
            //process the catch block
            let new_catch = self.process_exp(exp, ret_point, context)?;
            //convert the error to a runtime error
            let new_err = self.convert_err(err.fetch(context)?)?;
            //record the mapping
            new_catches.push((new_err,new_catch));
            //ready for the next branch if their is one or not is irrelevant
            self.state.next_branch();
        }
        //finish branching
        self.state.end_branching(branch_point);
        //generate the runtime code
        Ok(Some(ROpCode::Try(new_try, new_catches.finish())))
    }

    fn invoke<S:Store>(&mut self, fun:FuncRef, vals:&[ValueRef], context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //diferentiate between native and Custom
        let (f_desc, rets, cost) = match *fun.fetch(context)? {
            ResolvedFunction::Import { ref module, offset, .. } => {
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
                    (index,rets,resources) => {
                        //account for the ressources
                        self.state.include_call_resources(resources);
                        //return the essential info
                        (FunDesc::Custom(index),rets,gas::call(fun_comp.params.len())) //gas is already accounted for -- Maybe call cost???
                    }
                }
            },
            //if it is a native do a case by case transformation (most just map 1 to 1)
            ResolvedFunction::Native { typ, ref applies } => match typ {
                NativeFunc::And => (FunDesc::Native(Operand::And),1, gas::and(applies)),
                NativeFunc::Or => (FunDesc::Native(Operand::Or),1, gas::or(applies)),
                NativeFunc::Xor => (FunDesc::Native(Operand::Xor),1, gas::xor(applies)),
                NativeFunc::Not => (FunDesc::Native(Operand::Not),1, gas::not(applies)),
                //The casts are represented differently at runtime, the compiletime rep was choosen to have Extend not throwing an error
                NativeFunc::SignCast  | NativeFunc::Cut  | NativeFunc::Extend => match *applies[1] { //the type 1 is the target type
                    //runtime is split into signed & unsigned conversions
                    ResolvedType::Native {  typ: NativeType::SInt(size), .. } => (FunDesc::Native(Operand::ToI(size)),1, gas::convert()),
                    ResolvedType::Native {  typ: NativeType::UInt(size), .. } => (FunDesc::Native(Operand::ToU(size)),1, gas::convert()),
                    _ => unreachable!()
                },
                NativeFunc::Add => (FunDesc::Native(Operand::Add),1, gas::add()),
                NativeFunc::Sub => (FunDesc::Native(Operand::Sub),1, gas::sub()),
                NativeFunc::Mul => (FunDesc::Native(Operand::Mul),1, gas::mul()),
                NativeFunc::Div => (FunDesc::Native(Operand::Div),1, gas::div()),
                NativeFunc::Eq => (FunDesc::Native(Operand::Eq),1, gas::eq(applies, context)?),
                NativeFunc::Hash => (FunDesc::Native(Operand::Hash),1, gas::hash(applies, context)?),
                NativeFunc::PlainHash => (FunDesc::Native(Operand::PlainHash),1,gas::hash_plain(applies)),
                NativeFunc::Lt => (FunDesc::Native(Operand::Lt),1, gas::cmp()),
                NativeFunc::Gt => (FunDesc::Native(Operand::Gt),1, gas::cmp()),
                NativeFunc::Lte => (FunDesc::Native(Operand::Lte),1, gas::cmp()),
                NativeFunc::Gte => (FunDesc::Native(Operand::Gte),1, gas::cmp()),
                //to Data can be a no op depending on the types
                NativeFunc::ToData =>  match *applies[0] {
                    //For ints it is needed
                    ResolvedType::Native {  typ: NativeType::UInt(_), .. }
                    | ResolvedType::Native {  typ: NativeType::SInt(_), .. } => (FunDesc::Native(Operand::ToData),1,  gas::to_data(applies)),
                    //the rest are no ops --  same bit representation --
                    _  => {
                        //is a NoOp, as Unique & Singleton have same repr: Data(20) & are unique (prevails uniqueness)
                        //push the compiletime stack
                        let pos = self.get(vals[0]);
                        self.state.push_alias(pos);
                        //return eliminated indicator
                        return Ok(None)
                    }
                },
                NativeFunc::Concat => (FunDesc::Native(Operand::Concat),1, gas::concat(applies)),
                NativeFunc::GetBit => (FunDesc::Native(Operand::GetBit),1, gas::get_bit(applies)),
                NativeFunc::SetBit => (FunDesc::Native(Operand::SetBit),1, gas::set_bit(applies)),
                //is a NoOp, as Unique & Singleton & Manifest & Index & Ref have same repr: Data(20)
                NativeFunc::ToUnique => {
                    //push the compiletime stack
                    let pos = self.get(vals[0]);
                    self.state.push_alias(pos);
                    //return eliminated indicator
                    return Ok(None)
                },
                //todo: Data alloc mem use
                NativeFunc::GenUnique => (FunDesc::Native(Operand::GenUnique),2,gas::gen_unique()),
                NativeFunc::FullHash => (FunDesc::Native(Operand::FullHash),1,gas::fetch_env_hash()),
                NativeFunc::TxTHash => (FunDesc::Native(Operand::TxTHash),1,gas::fetch_env_hash()),
                NativeFunc::CodeHash => (FunDesc::Native(Operand::CodeHash),1,gas::fetch_env_hash()),
                NativeFunc::BlockNo => (FunDesc::Native(Operand::BlockNo),1,gas::fetch_env_val()),
                //Index and Ref have the same runtime behaviour and representation, the difference is only in the type and allowed usage
                NativeFunc::GenIndex |  NativeFunc::ToRef  => match *applies[0] {
                    //this is just hashing - but in the key domain
                    ResolvedType::Native {  typ: NativeType::Data(_), .. } => (FunDesc::Native(Operand::GenIndex),1, gas::hash_plain(applies)),
                    //these are no ops --  same bit representation -- |Index is for toref, others are for genIndex|
                    ResolvedType::Native {  typ: NativeType::Unique, .. }
                    | ResolvedType::Native {  typ: NativeType::Singleton, .. }
                    | ResolvedType::Native {  typ: NativeType::Index, .. } => {
                        //push the compiletime stack
                        let pos = self.get(vals[0]);
                        self.state.push_alias(pos);
                        //return eliminated indicator
                        return Ok(None)
                    }
                    _ => unreachable!()
                },
                NativeFunc::Derive => (FunDesc::Native(Operand::Derive), 1,gas::join_hash()),
            }
        };

        self.state.use_gas(cost);

        //find the params runtime pos
        let adapted = self.alloc.iter_alloc_slice(vals.iter().map(|val|self.translate_ref(*val)))?;

        //push all the results to both stacks
        for _ in 0..rets{
            //its result of a primitive allocs a Object (some do also alloc Data, this is in the corresponding ones)
            self.state.push_real();
        }
        //generate the runtime code
        Ok(Some(ROpCode::Invoke(f_desc,adapted)))
    }

    //extract all the produced code
    pub fn extract_functions(self) -> Result<SlicePtr<'b, Ptr<'b,RExp<'b>>>> {
        self.alloc.iter_alloc_slice(self.functions.into_iter().map(|c|c.unwrap()))
    }
}
