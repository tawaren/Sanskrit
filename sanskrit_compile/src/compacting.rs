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
use sanskrit_core::model::*;
use sanskrit_core::model::linking::Ref;
use sanskrit_interpreter::model::{OpCode as ROpCode, Entry};
use sanskrit_interpreter::model::Exp as RExp;
#[cfg(feature = "dynamic_gas")]
use sanskrit_interpreter::model::TxTFunction;

use sanskrit_core::model::resolved::*;
use sanskrit_common::errors::*;
use sanskrit_common::model::*;
use sanskrit_common::store::*;
use core::mem;
//use std::mem;
use sanskrit_common::arena::*;
use crate::gas_table::gas;
use sanskrit_core::utils::Crc;
use sanskrit_common::encoding::VirtualSize;
use crate::collector::{Collector, CollectResult};
use sanskrit_core::loader::Loader;
use crate::externals::{CompilationResult, ExpResources, CompilationExternals};

struct State {
    //the gas used in this trace
    gas:u64,
    //the maximal number of gas used
    max_gas:u64,

    //the gas used in this trace without calls
    #[cfg(feature = "dynamic_gas")]
    local_gas:u64,
    //the maximal number of gas used without calls
    #[cfg(feature = "dynamic_gas")]
    max_local_gas:u64,

    //the mem used in this trace
    mem:u64,
    //the maximal number of mem used
    max_mem:u64,
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

//Todo: we need to consume dynamic Gas locally in branches of switch and try
//      Otherwise it can happen that it is wrong
pub struct Compactor<'b,'h> {
    //state
    state:State,
    //all the embedded functions and where to find them at runtime
    // boolean marks implements
    fun_mapping:BTreeMap<(Crc<ModuleLink>,u8,bool),(u16,u8,ExpResources)>,
    //the sys of all the embedded functions
    #[cfg(not(feature = "dynamic_gas"))]
    functions:SliceBuilder<'b,Ptr<'b,RExp<'b>>>,
    #[cfg(feature = "dynamic_gas")]
    functions:SliceBuilder<'b,TxTFunction<'b>>,
    //allocator
    alloc:&'b HeapArena<'h>,
    // block
    block: Vec<ROpCode<'b>>
}

#[cfg(not(feature = "dynamic_gas"))]
type BranchPoint = (u64,u64,u64,u64);
#[cfg(feature = "dynamic_gas")]
type BranchPoint = (u64,u64,u64,u64,u64,u64);

type ReturnPoint = (u32,usize);

//todo: Comment
impl State {
    fn new() -> Self {
        State {
            gas: 0,
            max_gas: 0,
            #[cfg(feature = "dynamic_gas")]
            local_gas: 0,
            #[cfg(feature = "dynamic_gas")]
            max_local_gas: 0,
            mem: 0,
            max_mem: 0,
            frames: 0,
            max_frames: 0,
            manifested_stack: 0,
            max_manifest_stack: 0,
            stack: Vec::new(),
        }
    }

    fn push_real(&mut self) -> Result<()>{
        let pos = self.manifested_stack;
        if pos == u16::MAX as u32 {
            return error(||"Stack limit reached")
        }

        self.stack.push(pos as usize);
        self.manifested_stack+=1;
        if self.manifested_stack > self.max_manifest_stack {
            self.max_manifest_stack = self.manifested_stack;
        }
        Ok(())
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

    #[cfg(feature = "dynamic_gas")]
    #[inline(always)]
    fn use_local_gas(&mut self, local_gas:u64){
        self.local_gas = self.local_gas.saturating_add(local_gas);
        if self.local_gas > self.max_local_gas {
            self.max_local_gas = self.local_gas
        }
    }

    fn use_gas(&mut self, gas:u64) {
        self.gas = self.gas.saturating_add(gas);
        if self.gas > self.max_gas {
            self.max_gas = self.gas
        }
        #[cfg(feature = "dynamic_gas")]
        self.use_local_gas(gas);
    }

    #[cfg(feature = "dynamic_gas")]
    fn use_dynamic_gas(&mut self, gas:u64, local_gas:u64) {
        self.gas = self.gas.saturating_add(gas as u64);
        if self.gas > self.max_gas {
            self.max_gas = self.gas
        }
        self.local_gas = self.local_gas.saturating_add(local_gas as u64);
        if self.local_gas > self.max_local_gas {
            self.max_local_gas = self.local_gas
        }
    }

    fn use_mem(&mut self, mem:u64) {
        self.mem = self.mem.saturating_add(mem as u64);
        if self.mem > self.max_mem {
            self.max_mem = self.mem
        }
    }

    fn extract_call_resources(self) -> ExpResources {
        ExpResources {
            gas: self.max_gas,
            #[cfg(feature = "dynamic_gas")]
            local_gas: self.max_local_gas,
            mem: self.max_mem,
            manifest_stack: self.max_manifest_stack,
            frames: self.max_frames,
        }
    }

    fn include_resources(&mut self, res:ExpResources, mult:u64) {
        #[cfg(feature = "dynamic_gas")]
        self.use_dynamic_gas(mult*res.gas, mult*res.local_gas);
        #[cfg(not(feature = "dynamic_gas"))]
        self.use_gas(mult*res.gas);
        self.use_mem(mult*res.mem);
        self.max_frames = self.max_frames.max(self.frames + res.frames);
        self.max_manifest_stack = self.max_manifest_stack.max(self.manifested_stack + res.manifest_stack);

    }

    fn include_call_resources(&mut self, res:ExpResources, mult:u64) {
        //The whole point of local_gas is to exclude nested calls
        #[cfg(feature = "dynamic_gas")]
        self.use_dynamic_gas(mult*res.gas, 0);
        #[cfg(not(feature = "dynamic_gas"))]
        self.use_gas(mult*res.gas);
        self.use_mem(mult*res.mem);
        self.max_frames = self.max_frames.max(self.frames + res.frames);
        self.max_manifest_stack = self.max_manifest_stack.max(self.manifested_stack + res.manifest_stack);

    }

    fn include_tail_call_resources(&mut self, frame_start:u32, res:ExpResources, mult:u64) {
        //The whole point of local_gas is to exclude nested calls
        #[cfg(feature = "dynamic_gas")]
        self.use_dynamic_gas(mult*res.gas, 0);
        #[cfg(not(feature = "dynamic_gas"))]
        self.use_gas(mult*res.gas);
        self.use_mem(mult*res.mem);
        //-1 as we can reuse the current frame
        self.max_frames = self.max_frames.max(self.frames -1 + res.frames);
        //res.manifest_stack + frame_start is the manifested_stack of the tail_call (without the current frames elements)
        self.max_manifest_stack = self.max_manifest_stack.max(self.manifested_stack.max(res.manifest_stack + frame_start));

    }

    #[cfg(feature = "dynamic_gas")]
    fn start_branching(&mut self) -> BranchPoint {
        let res = (self.gas, self.max_gas, self.local_gas, self.max_local_gas, self.mem, self.max_mem);
        self.gas = 0;
        self.max_gas = 0;
        self.local_gas = 0;
        self.max_local_gas = 0;
        self.mem = 0;
        self.max_mem = 0;
        res
    }

    #[cfg(not(feature = "dynamic_gas"))]
    fn start_branching(&mut self) -> BranchPoint {
        let res = (self.gas, self.max_gas, self.mem, self.max_mem);
        self.gas = 0;
        self.max_gas = 0;
        self.mem = 0;
        self.max_mem = 0;
        res
    }

    #[cfg(feature = "dynamic_gas")]
    fn next_branch(&mut self) {
        self.gas = 0;
        self.local_gas = 0;
        self.mem = 0;
    }

    #[cfg(not(feature = "dynamic_gas"))]
    fn next_branch(&mut self) {
        self.gas = 0;
        self.mem = 0;
    }

    #[cfg(not(feature = "dynamic_gas"))]
    fn end_branching(&mut self, (gas, max_gas, mem, max_mem):BranchPoint) {
        self.gas = gas + self.max_gas;
        self.max_gas = self.gas.max(max_gas);
        self.mem = mem + self.max_mem;
        self.max_mem = self.mem.max(max_mem);
    }

    #[cfg(feature = "dynamic_gas")]
    fn end_branching(&mut self, (gas, max_gas, local_gas, max_local_gas, mem, max_mem):BranchPoint) {
        self.gas = gas + self.max_gas;
        self.max_gas = self.gas.max(max_gas);
        self.local_gas = local_gas + self.max_local_gas;
        self.max_local_gas = self.local_gas.max(max_local_gas);
        self.mem = mem + self.max_mem;
        self.max_mem = self.mem.max(max_mem);
    }

    fn return_point(&self) -> ReturnPoint{
        (self.manifested_stack, self.stack.len())
    }

    fn rewind(&mut self, (manifested_stack, stack_size):ReturnPoint){
        self.manifested_stack = manifested_stack;
        self.stack.truncate(stack_size);
    }
}

#[cfg(not(feature = "dynamic_gas"))]
type CollectRes<'b> = (SlicePtr<'b,Ptr<'b,RExp<'b>>>, ExpResources);

#[cfg(feature = "dynamic_gas")]
type CollectRes<'b> = (SlicePtr<'b,TxTFunction<'b>>, ExpResources);

impl<'b,'h> Compactor<'b,'h> {

    pub fn compact<S:Store,CE:CompilationExternals>(fun:&FunctionComponent, body:&Exp, store:&Loader<S>, alloc:&'b HeapArena<'h>) -> Result<CollectRes<'b>> {
        let functions = Collector::collect(fun,store)?;
        let mut compactor = Compactor {
            state:State::new(),
            fun_mapping: BTreeMap::new(),
            functions: alloc.slice_builder(functions.len()+1)?,
            alloc,
            block: Vec::new(),
        };

        for col_res in functions {
            let (key, processed, resources, returns) = match col_res {
                CollectResult::Function(fun_cache) => {
                    let module = fun_cache.module().clone();
                    let fun_comp = fun_cache.retrieve();
                    //we should not get eliminatable functions
                    assert!(!fun_comp.shared.returns.is_empty() || fun_comp.shared.transactional);
                    //get the targets context
                    let new_ctx = Context::from_module_component(fun_comp, &module, true, store)?;
                    //get the body
                    let body = match fun_comp.body {
                        CallableImpl::External => unreachable!("top level functions should not be returned by Collector::collect(fun,store,limiter)?"),
                        CallableImpl::Internal {ref code, ..} => code
                    };

                    //compact the function
                    let (processed, resources) = compactor.process_func::<_,CE>(fun_comp.shared.params.len(), body, &new_ctx)?;
                    //remember the info
                    ((module,fun_cache.offset(), false), processed, resources, fun_comp.shared.returns.len() as u8)
                },
                CollectResult::Implement(impl_cache) => {
                    let module = impl_cache.module().clone();
                    let impl_comp = impl_cache.retrieve();
                    //get the targets context
                    let new_ctx = Context::from_module_component(impl_comp, &module, true, store)?;
                    //get the body
                    let body = match impl_comp.body {
                        CallableImpl::External => unreachable!("top level implement should not be returned by Collector::collect(fun,store,limiter)?"),
                        CallableImpl::Internal {ref code, ..} => code
                    };
                    //get perm
                    let r_perm = impl_comp.sig.fetch(&new_ctx)?;
                    //get the signature
                    let sig = r_perm.get_sig()?;
                    //compute the params
                    let num_params = impl_comp.params.len() + sig.params.len();
                    //compact the function
                    let (processed, resources) = compactor.process_func::<_,CE>(num_params, body, &new_ctx)?;
                    //remember the info
                    ((module,impl_cache.offset(), true), processed, resources, 1)
                }
            };

            //find the next free number
            let next_idx = compactor.functions.len();
            //ensure we do not go over the limit
            if next_idx > u16::MAX as usize {return error(||"Number of functions out of range")}
            //fill the slot with the compacted function
            #[cfg(not(feature = "dynamic_gas"))]
            compactor.functions.push(processed);

            #[cfg(feature = "dynamic_gas")]
            if resources.local_gas > u32::MAX as u64 {return error(||"Consumed Gas out of range")}
            #[cfg(feature = "dynamic_gas")]
            compactor.functions.push(TxTFunction{
                gas: resources.local_gas as u32,
                body: processed
            });


            let old = compactor.fun_mapping.insert(key, (
                next_idx as u16,
                returns,
                resources
            ));
            //  for the case that someone gets the idea to allow recursion
            assert_eq!(old,None);
        }
        //get the top context
        let top_context = Context::from_top_component(fun, store)?;
        //compact the top function
        let (processed, resources) = compactor.process_func::<_,CE>(fun.shared.params.len(), body, &top_context)?;
        //fill the slot with the compacted function
        #[cfg(not(feature = "dynamic_gas"))]
        compactor.functions.push(processed);
        #[cfg(feature = "dynamic_gas")]
        if resources.local_gas > u32::MAX as u64 {return error(||"Consumed Gas out of range")}
        #[cfg(feature = "dynamic_gas")]
        compactor.functions.push(TxTFunction{
            gas: resources.local_gas as u32,
            body: processed
        });
        //get all functions
        let res = compactor.functions.finish();
        //return the result
        Ok((res,resources))
    }


    //compacts a function
    fn process_func<S:Store,CE:CompilationExternals>(&mut self, num_params:usize, code:&Exp, context:&Context<S>) -> Result<(Ptr<'b,RExp<'b>>,ExpResources)> {
        //Prepare a new Stack (Save old one)
        let mut state = State::new();
        mem::swap(&mut self.state, &mut state);
        //ret point
        let ret_point = self.state.return_point();
        //push initial params to the runtime and compiletime stack
        for _ in 0..num_params {
            self.state.push_real()?;
        }
        //compact body
        let (body, _) = self.process_exp::<_,CE>(&code, ret_point, context, None)?;
        //restore old Stack
        mem::swap(&mut state, &mut &mut self.state);
        //return body & Ressource infos
        Ok((body, state.extract_call_resources()))
    }

    fn manifest_stack(&mut self, actual_elems:i16, expected_elems:u8) -> Result<()> {
        //flag that check if manifest is needed
        let mut require_manifest = actual_elems != expected_elems as i16;
        //the return transform param
        let mut rets = self.alloc.slice_builder(expected_elems as usize)?;
        //go over each manifest
        for i in 0..expected_elems {
            //create the virtual value ref
            let val = ValueRef((expected_elems-i-1) as u16);
            //transform into the real value ref
            let fetch = self.translate_ref(val);
            //if the real value ref is different from the virtual one then we need to manifest
            require_manifest = require_manifest | (fetch != val);
            //remember the real value ref
            rets.push(fetch)
        }

        //if we need to manifest
        if require_manifest {
            //charge the gas
            self.state.use_gas(gas::ret(expected_elems as usize));
            //push an opcode
            self.block.push(ROpCode::Return(rets.finish()));
        }
        Ok(())
    }

    //compacts an expression (block)
    fn process_exp<S:Store,CE:CompilationExternals>(&mut self, exp:&Exp, ret_point:ReturnPoint, context:&Context<S>, tail_info:Option<u32>) -> Result<(Ptr<'b,RExp<'b>>, u8)>{
        // add the frame
        if tail_info.is_none() {self.state.add_frame();}
        //in case of a return we need to find out which opcodes we can eliminate
        let old_opcodes = mem::replace(&mut self.block, Vec::with_capacity(exp.0.len()));
        let mut actual_rets = -1;
        let iter = &mut exp.0.iter();
        let len = iter.len();
        //Process all but last
        for code in iter.take(len - 1) {
            //process the opcode
            let (manifest, rets) = self.process_opcode::<_,CE>(code, context, None)?;
            if manifest { actual_rets = rets as i16; }
        }
        //process the last one special (needs adapted tail_info if None it becomes this expressions start)
        let (manifest, expect_rets) = self.process_opcode::<_,CE>(iter.next().unwrap(), context, tail_info.or(Some(ret_point.0)))?;
        //Note: If !manifest then tail_info was ignored anyways (as all actual calls return true for manifest)
        //      Conclusion: If we used tail info for optimisation then actual_rets == expect_rets & The returned elems are on top of the stack already
        //                  Thus self.manifest_stack will not produce a return opcode
        if manifest { actual_rets = expect_rets as i16; }


        //manifest the result of the last Opcode if necessary
        //Note: this is needed as the end of the block requires the values on top of the runtime stack but we may have optimized them away
        self.manifest_stack(actual_rets, expect_rets)?;
        //recover the opcodes and alloc them
        let opcodes =  mem::replace(&mut self.block, old_opcodes);
        let codes = self.alloc.iter_alloc_slice(opcodes.into_iter())?;
        //Unwind the runtime and compiletime stack
        self.state.rewind(ret_point);
        //push the results on both stacks
        for _ in 0..expect_rets {
            self.state.push_real()?;
        }
        // drop the frame
        if tail_info.is_none() {self.state.drop_frame();}
        //Generate and return the optimized Expression
        Ok((self.alloc.alloc(RExp(codes)), expect_rets))
    }

    //compact or even eliminate an opcode
    pub fn process_opcode<S:Store,CE:CompilationExternals>(&mut self, opcode:&OpCode, context:&Context<S>, tail_info:Option<u32>) -> Result<(bool,u8)> {
        //delegate each opcode to a dedicated function
        match *opcode {
            OpCode::Lit(ref data, perm) => self.lit::<_,CE>(data, perm, context),
            OpCode::Let(ref exp) => self.let_::<_,CE>(exp, context, tail_info),
            OpCode::Copy(val) => self.copy(val),
            OpCode::Move(val) =>  self.copy(val),
            OpCode::Return(ref vals) => self._return(vals),
            OpCode::Project(_,val) => self.copy(val),
            OpCode::UnProject(_, val) => self.copy(val),
            OpCode::Discard(_) => Ok((false, 0)),
            OpCode::DiscardMany(_) => Ok((false, 0)),
            OpCode::InspectUnpack(val, perm) => self.unpack(val,perm,None, context),
            OpCode::Unpack(val, perm) => self.unpack(val,perm,None, context),
            OpCode::CopyUnpack(val, perm) => self.unpack(val,perm,None, context),
            OpCode::Field(val, perm, field) => self.get_field(val, perm, field, context),
            OpCode::CopyField(val, perm, field) => self.get_field(val,perm,field, context),
            OpCode::InspectSwitch(val, perm, ref exps) => self.switch::<_,CE>(val, perm, exps, context, tail_info),
            OpCode::Switch(val, perm, ref exps) => self.switch::<_,CE>(val, perm, exps, context, tail_info),
            OpCode::CopySwitch(val, perm, ref exps) => self.switch::<_,CE>(val, perm, exps, context, tail_info),
            OpCode::Pack(perm, tag, ref values) => self.pack(perm, tag,values, context),
            OpCode::CopyPack(perm, tag, ref values) => self.pack(perm, tag,values, context),
            OpCode::Invoke(perm, ref values) =>  match **perm.fetch(context)?.get_fun()? {
                ResolvedCallable::Function{ref module, offset, ..} => self.invoke_fun::<_,CE>(module,offset,values, context, tail_info),
                ResolvedCallable::Implement{ref module, offset, ..} => self.create_sig(module,offset,values, context),
            },
            OpCode::TryInvoke(perm, ref values, ref succ, ref fail) =>  match **perm.fetch(context)?.get_fun()? {
                ResolvedCallable::Function{ref module, offset, ..} => self.try_invoke_fun::<_,CE>(module,offset,values, succ, fail, context, tail_info),
                _ => unreachable!()
            },
            OpCode::RepeatedInvoke(reps, perm, ref values, cond, abort_tag) => match **perm.fetch(context)?.get_fun()? {
                ResolvedCallable::Function{ref module, offset, ..} => self.invoke_repeated_fun(module,offset,values, cond, abort_tag, reps, context, tail_info),
                _ => unreachable!()
            }
            OpCode::RepeatedTryInvoke(reps, perm , ref values, cond, abort_tag, ref succ, ref fail) => match **perm.fetch(context)?.get_fun()? {
                ResolvedCallable::Function{ref module, offset, ..} => self.try_invoke_repeated_fun::<_,CE>(module,offset,values, cond, abort_tag, reps, succ, fail, context, tail_info),
                _ => unreachable!()
            }

            OpCode::InvokeSig(targ, perm, ref values) => self.invoke_sig(targ, perm,values,context),
            OpCode::TryInvokeSig(targ, perm, ref values, ref succ, ref fail) => self.try_invoke_sig::<_,CE>(targ,perm,values,succ,fail, context, tail_info),
            OpCode::RollBack(_, ref produce) => self.rollback(produce),
        }
    }

    //helper to get a value from compiletime stack over a ValueRef
    fn get(&self, val:usize) -> usize {
        let pos = self.state.stack.len() - val - 1;
        self.state.stack[pos]
    }

    //Takes a compiletime value ref and makes a new runtime value ref
    fn translate_ref(&self, val:ValueRef) -> ValueRef {
        //the pos on the runtime stack
        let pos = self.get(val.0 as usize);
        //the distance from the top of the stack
        let n_index = self.state.manifested_stack - (pos as u32) -1;
        //assert we are not to far away
        //Holds as stack is limited to 2^16 entries
        assert!(n_index <= u16::max_value() as u32);
        //generate the result
        ValueRef(n_index as u16)
    }

    fn lit<S:Store, CE:CompilationExternals>(&mut self, data:&LargeVec<u8>, perm:PermRef, context:&Context<S>) -> Result<(bool,u8)> {
        //Extract the kind of lit. This increase runtime rep but speeds up arithmetic
        let r_typ = perm.fetch(context)?.get_type()?.clone();
        if let ResolvedType::Lit {ref module, offset, ..} = *r_typ {
            //load the constructed type from the store
            let data_typ_cache = context.store.get_component::<DataComponent>(&*module, offset)?;
            let data_comp = data_typ_cache.retrieve();

            //push the lit on both stacks
            self.state.push_real()?;

            match data_comp.body {
                DataImpl::Internal {..} => unreachable!(),
                DataImpl::External(_) => {
                    let caller = ModRef(0).fetch(&context)?.to_hash();

                    //compile
                    let (costs, code) = match  CE::compile_lit(&*module, offset, self.alloc.copy_alloc_slice(&data.0)?, &caller, &self.alloc)? {
                        //indicates that this is a no-op
                        CompilationResult::ReorderResult(new_order) => {
                            assert_eq!(new_order.len(), 0);
                            return Ok((false,1))
                        }
                        CompilationResult::OpCodeResult(res,code) => (res,code),
                    };
                    //process the lit gas
                    self.state.include_resources(costs, 1);
                    //create the runtime op
                    self.block.push(code)
                }
            }
        } else {
            unreachable!()
        }
        Ok((true,1))
    }

    fn let_<S:Store,CE:CompilationExternals>(&mut self, exp:&Exp, context:&Context<S>, tail_info:Option<u32>) -> Result<(bool,u8)> {
        //if the let has only one opcode their is no need for the let
        if exp.0.len() == 1 {
            //process the nested expression
            self.process_opcode::<_,CE>(&exp.0[0], context, tail_info)
        } else {
            //cost
            self.state.use_gas(gas::_let());
            //capture current stack positions
            let ret_point = self.state.return_point();
            //process the nested expression
            let (n_exp, rets) = self.process_exp::<_,CE>(exp, ret_point, context, tail_info)?;
            //generate the let
            self.block.push(ROpCode::Let(n_exp));
            Ok((true,rets))
        }
    }

    fn copy(&mut self, val:ValueRef) -> Result<(bool,u8)> {
        //just push the compile time stack as the elem already is on the runtime stack
        let pos = self.get(val.0 as usize);
        self.state.push_alias(pos);
        //copy can be eliminated
        Ok((false,1))
    }

    fn _return(&mut self, vals:&[ValueRef]) -> Result<(bool,u8)> {
        for (offset,fetch) in vals.iter().enumerate() {
            let ValueRef(dist) = *fetch;
            let pos = self.get(dist as usize+offset);
            self.state.push_alias(pos);
        }
        assert!(vals.len() <= u8::max_value() as usize);
        Ok((false,vals.len() as u8))
    }

    fn unpack<S:Store>(&mut self, val:ValueRef, perm:PermRef, tag:Option<Tag>, context:&Context<S>) -> Result<(bool,u8)> {
        //fetch the perm
        let r_perm = perm.fetch(context)?;
        //get the str information
        let r_ctr = r_perm.get_ctrs()?;
        //check if it is a wrapper
        if r_ctr.len() == 1 && r_ctr[0].len() == 1 {
            //if a wrapper just push the compile time stack as the elem already is on the runtime stack
            let pos = self.get(val.0 as usize);
            self.state.push_alias(pos);
            //eliminate the unpack
            Ok((false,1))
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
                Ok((false,0))
            }  else {
                //account for the gas
                self.state.use_gas(gas::unpack(r_ctr[tag as usize].len()));
                //push all fields from the ctr to both stacks
                for _ in 0..r_ctr[tag as usize].len(){
                    self.state.push_real()?
                }
                //generate the runtime sys
                self.block.push(ROpCode::Unpack(new_ref));
                assert!(r_ctr[tag as usize].len() <= u8::max_value() as usize);
                Ok((true,r_ctr[tag as usize].len() as u8))
            }
        }
    }

    fn rollback(&mut self, produces:&[TypeRef]) -> Result<(bool,u8)> {
        //push all produces to both stacks
        for _ in 0..produces.len(){
            self.state.push_real()?
        }
        //generate the runtime sys
        self.block.push(ROpCode::Rollback);
        self.state.use_gas(gas::rollback());

        assert!(produces.len() <= u8::max_value() as usize);
        Ok((true,produces.len() as u8))
    }

    fn get_field<S:Store>(&mut self, val:ValueRef, perm:PermRef, field:u8, context:&Context<S>) -> Result<(bool,u8)> {
        //fetch the perm
        let r_perm = perm.fetch(context)?;
        //get the str information
        let ctrs = r_perm.get_ctrs()?;
        //check if it is a wrapper
        if ctrs.len() == 1 && ctrs[0].len() == 1 {
            //if a wrapper just push the compile time stack as the elem already is on the runtime stack
            let pos = self.get(val.0 as usize);
            self.state.push_alias(pos);
            //eliminate the unpack
            Ok((false,1))
        } else {
            //account for the gas
            self.state.use_gas(gas::field());
            //find the runtime pos
            let new_ref = self.translate_ref(val);
            //push the field onto the stack
            self.state.push_real()?;
            //generate the runtime sys
            self.block.push(ROpCode::Get(new_ref, field));
            Ok((true,1))
        }
    }

    fn pack<S:Store>(&mut self, perm:PermRef, tag:Tag, vals:&[ValueRef], context:&Context<S>) -> Result<(bool,u8)> {
        let r_perm = perm.fetch(context)?;
        //get the str information
        let ctrs = r_perm.get_ctrs()?;
        //check if it is a wrapper
        if ctrs.len() == 1 && ctrs[0].len() == 1 {
            //if a wrapper just push the compile time stack as the elem already is on the runtime stack
            let pos = self.get(vals[0].0 as usize);
            self.state.push_alias(pos);
            //eliminate the unpack
            Ok((false,1))
        } else {
            //check if it is an enum (we optimize these, mainly for efficient booleans)
            //We inline them on stack instead of allocating them on heap
            //account for the gas
            self.state.use_gas(gas::pack(vals.len()));
            //account for the mem
            self.state.use_mem(vals.len() as u64 * Entry::SIZE as u64); //Note T is irrelevant but we provide u8 to please compiler
            //find the input fields position at runtime
            let adapted = self.alloc.iter_alloc_slice(vals.iter().map(|val|self.translate_ref(*val)))?;
            //push the packed element ot both stacks
            self.state.push_real()?;
            //generate the runtime sys
            self.block.push(ROpCode::Pack(tag,adapted));
            Ok((true,1))
        }
    }

    fn switch<S:Store,CE:CompilationExternals>(&mut self, val:ValueRef, perm:PermRef, exps:&[Exp], context:&Context<S>, tail_info:Option<u32>) -> Result<(bool,u8)> {
        //fetch the perm
        let r_perm = perm.fetch(context)?;
        //get the str information
        let r_ctr = r_perm.get_ctrs()?;
        //check if it is a wrapper
        if r_ctr.len() == 1 && r_ctr[0].len() == 1 {
            //if a wrapper just push the compile time stack as the elem already is on the runtime stack
            let pos = self.get(val.0 as usize);
            self.state.push_alias(pos);
            //just emit the single branch
            self.let_::<_,CE>(&exps[0], context, tail_info)
        } else {
            //account for the gas of the switch
            self.state.use_gas(gas::switch());
            //find the inputs runtime position
            let new_ref = self.translate_ref(val);
            //capture the stack
            let ret_point = self.state.return_point();
            //process the branches
            let branch_point = self.state.start_branching();
            let mut new_exps = self.alloc.slice_builder(exps.len())?;
            let mut rets = 0;
            let mut max_fields = 0;
            for (tag,exp) in exps.iter().enumerate() {
                //eliminate the stack effects of the previous branch
                self.state.rewind(ret_point);
                assert!(tag <= u8::MAX as usize);
                //the branch has the unpacked fields
                //We need this for gas accounting
                max_fields = max_fields.max(r_ctr[tag as usize].len());

                //push all fields from the ctr to both stacks
                for _ in 0..r_ctr[tag as usize].len(){
                    self.state.push_real()?
                }
                //process the branch body
                let (n_exp, b_rets) = self.process_exp::<_,CE>(exp, ret_point, context, tail_info)?;
                rets = b_rets;
                //push the exp
                new_exps.push(n_exp);
                //ready for the next branch if their is one or not is irrelevant
                self.state.next_branch();
            }

            //account for the gas of the unpack
            self.state.use_gas(gas::unpack(max_fields));

            //finish branching
            self.state.end_branching(branch_point);

            //check if it is an enum (we optimize these, mainly for efficient booleans)
            //We inline them on stack instead of allocating them on heap
            //generate the runtime sys
            self.block.push(ROpCode::Switch(new_ref,new_exps.finish()));
            Ok((true,rets))
        }
    }

    fn invoke_fun<S:Store,CE:CompilationExternals>(&mut self, module:&Crc<ModuleLink>, offset:u8, vals:&[ValueRef], context:&Context<S>, tail_info:Option<u32>) -> Result<(bool,u8)> {
        match self.invoke_core::<_,CE>(module,offset,vals,context,tail_info)? {
            (Some(code), rets) => {
                //push all the results to both stacks
                for _ in 0..rets{
                    //its result of a primitive allocs a Object (some do also alloc Data, this is in the corresponding ones)
                    self.state.push_real()?;
                }
                //generate the runtime
                self.block.push(code);
                Ok((true,rets))
            },
            (None, rets) => Ok((false,rets))
        }
    }

    fn invoke_repeated_fun<S:Store>(&mut self, module:&Crc<ModuleLink>, offset:u8, vals:&[ValueRef], cond:u8, abort_tag:u8, reps:u8,  context:&Context<S>, tail_info:Option<u32>) -> Result<(bool,u8)> {
        match self.invoke_repeated_core(module,offset,vals,cond, abort_tag, reps, context,tail_info)? {
            (Some(code), rets) => {
                //push all the results to both stacks
                for _ in 0..rets{
                    //its result of a primitive allocs a Object (some do also alloc Data, this is in the corresponding ones)
                    self.state.push_real()?;
                }
                //generate the runtime sys
                self.block.push(code);
                Ok((true,rets))
            },
            (None, rets) => Ok((false,rets))
        }
    }

    //Todo: where is gas cost?
    fn r#try<S:Store,CE:CompilationExternals>(&mut self, code:ROpCode<'b>, rets:u8, vals:&[(bool,ValueRef)], succ:&Exp, fail:&Exp, context:&Context<S>, tail_info:Option<u32>) -> Result<(bool,u8)> {
        //if the inner is a continuation we need to push a try frame
        if is_continuation(&code) {self.state.add_frame()}
        //account for the gas of the try
        self.state.use_gas(gas::r#try());
        //capture the stack
        let ret_point = self.state.return_point();
        //process the branches
        let branch_point = self.state.start_branching();
        //push all the results to both stacks
        for _ in 0..rets{
            //its result of a primitive alloc a Object (some do also alloc Data, this is in the corresponding ones)
            self.state.push_real()?;
        }
        //proccess the expression
        let (new_succ, s_rets) = self.process_exp::<_,CE>(succ, ret_point, context, tail_info)?;
        //ready for the next branch if their is one or not is irrelevant
        self.state.next_branch();
        //eliminate the stack effects of the previous branch
        self.state.rewind(ret_point);
        //push aliases to the stack
        for (offset, (_,ValueRef(val))) in vals.iter().filter(|(e,_)|*e).enumerate(){
            let pos = self.get(*val as usize +offset);
            //push an alias for the old param on the stack
            self.state.push_alias(pos);
        }
        //proccess the expression
        let (new_fail, _)  = self.process_exp::<_,CE>(fail, ret_point, context, tail_info)?;
        //finish branching
        self.state.end_branching(branch_point);
        //if the inner is a continuation we need to drop a try frame
        if is_continuation(&code) {self.state.drop_frame()}
        //generate the runtime sys
        self.block.push(ROpCode::Try(self.alloc.alloc(code),new_succ,new_fail));
        Ok((true,s_rets))
    }

    fn try_invoke_fun<S:Store,CE:CompilationExternals>(&mut self, module:&Crc<ModuleLink>, offset:u8, vals:&[(bool,ValueRef)], succ:&Exp, fail:&Exp, context:&Context<S>, tail_info:Option<u32>) -> Result<(bool,u8)> {
        let plain_vals:Vec<_> = vals.iter().map(|(_,v)|*v).collect();
        match self.invoke_core::<_,CE>(module,offset,&plain_vals, context, None)? {
            (Some(code), rets) => self.r#try::<_,CE>(code,rets,vals,succ,fail, context, tail_info),
            //call was eliminated so we can just continue with the success
            (None, _) => self.let_::<_,CE>(succ, context, tail_info)
        }
    }

    fn try_invoke_repeated_fun<S:Store,CE:CompilationExternals>(&mut self, module:&Crc<ModuleLink>, offset:u8, vals:&[(bool,ValueRef)], cond:u8, abort_tag:u8, reps:u8, succ:&Exp, fail:&Exp, context:&Context<S>, tail_info:Option<u32>) -> Result<(bool,u8)> {
        let plain_vals:Vec<_> = vals.iter().map(|(_,v)|*v).collect();
        match self.invoke_repeated_core(module,offset,&plain_vals,cond, abort_tag, reps, context, None)? {
            (Some(code), rets) => self.r#try::<_,CE>(code,rets,vals,succ,fail,context, tail_info),
            //call was eliminated so we can just continue with the success
            (None, _) => self.let_::<_,CE>(succ, context, tail_info)
        }
    }


    fn invoke_core<S:Store,CE:CompilationExternals>(&mut self, module:&Crc<ModuleLink>, offset:u8, vals:&[ValueRef], context:&Context<S>, tail_info:Option<u32>) -> Result<(Option<ROpCode<'b>>, u8)> {
        //load the called function from the store
        let fun_cache = context.store.get_component::<FunctionComponent>(&*module, offset)?;
        let fun_comp = fun_cache.retrieve();
        //if the function does not have an impact omit it (no returns & no risk will not change anything)
        if fun_comp.shared.returns.is_empty() && !fun_comp.shared.transactional{
            return Ok((None,0))
        }
        //adapted values
        let adapted = self.alloc.iter_alloc_slice(vals.iter().map(|val|self.translate_ref(*val)))?;
        //produce it
        let (code, rets, cost) = match fun_comp.body {
            CallableImpl::External=> {
                //caller fetch
                let caller = ModRef(0).fetch(&context)?.to_hash();
                //compile
                let (costs, code) = match CE::compile_call(&*module, offset, adapted, &caller, &self.alloc)? {
                    //indicates that this is a no-op
                    CompilationResult::ReorderResult(new_order) => {
                        //fetch the aliases
                        assert!(new_order.len() <= u8::MAX as usize);
                        for (offset,fetch) in new_order.iter().enumerate() {
                            let ValueRef(param_offset) = vals[*fetch as usize];
                            let pos = self.get(param_offset as usize + offset);
                            self.state.push_alias(pos);
                        }
                        return Ok((None, new_order.len() as u8))
                    }
                    CompilationResult::OpCodeResult(res,code) => (res,code),
                };
                //account for the ressources
                self.state.include_resources(costs, 1);
                //return the essential info
                (code,fun_comp.shared.returns.len() as u8,gas::call(fun_comp.shared.params.len()))
            },
            CallableImpl::Internal { .. } => {
                //extract the module Hash (needed by emit & context)
                if let Some((index,rets, resources)) = self.fun_mapping.get(&(module.clone(),offset,false)) {
                    //account for the ressources
                    match tail_info {
                        Some(frame_start) => self.state.include_tail_call_resources(frame_start,*resources, 1),
                        None =>  self.state.include_call_resources(*resources, 1)
                    }
                    //return the essential info
                    (ROpCode::Invoke(*index,adapted),*rets,gas::call(fun_comp.shared.params.len()))
                } else {
                    unreachable!()
                }
            }
        };

        self.state.use_gas(cost);
        Ok((Some(code),rets))
    }


    fn invoke_repeated_core<S:Store>(&mut self, module:&Crc<ModuleLink>, offset:u8, vals:&[ValueRef], cond:u8, abort_tag:u8, reps:u8, context:&Context<S>, tail_info:Option<u32>) -> Result<(Option<ROpCode<'b>>, u8)> {
        //load the called function from the store
        let fun_cache = context.store.get_component::<FunctionComponent>(&*module, offset)?;
        let fun_comp = fun_cache.retrieve();
        //if the function does not have an impact omit it (no returns & no risk will not change anything)
        if fun_comp.shared.returns.is_empty() && !fun_comp.shared.transactional{
            return Ok((None,0))
        }
        //adapted values
        let adapted = self.alloc.iter_alloc_slice(vals.iter().map(|val|self.translate_ref(*val)))?;
        let cond_ref = ValueRef((adapted.len() as u16) - cond as u16 - 1);

        //produce it
        let (code, rets, cost) = match fun_comp.body {
            CallableImpl::External => unimplemented!(),
            CallableImpl::Internal { .. } => {
                //extract the module Hash (needed by emit & context)
                if let Some((index,rets, resources)) = self.fun_mapping.get(&(module.clone(),offset, false)) {
                    //account for the ressources
                    match tail_info {
                        Some(frame_start) => self.state.include_tail_call_resources(frame_start,*resources, reps as u64),
                        None => self.state.include_call_resources(*resources, reps as u64)
                    }
                    //return the essential info
                    (ROpCode::RepeatedInvoke(*index,adapted, cond_ref, Tag(abort_tag),reps),*rets,gas::repeated_call(fun_comp.shared.params.len(), reps as u64))
                } else {
                    unreachable!()
                }
            }
        };

        self.state.use_gas(cost);
        Ok((Some(code),rets))
    }

    fn create_sig<S:Store>(&mut self, module:&Crc<ModuleLink>, offset:u8, vals:&[ValueRef], context:&Context<S>) -> Result<(bool,u8)> {
        //load the called function from the store
        let impl_cache = context.store.get_component::<ImplementComponent>(&*module, offset)?;
        let impl_comp = impl_cache.retrieve();
        //get perm
        let r_perm = impl_comp.sig.fetch(context)?;
        //get the signature
        let sig = r_perm.get_sig()?;
        //produce it (if not eliminated)
        let (code, cost) = if sig.returns.is_empty() && !sig.transactional{
            (ROpCode::Void,gas::void())
        } else {
            //extract the module Hash (needed by emit & context)
            if let Some((index,_, resources)) = self.fun_mapping.get(&(module.clone(),offset, true)) {
                //adapted values
                let adapted = self.alloc.iter_alloc_slice(vals.iter().map(|val|self.translate_ref(*val)))?;
                //account for the packing of the pre applied values
                self.state.use_mem(vals.len() as u64 * Entry::SIZE as u64);
                //account for the resources used when it is called
                self.state.include_call_resources(*resources, 1);

                //return the essential info
                (ROpCode::CreateSig(*index,adapted),gas::sig(impl_comp.params.len()))
            } else {
                unreachable!()
            }
        };
        self.state.use_gas(cost);
        //push the result to both stacks
        self.state.push_real()?;
        //generate the runtime sys
        self.block.push(code);
        Ok((true,1))
    }

    fn invoke_sig_core<S:Store>(&mut self, target:ValueRef, perm:PermRef, vals:&[ValueRef], context:&Context<S>)  -> Result<Option<(ROpCode<'b>, u8)>> {
        //get perm
        let r_perm = perm.fetch(context)?;
        //get the signature
        let sig = r_perm.get_sig()?;
        //can we eliminate
        if sig.returns.is_empty() && !sig.transactional{
            return Ok(None)
        }
        //adapt target
        let target_adapted = self.alloc.alloc(self.translate_ref(target));
        //adapted values
        let adapted = self.alloc.iter_alloc_slice(vals.iter().map(|val|self.translate_ref(*val)))?;
        //use gas
        self.state.use_gas(gas::call(sig.params.len()));
        //generate the runtime sys
        Ok(Some((ROpCode::InvokeSig(*target_adapted, adapted),sig.returns.len() as u8)))
    }

    fn invoke_sig<S:Store>(&mut self, target:ValueRef, perm:PermRef, vals:&[ValueRef], context:&Context<S>) -> Result<(bool,u8)> {
        if let Some((code, rets)) = self.invoke_sig_core(target,perm,vals,context)? {
            //push all the results to both stacks
            for _ in 0..rets{
                //its result of a primitive allocs a Object (some do also alloc Data, this is in the corresponding ones)
                self.state.push_real()?;
            }
            //generate the runtime sys
            self.block.push(code);
            Ok((true,rets))
        } else {
            Ok((false,0))
        }
    }

    fn try_invoke_sig<S:Store, CE:CompilationExternals>(&mut self, target:ValueRef, perm:PermRef, vals:&[(bool,ValueRef)], succ:&Exp, fail:&Exp, context:&Context<S>, tail_info:Option<u32>) -> Result<(bool,u8)> {
        let plain_vals:Vec<_> = vals.iter().map(|(_,v)|*v).collect();
        match self.invoke_sig_core(target,perm,&plain_vals,context)?{
            Some((code, rets)) => self.r#try::<_,CE>(code,rets, vals, succ,fail, context, tail_info),
            None => self.let_::<_,CE>(succ, context, tail_info)
        }
    }

}

fn is_continuation(code:&ROpCode) -> bool {
    match *code {
        ROpCode::Let(_)
        | ROpCode::Switch(_, _)
        | ROpCode::InvokeSig(_, _)
        | ROpCode::Invoke(_, _)
        | ROpCode::Try(_, _, _) => true,
        _ => false
    }
}
