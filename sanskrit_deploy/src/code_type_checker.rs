//! A type checker that walks along the ast and checks if the sys is sound
//!
//! The type checker is responsible to check that the inputs to the different Opcodes, as well as the parameters to function calls have the right types.
//!  It further enforces that the needed capabilities are present for restricted opcodes like `Copy`, `Drop`, `Unpack`, ...
//!  Beside checking types it needs to ensure that borrowed values are only used in positions where they are allowed
//!
//! For tracking the types of the values the `linear_type_stack` is heavily used as well as the `native` module to check that the corresponding types are used correctly.
//! After this module checked an Ast without generating an error the types are sound and further step can ignore them, but the information can not be discarded as it is needed by future type checking runs of other sys linking to the current sys
//! Where later phases need some type information like the amount of constructor parameters of a type it is already present in the input sys and its consistency is checked by the type checker
//!  This approach ensures that later phases to not have to load and parse types later on but only sys for functions.

use crate::linear_stack::*;
use sanskrit_core::model::*;
use sanskrit_core::model::linking::*;
use sanskrit_core::model::resolved::*;
use sanskrit_core::model::bitsets::*;
use alloc::vec::Vec;
use sanskrit_core::resolver::Context;
use sanskrit_common::model::*;
use sanskrit_core::loader::StateManager;
use sp1_zkvm_col::arena::URef;

//Todo: Make Configurable
//used to ensure that their is a stack size that prevents stack overflows
const MAX_NESTING_DEPTH:usize = 50;

pub struct TypeCheckerContext<'b, S:StateManager +'b> {
    context: Context<'b, S>,                              //The Resolved Components from the input
    stack: LinearStack<URef<'static,ResolvedType>>,   //The current stack layout
    transactional:bool,
    depth:usize,
    limit:usize,
}

impl<'b, S:StateManager +'b> TypeCheckerContext<'b,S> {

    //Creates a new Empty context
    pub fn new(context: Context<'b, S>) -> Self {
        //Define some reused types and capabilities
        TypeCheckerContext {
            context,
            stack: LinearStack::new(),
            transactional: false,
            depth: 0,
            limit: MAX_NESTING_DEPTH
        }
    }

    fn clean_frame(&mut self, results:u8, start:usize) {
        //how many values are on the stack
        let frame_size = self.stack.stack_depth() - start;

        //is the stack to big?
        assert!(frame_size <= u16::MAX as usize);
        assert!(results as usize <= frame_size);
        //check every not returned item on the stack (start with the last one to clean bottom up)
        for v in (results as u16)..(frame_size as u16) {
            let target = ValueRef(v);
            //discard it if allowed
            //returns the status of a stack elem without modifying anything
            if !self.stack.get_elem(target).can_be_freed() {
                self.discard(target);
            }
        }
    }

    //todo: I hate this duplication but the signtures are different and unification is hard
    pub fn type_check_implement(&mut self, imp:&ImplementComponent, code:&Exp) {
        //Fetch the Permission
        let r_perm = imp.sig.fetch(&self.context);
        //Check that it is the correct one
        assert!(r_perm.check_permission(Permission::Implement));
        //get the signature
        let sig = r_perm.get_sig();
        //Capture transactional declaration
        self.transactional = sig.transactional;
        // num params
        let num_params = imp.params.len()+ sig.params.len();
        //Push the capt parameters onto the stack
        for c in &imp.params {
            //captures are always owned
            self.stack.provide(c.typ.fetch(&self.context));
        }
        //Push the provided parameters onto the stack
        for c in &sig.params {
            //distinguish between owned and borrowed (read-only) parameters
            if c.consumes {
                self.stack.provide(c.typ);
            } else {
                self.stack.borrow(c.typ);
            }
        }

        //Start a new block for th body of the function
        let block = self.stack.start_block();
        //Type check the function body
        let rets = self.type_check_exp(code);

        //Type check the Result
        //Ensure the amount is correct
        assert!(rets as usize == sig.returns.len());
        //iterate over each return (deepest first)
        for (v,t) in sig.returns.iter().rev().enumerate() {
            //Check if the returned value has the expected type
            assert!(v <= u8::MAX as usize);
            assert!(self.stack.value_of(ValueRef(v as u16)) == *t);
        }

        //discard unneeded items
        self.clean_frame(rets, num_params);
        //Close the function body block (leaves just params & results on the Stack)
        self.stack.end_block(block, rets);

        //Check that the function signature matches the resulting stack layout
        assert!(sig.returns.len() <= u8::MAX as usize);
        self.stack.check_function_return_signature(sig.returns.len() as u8);
        assert!(imp.params.len() <= u8::MAX as usize);
        assert!(sig.params.len() <= u8::MAX as usize);

        for (v, p) in sig.params.iter().rev().enumerate() {
            if p.consumes {
                let target = ValueRef(v as u16);
                //discard it if allowed
                //returns the status of a stack elem without modifying anything
                if !self.stack.get_elem(target).can_be_freed() {
                    self.discard(target);
                }
            }
        }

        let offset = sig.params.len();
        for (v,p) in imp.params.iter().rev().enumerate() {
            if p.consumes {
                let target = ValueRef((offset + v) as u16);
                //discard it if allowed
                //returns the status of a stack elem without modifying anything
                if !self.stack.get_elem(target).can_be_freed() {
                    self.discard(target);
                }
            }
        }

        //todo: auto Discard consumes that have drop and are not consumed
        self.stack.check_function_param_signature(imp.params.len() as u16 + sig.params.len() as u16)
    }

    //Type checks a function in the current context
    pub fn type_check_function(&mut self, func:&FunctionComponent, code:&Exp) {
        //Capture transactional declaration
        self.transactional = func.shared.transactional;
        //Push the input parameters onto the stack
        for p in &func.shared.params {
            let typ =  p.typ.fetch(&self.context);
            //distinguish between owned and borrowed (read-only) parameters
            if p.consumes {
                self.stack.provide(typ);
            } else {
                self.stack.borrow(typ);
            }
        }

        //Start a new block for th body of the function
        let block = self.stack.start_block();
        //Type check the function body
        let rets = self.type_check_exp(code);
        //Type check the Result
        //Ensure the amount is correct
        assert!(rets as usize == func.shared.returns.len());
        //iterate over each return (deepest first)
        for (idx,t) in func.shared.returns.iter().rev().enumerate() {
            //Check if the returned value has the expected type
            let ret_typ = t.fetch(&self.context);
            assert!(idx <= u8::MAX as usize);
            assert!(self.stack.value_of(ValueRef(idx as u16)) == ret_typ);
        }

        //discard unneeded items
        self.clean_frame(rets, func.shared.params.len());
        //Close the function body block (leaves just params & results on the Stack)
        self.stack.end_block(block, rets);

        //Check that the function signature matches the resulting stack layout
        assert!(func.shared.returns.len() <= u8::MAX as usize);
        self.stack.check_function_return_signature(func.shared.returns.len() as u8);
        assert!(func.shared.params.len() <= u8::MAX as usize);

        for (v, p) in func.shared.params.iter().rev().enumerate() {
            if p.consumes {
                let target = ValueRef(v as u16);
                //discard it if allowed
                //returns the status of a stack elem without modifying anything
                if !self.stack.get_elem(target).can_be_freed() {
                    self.discard(target);
                }
            }
        }

        self.stack.check_function_param_signature(func.shared.params.len() as u16)
    }

    //Type checks an expression in the current context
    fn type_check_exp<'c>(&mut self, exp: &'c Exp) -> u8 {
        //This is done to prevent a stack overflow
        // Basically it expresses that functions with more than self.limit levels are invalid
        //increase the nesting size
        self.depth+=1;
        //check that we are not to deep
        assert!(self.depth <= self.limit);
        //catch the last size
        let mut rets = 0;
        //prepare the lock_holder
        let mut lock_holder:Vec<LockInfo> = Vec::new();
        //Type check the opcodes leading up to this Return
        for op in exp.0.iter() {
            rets = self.type_check_op_code(op, &mut lock_holder)
        }

        for lock in lock_holder {
            self.stack.unlock(lock)
        }

        for v in (0..(rets as u16)).rev() {
            //Get the Resolved Type of the source
            let r_typ = self.stack.value_of(ValueRef(v));
            //Return is only allowed for types with the Copy capability
            assert!(r_typ.get_caps().contains(Capability::Unbound));
        }

        //decrease the nesting size
        self.depth-=1;
        //return the selected elements from the block
        rets
    }

    //The heavy lifter that type checks op sys
    fn type_check_op_code(&mut self, code: &OpCode, lock_holder:&mut Vec<LockInfo>) -> u8 {
        //Branch on the opcode type and check it
        match *code {
            OpCode::Lit(ref data, perm) => self.lit(data, perm),
            OpCode::Let(ref bind) => self.let_(bind),
            OpCode::Copy(value) => self.fetch(value, FetchMode::Copy),
            OpCode::Move(value) => self.fetch(value, FetchMode::Consume),
            OpCode::Return(ref values) => self._return(values),
            OpCode::Discard(value) => self.discard(value),
            OpCode::DiscardMany(ref values) => self.discard_many(values),
            OpCode::InspectUnpack(value, perm) => self.unpack(value, perm, None, lock_holder),
            OpCode::Unpack(value, perm) => self.unpack(value, perm, Some(FetchMode::Consume), lock_holder),
            OpCode::CopyUnpack(value, perm) => self.unpack(value, perm, Some(FetchMode::Copy), lock_holder),
            OpCode::InspectSwitch(value, perm, ref cases) => self.switch(value, perm, cases, None),
            OpCode::Switch(value, perm, ref cases) => self.switch(value, perm, cases, Some(FetchMode::Consume)),
            OpCode::CopySwitch(value, perm, ref cases) => self.switch(value, perm, cases, Some(FetchMode::Copy)),
            OpCode::Pack(perm, tag, ref values) => self.pack(perm, tag, values, FetchMode::Consume),
            OpCode::CopyPack(perm, tag, ref values) => self.pack(perm, tag, values, FetchMode::Copy),
            OpCode::Invoke(perm, ref vals) => self.invoke(perm,vals, None),
            OpCode::TryInvoke(perm, ref vals, ref suc, ref fail) => self.try_invoke(perm,vals, suc, fail, None),
            OpCode::Field(value, perm, field) => self.field(value, perm, field, FetchMode::Consume),
            OpCode::CopyField(value, perm, field) => self.field(value, perm, field, FetchMode::Copy),
            OpCode::RollBack(ref consumes, ref produces) => self.rollback(consumes, produces),
            OpCode::Project(typ, val) =>  self.project(typ,val),
            OpCode::UnProject(typ, val) =>  self.un_project(typ, val),
            OpCode::InvokeSig(fun,perm,  ref vals) => self.invoke_sig(fun, perm, vals),
            OpCode::TryInvokeSig(fun,perm,  ref vals, ref suc, ref fail) => self.try_invoke_sig(fun, perm, vals, suc, fail),
            OpCode::RepeatedInvoke(_, perm, ref vals, cond, _) => self.invoke(perm,vals, Some(cond)),
            OpCode::RepeatedTryInvoke(_, perm, ref vals, cond, _, ref suc, ref fail)  => self.try_invoke(perm,vals, suc, fail, Some(cond))
        }
    }

    fn lit(&mut self, data:&LargeVec<u8>, perm:PermRef) -> u8 {
        //fetch the permission
        let r_perm = perm.fetch(&self.context);
        //check that it is of the right type
        assert!(r_perm.check_permission(Permission::Create));
        //get the literals size
        let size = r_perm.get_lit_size();
        //Check that tis type can be generated from the provided Byte stream
        assert!(size as usize == data.0.len());
        //Tell the Stack that an element has appeared out of nowhere
        self.stack.provide(r_perm.get_type());
        1
    }

    //_ as let is keyword
    fn let_(&mut self, bind:&Exp) -> u8 {
        //capture frame start for clean up
        let start_height = self.stack.stack_depth();
        //Tell the stack that a new scope has started
        let block = self.stack.start_block();
        //Type check the content of the Let
        let results = self.type_check_exp(bind);
        //discard unneeded items
        self.clean_frame(results,start_height);
        //Close the scope leaving only the results on the Stack
        self.stack.end_block(block,results);
        results
    }

    fn fetch(&mut self, value:ValueRef, mode:FetchMode) -> u8 {
        if mode == FetchMode::Copy {
            //Get the Resolved Type of the source
            let v_typ = self.stack.value_of(value);
            //Copy is only allowed for types with the Copy capability
            assert!(v_typ.get_caps().contains(Capability::Copy))
        }

        //Move or borrow the value to the top of the stack
        self.stack.fetch(value,mode);
        1
    }

    fn project(&mut self, typ:TypeRef, value:ValueRef) -> u8 {
        //get the input type
        let v_typ = self.stack.value_of(value);
        //the wrapped type
        let n_typ = typ.fetch(&self.context);

        if let ResolvedType::Projection{ depth, un_projected } = *v_typ {
            //check that it is of the right type
            assert!(get_target(n_typ) == un_projected && n_typ.get_projection_depth() == depth+1)
        } else if let ResolvedType::Projection{ depth, un_projected } = *n_typ {
            //check that it is of the right type
            assert!(un_projected == v_typ && depth == 1);
        } else {
            panic!("Specified type is not the projection");
        }

        //Copy the value on top with another type
        self.stack.transform(value, n_typ,FetchMode::Copy);
        1
    }

    fn un_project(&mut self, typ:TypeRef, value:ValueRef) -> u8 {
        //get the input type
        let v_typ = self.stack.value_of(value);
        //the wrapped type
        let n_typ = typ.fetch(&self.context);
        //check that it is a nested image
        match *v_typ  {
            ResolvedType::Projection {depth, un_projected, ..} => {
                assert!(if let ResolvedType::Projection{..} = *un_projected {false} else {true});
                //check that it is of the right type
                assert!(get_target(n_typ) == un_projected && n_typ.get_projection_depth() == depth-1);
                assert!(un_projected.get_caps().contains(Capability::Primitive));
                //Copy the value on top with another type
                self.stack.transform(value, n_typ,FetchMode::Copy);
                1

            }
            _ => panic!("Only projections can be un-projected")
        }

    }


    fn discard(&mut self, value:ValueRef) -> u8 {
        //Get the Resolved Type of the target
        let v_typ = self.stack.value_of(value);
        //Drop is only allowed for types with the Drop capability
        assert!(v_typ.get_caps().contains(Capability::Drop));
        //Tell the stack that the value is discarded so he can check the linearity constraints
        self.stack.drop(value);
        0
    }

    fn discard_many(&mut self, value:&[ValueRef]) -> u8 {
        for v in value {
            self.discard(*v);
        }
        0
    }

    fn unpack(&mut self, value:ValueRef, perm:PermRef, mode:Option<FetchMode>, lock_holder:&mut Vec<LockInfo>) -> u8 {
        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value);
        //get the perm
        let r_perm = perm.fetch(&self.context);
        //Calc the required permission
        let perm_type = match mode {
            Some(FetchMode::Consume) => Permission::Consume,
            Some(FetchMode::Copy) | None => Permission::Inspect,
        };

        //check that it is of the right type
        assert!(r_perm.check_value_permission(r_typ, perm_type));
        //fetch the ctr
        let r_ctr = r_perm.get_ctrs();
        //check that it is a valid unpack target
        assert!(r_ctr.len() == 1);

        //Get the resolved constructors
        if Some(FetchMode::Copy) == mode {
            //Copied values need the copy capability
            assert!(r_typ.get_caps().contains(Capability::Copy));
        }
        //Tell the stack to execute the operation (will take care of borrow vs consume)
        match mode {
            Some(m) => self.stack.unpack(value, &r_ctr[0],m),
            None => {
                lock_holder.push(self.stack.lock(value));
                self.stack.inspect(value, &r_ctr[0]);
            }
        }

        assert!(r_ctr[0].len() <= u8::MAX as usize);
        r_ctr[0].len() as u8
    }

    fn field(&mut self, value:ValueRef, perm:PermRef, field:u8, mode:FetchMode) -> u8 {
        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value);
        //get the perm
        let r_perm = perm.fetch(&self.context);
        //Calc the required permission
        let perm_type = match mode {
            FetchMode::Consume => Permission::Consume,
            FetchMode::Copy => Permission::Inspect,
        };

        //check that it is of the right type
        assert!(r_perm.check_value_permission(r_typ, perm_type));

        //Get the resolved constructors
        let r_ctr = r_perm.get_ctrs();

        //Field get is not defined for types with less then one field in a single ctr
        assert!(r_ctr.len() == 1);
        //get the value typ
        let typ = r_ctr[0usize][field as usize];

        if mode == FetchMode::Consume {
            //Non-fetched values need the drop capability
            for (idx,field_type) in r_ctr[0usize].iter().enumerate() {
                assert!(idx == field as usize || field_type.get_caps().contains(Capability::Drop));
            }
        } else {
            //fetched value needs the copy capability
            assert!(typ.get_caps().contains(Capability::Copy));
        }
        //Tell the stack to execute the operation (will take care of borrow vs consume)
        self.stack.field(value, typ, mode);
        1
    }


    //None is inspect
    fn switch(&mut self, value:ValueRef, perm:PermRef, cases:&[Exp], mode:Option<FetchMode>) -> u8 {
        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value);
        //get the perm
        let r_perm = perm.fetch(&self.context);

        //Calc the required permission
        let perm_type = match mode {
            Some(_) => Permission::Consume,
            None => Permission::Inspect,
        };

        //check that it is of the right type
        assert!(r_perm.check_value_permission(r_typ,perm_type));

        //Get the resolved constructors
        let r_ctr = r_perm.get_ctrs();

        //Check that their is exactly one case per potential constructor
        assert!(r_ctr.len() == cases.len());

        //check that we can copy if it is required
        if Some(FetchMode::Copy) == mode {
            //Copied values need the copy capability
            assert!(r_typ.get_caps().contains(Capability::Copy))
        }

        //capture frame start for clean up
        let start_height = self.stack.stack_depth();
        //just a helper to make the loop simpler -- represents the types from the previous case (loop iter)
        let mut loop_res:Option<u8> = None;
        //lock value in case of inspect
        let lock = match mode {
            None => Some(self.stack.lock(value)),
            Some(_) => None
        };
        //Tell the stack that a the control flow branches
        let mut branching = self.stack.start_branching(cases.len());
        //Process all the branches
        // Note: The stack ensures that each branch returns the same Elements (this includes their type)
        for (i,case) in cases.iter().enumerate() {
            //if this is not the first iter then tell the stack that the next branch will be processed (will restore stack)
            if let Some(res) = loop_res {
                //discard unneeded items
                self.clean_frame( res,start_height);
                //go to next branch
                self.stack.next_branch( &mut branching, res);
            }
            //Tell the stack to execute the operation
            match mode {
                Some(f_mode) => self.stack.unpack(value, &r_ctr[i],f_mode),
                None => self.stack.inspect(value, &r_ctr[i]),
            };

            //remaining operations are specified by branch sys and now type checked
            let res = self.type_check_exp(case);
            //pass intermediary result to next iter
            loop_res = Some(res);
        }
        //extract res
        let res = loop_res.unwrap();
        //discard unneeded items
        self.clean_frame( res,start_height);
        //unlock value in case of inspect
        match lock {
            Some(lock_info) => self.stack.unlock(lock_info),
            None => {}
        };
        //finish the branching, leaves the stack with the common elements
        self.stack.end_branching(branching, res);
        res
    }

    fn pack(&mut self, perm:PermRef, Tag(t):Tag, values:&[ValueRef], mode:FetchMode) -> u8 {
        //fetch the permission
        let r_perm = perm.fetch(&self.context);
        //check that it is of the right type
        assert!(r_perm.check_permission(Permission::Create));

        //Get the Resolved Constructors
        let r_ctr = r_perm.get_ctrs();

        //check if applicable
        assert!(r_ctr.len() != 0);

        //check that the case exists and has the right number of fields
        assert!((t as usize) < r_ctr.len() && r_ctr[t as usize].len() == values.len());

        //check that each param is ok
        for (i,v) in values.iter().enumerate() {
            //fetch the type of the param
            let r_v = self.stack.value_of(*v);

            //check that the value has the copy if required
            assert!(mode != FetchMode::Copy || r_v.get_caps().contains(Capability::Copy));

            //Check that the type of the param matches
            assert!(r_ctr[t as usize][i] == r_v);
        }
        //Tell the stack to pack the value and place the result onto the stack
        self.stack.pack(&values, r_perm.get_type(), mode);
        1
    }

    fn rollback(&mut self, consumes:&[ValueRef], produces:&[TypeRef]) -> u8 {
        //Consume all inputs
        for c in consumes {
            self.stack.drop(*c);
        }
        //Push all the produces
        for p in produces {
            self.stack.provide(p.fetch(&self.context));
        }
        assert!(produces.len() <= u8::MAX as usize);
        produces.len() as u8
    }

    fn invoke_sig(&mut self, value:ValueRef, perm:PermRef, vals:&[ValueRef]) -> u8 {
        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value);
        //fetch the permission
        let r_perm = perm.fetch(&self.context);
        //check that it is of the right type
        assert!(r_perm.check_value_permission(r_typ, Permission::Call));
        //Get the Resolved Signature of the call target
        let sig = r_perm.get_sig();
        //consume it
        self.stack.consume(value);
        //check the call
        self.invoke_direct( &sig, vals)
    }

    fn try_invoke_sig(&mut self, value:ValueRef, perm:PermRef, vals:&[(bool,ValueRef)], suc:&Exp, fail:&Exp) -> u8 {
        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value);
        //fetch the permission
        let r_perm = perm.fetch(&self.context);
        //check that it is of the right type
        assert!(r_perm.check_value_permission(r_typ, Permission::Call));

        //Get the Resolved Signature of the call target
        let sig = r_perm.get_sig();
        //consume it
        self.stack.consume(value);
        //check the call
        self.invoke_try(&sig, vals, suc, fail)
    }

    fn check_repetition_condition(sig:&ResolvedSignature, cond_arg:u8) {
        assert!(sig.params.len() == sig.returns.len());

        for (p,r) in sig.params.iter().zip(sig.returns.iter()) {
            assert!(p.typ == *r);
        }

        assert!((cond_arg as usize) < sig.returns.len());
        assert!(sig.returns[cond_arg as usize].is_data());

        //Note we do not check tag is in range of ctr
        // 1: This is easier and is more efficient as we do not need constructor cache
        // 2: It allows to provide an ot of range if their is no abort condition
    }

    fn invoke(&mut self, perm:PermRef, vals:&[ValueRef], rep_cond:Option<u8>) -> u8 {
        //fetch the permission
        let r_perm = perm.fetch(&self.context);
        //check that it is of the right type
        assert!(r_perm.check_permission(Permission::Call));
        //Get the fun sig
        let sig = r_perm.get_sig();

        //is this a repeated invoke?
        match rep_cond {
            //check the repetition condition
            Some(cond_arg) => {
                //check that it is not an implement (the are not repeated callable)
                if let ResolvedCallable::Implement { .. }  = *r_perm.get_fun() {
                    panic!("Signature generation can not be used over repeated Call");
                }
                Self::check_repetition_condition(&sig,cond_arg);
                assert!(self.transactional);
            },
            None => {}
        }

        //check the sig
        self.invoke_direct( &sig, vals)
    }

    fn try_invoke(&mut self, perm:PermRef, vals:&[(bool,ValueRef)], suc:&Exp, fail:&Exp, rep_cond:Option<u8>) -> u8 {
        //fetch the permission
        let r_perm = perm.fetch(&self.context);
        //check that it is of the right type
        assert!(r_perm.check_permission(Permission::Call));
        //check that it is not an implement (the are not try callabel)
        if let ResolvedCallable::Implement { .. }  = *r_perm.get_fun() {
            panic!("Signature generation can not be used over Try Call");
        }

        //Get the fun sig
        let sig = r_perm.get_sig();

        //is this a repeated invoke?
        match rep_cond {
            //check the repetition condition
            Some(cond_arg) => Self::check_repetition_condition(&sig,cond_arg),
            None => {}
        }

        //check the sig
        self.invoke_try(&sig, vals, suc, fail)
    }

    fn invoke_direct(&mut self, signature:&ResolvedSignature, vals:&[ValueRef]) -> u8 {
        //Check that the right amount of arguments are supplied for the call
        assert!(signature.params.len() == vals.len());

        assert!(!signature.transactional || self.transactional);
        //Prepare the Inputs
        let inputs:Vec<(ValueRef,bool)> = vals.iter().zip(signature.params.iter()).map(|(v,p)| {
            //Ensure tat the argument has the expected type
            assert!(self.stack.value_of(*v) == p.typ);
            (*v, p.consumes)
        }).collect();

        //consume the params for the call
        self.stack.consume_params(&inputs);
        //Advice the stack to produce the returns
        for ret in &signature.returns {
            self.stack.provide(*ret);
        }
        assert!(vals.len() <= u8::MAX as usize);
        signature.returns.len() as u8
    }

    fn invoke_try(&mut self, signature:&ResolvedSignature, vals:&[(bool, ValueRef)], succ:&Exp, fail:&Exp) -> u8 {
        //Check that the right amount of arguments are supplied for the call
        assert!(signature.params.len() == vals.len());
        assert!(signature.transactional);

        //Prepare the Inputs
        let inputs:Vec<(ValueRef,bool)> = vals.iter().zip(signature.params.iter()).map(|((essential,v),p)| {
            //Ensure that the argument has the expected type
            assert!(self.stack.value_of(*v) == p.typ);
            if *essential {
                assert!(p.consumes);
                assert!(p.typ.get_caps().contains(Capability::Value));
            } else {
                assert!(p.consumes || p.typ.get_caps().contains(Capability::Drop));
            }

            (*v, p.consumes)
        }).collect();

        //consume the params for the call
        self.stack.consume_params(&inputs);
        //capture frame start for clean up
        let start_height = self.stack.stack_depth();
        //start the branching for the success case
        let mut branching = self.stack.start_branching(2);
            //Produce the returns
            //Advice the stack to produce the returns
            for ret in &signature.returns {
                self.stack.provide(*ret);
            }
            //on success operations are specified by branch sys and now type checked
            let suc_res = self.type_check_exp(succ);
            //discard unneeded items
            self.clean_frame( suc_res,start_height);
        //go to the failure case branch
        self.stack.next_branch( &mut branching, suc_res);
            //Advice the stack to recover the essential params (the non essentials are implicitly dropped or where not consumed in the first place)
            for (_, param) in vals.iter().zip(signature.params.iter()).filter(|((e,_),_)|*e) {
                self.stack.provide(param.typ);
            }
            //on failure operations are specified by branch sys and now type checked
            let fail_res = self.type_check_exp(fail);
            //discard unneeded items
            self.clean_frame( fail_res,start_height);
        //end the branch
        self.stack.end_branching(branching, fail_res);
        fail_res
    }

    fn _return(&mut self, vals:&[ValueRef]) -> u8 {
        //Consume the Inputs
        for (i,ValueRef(idx)) in vals.iter().enumerate() {
            //push it on top (the +i counteracts the already pushed ones)
            assert!(*idx as usize + i <= u16::MAX as usize);
            self.stack.fetch(ValueRef(idx+i as u16), FetchMode::Consume);
        }
        assert!(vals.len() <= u8::MAX as usize);
        vals.len() as u8
    }
}
