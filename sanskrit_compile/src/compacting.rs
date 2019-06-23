//!
//! This is a simple module/function compiler that does some Optimisation but only straight forward ones:
//!   1: It strips away type information as they are not needed at runtime (descriptors still contain type information)
//!   2: It eliminates space extensive information like Module Hashes
//!   3: It eliminates copies, moves, drops and frees as the substructural meaning of them is irrelevant at runtime
//!   4: It eliminates some operation that just change the type but not the value
//!   5: It eliminates functions that do not compute anything and are just needed to for the opaque and substructural types to work
//!   6: Eliminates wrapper types (new type pattern) as they are just a type change and can se the same runtime representation

use sanskrit_core::resolver::Context;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use sanskrit_core::model::*;
use sanskrit_core::model::resolved::ResolvedErr;
use sanskrit_core::model::linking::Ref;
use sanskrit_interpreter::model::OpCode as ROpCode;
use sanskrit_interpreter::model::Exp as RExp;
use sanskrit_core::model::resolved::*;
use sanskrit_common::errors::*;
use sanskrit_common::model::*;
use sanskrit_common::store::*;
use sanskrit_interpreter::model::Error;
use core::mem;
use sanskrit_common::arena::*;
use gas_table::gas;
use sanskrit_interpreter::externals::{CompilationExternals, CallResources};

struct State {
    //the gas used in this trace
    gas:u64,
    //the maximal number of gas used
    max_gas:u64,
    //number of active frames at runtime
    frames:u32,
    //maximal number of active frames
    max_frames:u32,
    //number of elements on the stack at runtime
    manifested_stack:u32,
    //maximal number of elements on the stack at runtime
    max_manifest_stack:u32,
    //elements on the stack at compiletime and where to find them on the runtime stack
    stack:Vec<usize>,
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

type BranchPoint = (u64,u64);
type ReturnPoint = (u32,usize);

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
        self.stack.push(pos as usize);
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

    fn drop_frame(&mut self) {
        self.frames -= 1;
    }

    fn use_gas(&mut self, gas:u64) {
        self.gas = self.gas.saturating_add(gas as u64);
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
        self.gas = gas + self.max_gas;
        self.max_gas = self.gas.max(max_gas);
    }

    fn return_point(&self) -> ReturnPoint{
        (self.manifested_stack, self.stack.len())
    }

    fn rewind(&mut self, (manifested_stack, stack_size):ReturnPoint){
        self.manifested_stack = manifested_stack;
        self.stack.truncate(stack_size);
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
        match self.err_mapping.get(&(err.module.clone(),err.offset)) {
            None => {
                //find the next free number
                let next_idx = self.err_mapping.len();
                //ensure we do not go over the limit
                if next_idx > u16::max_value() as usize {return size_limit_exceeded_error()}
                //generate the runtime error from the number
                let res = Error(next_idx as u16);
                //remember it
                self.err_mapping.insert((err.module.clone(),err.offset), res);
                Ok(res)
            },
            //if we already have a number return it
            Some(ref target) => Ok(**target),
        }
    }

    //Strip a function from the Module Hash and assign it a number
    // Additionally does compact the function and stores it in the transaction (needed to strip it from Hash)
    pub fn emit_func<S:Store,E:CompilationExternals>(&mut self, fun:&FunctionComponent, body:&Exp,  module:&Hash, offset:u8, context:&Context<S>) -> Result<(u16,u8,CallResources)> {
        match self.fun_mapping.get(&(*module,offset)) {
            None => {
                //find the next free number
                let next_idx = self.functions.len();
                //ensure we do not go over the limit
                if next_idx > u16::max_value() as usize {return size_limit_exceeded_error()}
                //reserve the Slot in the function code array (it is res.0)
                self.functions.push(None);
                //compact the function
                let (processed, resources) = self.process_func::<S,E>(fun, body, context)?;
                //get the infos needed during compaction (number, number of returns)
                let res = (next_idx as u16,fun.shared.returns.len() as u8, resources);
                //remember the info
                let old = self.fun_mapping.insert((*module,offset), res);
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
    fn process_func<S:Store,E:CompilationExternals>(&mut self, fun:&FunctionComponent, code:&Exp, context:&Context<S>) -> Result<(Ptr<'b,RExp<'b>>,CallResources)> {
        //Prepare a new Stack (Save old one)
        let mut state = State::new();
        mem::swap(&mut self.state, &mut state);
        //ret point
        let ret_point = self.state.return_point();
        //push initial params to the runtime and compiletime stack
        for _ in 0..fun.shared.params.len() {
            self.state.push_real();
        }
        //compact body
        let body = self.process_exp::<S,E>(&code, ret_point, context)?;
        //restore old Stack
        mem::swap(&mut state, &mut &mut self.state);
        //return body & Ressource infos
        Ok((body, state.extract_call_resources()))
    }

    //compacts an expression (block)
    fn process_exp<S:Store,E:CompilationExternals>(&mut self, exp:&Exp, ret_point:ReturnPoint, context:&Context<S>) -> Result<Ptr<'b,RExp<'b>>>{
        //differentiate between Return and Throw
        Ok(self.alloc.alloc(match *exp {
            Exp::Ret(ref opcodes, ref returns) => {
                //account for the returnd
                self.state.use_gas(gas::ret(returns.len()));
                // add the frame
                self.state.add_frame();
                //in case of a return we need to find out which opcodes we can eliminate
                let mut new_opcodes = self.alloc.slice_builder(opcodes.0.len())?;
                for code in &opcodes.0 {
                    //process the opcode
                    match self.process_opcode::<S,E>(code, context)? {
                        None => {}, //can be eliminated
                        Some(n_code) => {
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
                self.state.use_gas(gas::throw());
                //Unwind the runtime and compiletime stack
                self.state.rewind(ret_point);
                //Generate and return the optimized Throw
                RExp::Throw(self.convert_err(err.fetch(context)?)?)
            },
        }))
    }

    //compact or even eliminate an opcode
    pub fn process_opcode<S:Store,E:CompilationExternals>(&mut self, opcode:&OpCode, context:&Context<S>) -> Result<Option<ROpCode<'b>>>{
        //delegate each opcode to a dedicated function
        match *opcode {
            OpCode::Lit(ref data, typ) => self.lit::<S,E>(data,typ, context),
            OpCode::Let(ref exp) => self.let_::<S,E>(exp, context),
            OpCode::CopyFetch(val) => self.copy(val),
            OpCode::Fetch(val) =>  self.copy(val),
            OpCode::BorrowFetch(val) => self.copy(val),
            OpCode::Image(val) => self.copy(val),
            OpCode::ExtractImage(val) => self.copy(val),
            OpCode::Discard(_) => Ok(None),
            OpCode::DiscardMany(_) => Ok(None),
            OpCode::DiscardBorrowed(_, _) => Ok(None),
            OpCode::BorrowUnpack(val, typ) => self.unpack(val,typ,None, context),
            OpCode::Unpack(val, typ) => self.unpack(val,typ,None, context),
            OpCode::CopyUnpack(val, typ) => self.unpack(val,typ,None, context),
            OpCode::Field(val, typ, field) => self.get_field(val, typ, field, context),
            OpCode::CopyField(val, typ, field) => self.get_field(val,typ,field, context),
            OpCode::BorrowField(val, typ, field) => self.get_field(val,typ,field, context),
            OpCode::BorrowSwitch(val, typ, ref exps) => self.switch::<S,E>(val, typ, exps, context),
            OpCode::Switch(val, typ, ref exps) => self.switch::<S,E>(val, typ, exps, context),
            OpCode::CopySwitch(val, typ, ref exps) => self.switch::<S,E>(val, typ, exps, context),
            OpCode::BorrowPack(typ, tag, ref values) => self.pack(typ, tag,values, context),
            OpCode::Pack(typ, tag, ref values) => self.pack(typ, tag,values, context),
            OpCode::CopyPack(typ, tag, ref values) => self.pack(typ, tag,values, context),
            OpCode::Invoke(func, ref values) => self.invoke::<S,E>(func,values, context),
            OpCode::Try(ref try, ref catches) => self.try::<S,E>(try,catches, context),
            //OpCode::ModuleIndex => self.module_index(context),
            //todo: implement
            OpCode::CreateSig(_,_,_) => unimplemented!(),
            OpCode::InvokeSig(_,_,_) => unimplemented!(),
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
        let n_index = self.state.manifested_stack - (pos as u32) -1;
        //assert we are not to far away
        assert!(n_index <= u16::max_value() as u32);
        //generate the result
        ValueRef(n_index as u16)
    }

    fn lit<S:Store, E:CompilationExternals>(&mut self, data:&LargeVec<u8>, typ:TypeRef, context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //Extract the kind of lit. This increase runtime rep but speeds up arithmetic
        let r_typ = typ.fetch(context)?;
        if let ResolvedType::Lit {ref module, offset, ..} = *r_typ {
            //load the constructed type from the store
            let data_typ_cache = context.store.get_data_type(&*module, offset)?;
            let data_comp = data_typ_cache.retrieve();

            //push the lit on both stacks
            self.state.push_real();

            //todo: can we use the lit_size cache here??
            // if not, where do we use it???
            match data_comp.body {
                DataImpl::Adt{..} | DataImpl::ExternalAdt(_) => unreachable!(),
                DataImpl::Lit(size) => {
                    //process the lit gas
                    self.state.use_gas(gas::lit(size));
                    //create the runtime op
                    Ok(Some(ROpCode::Lit(self.alloc.copy_alloc_slice(&data.0)?)))
                },
                DataImpl::ExternalLit(id,size) => {
                    let caller = ModRef(0).fetch(&context)?.to_hash();
                    let (costs, code) = E::compile_lit(id, self.alloc.copy_alloc_slice(&data.0)?, &caller, &self.alloc)?;
                    //process the lit gas
                    self.state.include_call_resources(costs);
                    //create the runtime op
                    Ok(Some(code))
                }
            }
        } else {
            unreachable!()
        }

    }

    fn let_<S:Store,E:CompilationExternals>(&mut self, exp:&Exp, context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //cost
        self.state.use_gas(gas::_let());
        //capture current stack positions
        let ret_point = self.state.return_point();
        //process the nested expression
        let n_exp = self.process_exp::<S,E>(exp, ret_point, context)?;
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
                None => 0 as u8,
                Some(Tag(t)) => t,
            };

            if r_ctr[tag as usize].is_empty() {
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

    fn switch<S:Store,E:CompilationExternals>(&mut self, val:ValueRef, typ:TypeRef, exps:&[Exp], context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
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
            new_exps.push(self.process_exp::<S,E>(exp, ret_point, context)?);
            //ready for the next branch if their is one or not is irrelevant
            self.state.next_branch();
        }
        //finish branching
        self.state.end_branching(branch_point);
        //generate the runtime code
        Ok(Some(ROpCode::Switch(new_ref,new_exps.finish())))
    }

    fn try<S:Store,E:CompilationExternals>(&mut self, try:&Exp, catches:&[(ErrorRef, Exp)], context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //account for the gas
        self.state.use_gas(gas::try(catches.len()));
        //tries need a extra frame at runtime to keep track of the catch block
        self.state.add_frame();
        //capture the stack
        let ret_point = self.state.return_point();
        //process the try block body
        let new_try = self.process_exp::<S,E>(try, ret_point, context)?;
        //the extra frame is only needed for the catch part
        self.state.drop_frame();
        //process the catches
        let branch_point = self.state.start_branching();
        let mut new_catches = self.alloc.slice_builder(catches.len())?;
        for (err,exp) in catches {
            //eliminate the stack effects of the previous branch
            self.state.rewind(ret_point);
            //process the catch block
            let new_catch = self.process_exp::<S,E>(exp, ret_point, context)?;
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

    /*
    fn module_index<S:Store>(&mut self,context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //Extract the ModuleHash
        let data = context.get_mod(ModRef(0))?.to_hash();
        //process the lit
        self.state.use_gas(gas::module_index());
        //push the lit on both stacks
        self.state.push_real();
        //create the runtime op
        //this is not domain hashed, as everithing else is domain hashed at runtime this is ok
        Ok(Some(ROpCode::Lit(self.alloc.copy_alloc_slice(&data)?,LitDesc::Data)))
    }
    */
    fn invoke<S:Store,E:CompilationExternals>(&mut self, fun:FuncRef, vals:&[ValueRef], context:&Context<S>) -> Result<Option<ROpCode<'b>>> {
        //diferentiate between native and Custom
        let r_fun = fun.fetch(context)?;
        //load the called function from the store
        let fun_cache = context.store.get_func(&*r_fun.module, r_fun.offset)?;
        let fun_comp = fun_cache.retrieve();
        //if the function does not have an impact omit it (no returns & no risk will not change anything)
        if fun_comp.shared.returns.is_empty() && fun_comp.shared.risk.is_empty() {
            return Ok(None)
        }
        //extract the module Hash (needed by emit & context)
        let hash = r_fun.module.to_hash();
        //get the context of the new function
        let new_ctx = Context::from_store_func(fun_comp, hash, &context.store)?;
        //adapted values
        let adapted = self.alloc.iter_alloc_slice(vals.iter().map(|val|self.translate_ref(*val)))?;
        //produce it
        let (code, rets, cost) = match fun_comp.body {
            FunctionImpl::External(call_id) => {
                //caller fetch
                let caller = ModRef(0).fetch(&context)?.to_hash();
                //Collect hints for externals
                let adapted = self.alloc.iter_alloc_slice(vals.iter().map(|val|self.translate_ref(*val)))?;

                //compile
                let (costs, code) = match E::compile_call(call_id, adapted, &caller, &self.alloc)? {
                    //indicates that this is a no-op
                    None => {
                        //for now only id allowed
                        assert_eq!(vals.len(), 1);
                        assert_eq!(fun_comp.shared.returns.len(), 1);
                        let pos = self.get(vals[0]);
                        self.state.push_alias(pos);
                        return Ok(None)
                    },
                    Some(res) => res,
                };

                //account for the ressources
                self.state.include_call_resources(costs);
                //return the essential info
                (code,fun_comp.shared.returns.len() as u8,gas::call(fun_comp.shared.params.len()))
            },
            FunctionImpl::Internal {ref code, ..} => {
                match self.emit_func::<S,E>(fun_comp,  code, &hash,r_fun.offset,&new_ctx)? {
                    (index,rets,resources) => {
                        //adapted values
                        let adapted = self.alloc.iter_alloc_slice(vals.iter().map(|val|self.translate_ref(*val)))?;
                        //account for the ressources
                        self.state.include_call_resources(resources);
                        //return the essential info
                        (ROpCode::Invoke(index,adapted),rets,gas::call(fun_comp.shared.params.len()))
                    }
                }
            }
        };

        self.state.use_gas(cost);
        
        //push all the results to both stacks
        for _ in 0..rets{
            //its result of a primitive allocs a Object (some do also alloc Data, this is in the corresponding ones)
            self.state.push_real();
        }
        //generate the runtime code
        Ok(Some(code))
    }

    //extract all the produced code
    pub fn extract_functions(self) -> Result<SlicePtr<'b, Ptr<'b,RExp<'b>>>> {
        self.alloc.iter_alloc_slice(self.functions.into_iter().map(|c|c.unwrap()))
    }
}
