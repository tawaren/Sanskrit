//! A type checker that walks along the ast and checks if the code is sound
//!
//! The type checker is responsible to check that the inputs to the different Opcodes, as well as the parameters to function calls have the right types.
//!  It further enforces that the needed capabilities are present for restricted opcodes like `Copy`, `Drop`, `Unpack`, ...
//!  Beside checking types it needs to ensure that borrowed values are only used in positions where they are allowed
//!
//! For tracking the types of the values the `linear_type_stack` is heavily used as well as the `native` module to check that the corresponding types are used correctly.
//! After this module checked an Ast without generating an error the types are sound and further step can ignore them, but the information can not be discarded as it is needed by future type checking runs of other code linking to the current code
//! Where later phases need some type information like the amount of constructor parameters of a type it is already present in the input code and its consistency is checked by the type checker
//!  This approach ensures that later phases to not have to load and parse types later on but only code for functions.

use linear_stack::*;
use sanskrit_core::model::*;
use sanskrit_core::model::linking::*;
use sanskrit_core::model::resolved::*;
use sanskrit_core::model::bitsets::*;
use sanskrit_common::errors::*;
use sanskrit_common::store::Store;
use alloc::vec::Vec;
use sanskrit_core::resolver::Context;
use sanskrit_common::model::*;
use sanskrit_core::utils::Crc;
use sanskrit_core::accounting::Accounting;

pub struct TypeCheckerContext<'b, S:Store + 'b> {
    context: Context<'b, S>,                     //The Resolved Components from the input
    stack: LinearStack<'b, Crc<ResolvedType>>,   //The current stack layout
    transactional:bool,
    depth:usize,
    limit:usize,
}

impl<'b, S:Store + 'b> TypeCheckerContext<'b,S> {
    //Creates a new Empty context
    pub fn new(accounting:&'b Accounting, context: Context<'b, S>) -> Self {
        //Define some reused types and capabilities
        TypeCheckerContext {
            context,
            stack: LinearStack::new(accounting),
            transactional: false,
            depth: 0,
            limit: accounting.nesting_limit
        }
    }

    //Note: I would love to do here steals as well but the algorithm is complicated
    //      And I want to keep the on chain verifier simple
    fn clean_frame(&mut self, results:u8, start:usize) -> Result<()> {
        //how many values are on the stack
        let frame_size = self.stack.stack_depth() - start;

        //is the stack to big?
        if frame_size > u16::max_value() as usize {
            return error(||"Frame size exceeded maximum allowed size")
        }

        assert!(results as usize <= frame_size);
        //check every not returned item on the stack (start with the last one to clean bottom up)
        for v in (results as u16)..(frame_size as u16) {
            let target = ValueRef(v);
            //discard it if allowed
            //returns the status of a stack elem without modifying anything
            if !self.stack.get_elem(target)?.can_be_freed() {
                self.discard(target)?;
            }
        }

        //all fine
        Ok(())
    }

    //todo: I hate this duplication but the signtures are different and unification is hard
    pub fn type_check_implement(&mut self, imp:&ImplementComponent, code:&Exp) -> Result<()>{
        //Fetch the Permission
        let r_perm = imp.sig.fetch(&self.context)?;
        //Check that it is the correct one
        if !r_perm.check_permission(Permission::Implement) {
            //expected type does not much provided type
            return error(||"Wrong Permission supplied")
        }
        //get the signature
        let sig = r_perm.get_sig()?;
        // num params
        let num_params = imp.params.len()+ sig.params.len();
        //Push the capt parameters onto the stack
        for c in &imp.params {
            //captures are always owned
            self.stack.provide(c.typ.fetch(&self.context)?)?;
        }
        //Push the provided parameters onto the stack
        for c in &sig.params {
            //distinguish between owned and borrowed (read-only) parameters
            if c.consumes {
                self.stack.provide(c.typ.clone())?;
            } else {
                self.stack.borrow(c.typ.clone())?;
            }
        }

        //Start a new block for th body of the function
        let block = self.stack.start_block();
        //Type check the function body
        let rets = self.type_check_exp(code)?;

        //Type check the Result
        //Ensure the amount is correct
        if rets as usize != sig.returns.len() {
            return error(||"Number of returned values mismatches number of returned values in the signature declaration")
        }
        //iterate over each return (deepest first)
        for (v,t) in sig.returns.iter().rev().enumerate() {
            //Check if the returned value has the expected type
            assert!(v <= u8::max_value() as usize);
            if self.stack.value_of(ValueRef(v as u16))? != *t {
                return error(||"Returned value has different type from return type declaration of the signature")
            }
        }

        //discard unneeded items
        self.clean_frame(rets, num_params)?;
        //Close the function body block (leaves just params & results on the Stack)
        self.stack.end_block(block, rets)?;

        //Check that the function signature matches the resulting stack layout
        assert!(sig.returns.len() <= u8::max_value() as usize);
        self.stack.check_function_return_signature(sig.returns.len() as u8)?;
        assert!(imp.params.len() <= u8::max_value() as usize);
        assert!(sig.params.len() <= u8::max_value() as usize);
        self.stack.check_function_param_signature(imp.params.len() as u16 + sig.params.len() as u16, false)
    }

    //Type checks a function in the current context
    pub fn type_check_function(&mut self, func:&FunctionComponent, code:&Exp) -> Result<()>{
        //Capture transactional declaration
        self.transactional = func.shared.transactional;
        //Push the input parameters onto the stack
        for p in &func.shared.params {
            let typ =  p.typ.fetch(&self.context)?;
            //distinguish between owned and borrowed (read-only) parameters
            if p.consumes {
                self.stack.provide(typ)?;
            } else {
                self.stack.borrow(typ)?;
            }
        }

        //Start a new block for th body of the function
        let block = self.stack.start_block();
        //Type check the function body
        let rets = self.type_check_exp(code)?;
        //Type check the Result
        //Ensure the amount is correct
        if rets as usize != func.shared.returns.len() {
            return error(||"Number of returned values mismatches number of returned values in the function declaration")
        }
        //iterate over each return (deepest first)
        for (idx,t) in func.shared.returns.iter().rev().enumerate() {
            //Check if the returned value has the expected type
            let ret_typ = t.fetch(&self.context)?;
            assert!(idx <= u8::max_value() as usize);
            if self.stack.value_of(ValueRef(idx as u16))? != ret_typ {
                return error(||"Returned value has different type from return type declaration of function")
            }
        }

        //discard unneeded items
        self.clean_frame(rets, func.shared.params.len())?;
        //Close the function body block (leaves just params & results on the Stack)
        self.stack.end_block(block, rets)?;

        //Check that the function signature matches the resulting stack layout
        assert!(func.shared.returns.len() <= u8::max_value() as usize);
        self.stack.check_function_return_signature(func.shared.returns.len() as u8)?;
        assert!(func.shared.params.len() <= u8::max_value() as usize);
        self.stack.check_function_param_signature(func.shared.params.len() as u16, false)
    }

    //Type checks an expression in the current context
    fn type_check_exp<'c>(&mut self, exp: &'c Exp) -> Result<u8> {
        //This is done to prevent a stack overflow
        // Basically it expresses that functions with more than self.limit levels are invalid
        //increase the nesting size
        self.depth+=1;
        //check that we are not to deep
        if self.depth > self.limit {
            return error(||"Limit for block nesting reached")
        }
        //catch the last size
        let mut rets = 0;
        //Type check the opcodes leading up to this Return
        for op in exp.0.iter() {
            rets = self.type_check_op_code(op)?
        }

        for v in (0..(rets as u16)).rev() {
            //Get the Resolved Type of the source
            let r_typ = self.stack.value_of(ValueRef(v))?;
            //Return is only allowed for types with the Copy capability
            if !r_typ.get_caps().contains(Capability::Unbound) {
                return error(||"Returning a value requires the unbound capability")
            }
        }

        //decrease the nesting size
        self.depth-=1;
        //return the selected elements from the block
        Ok(rets)
    }

    //The heavy lifter that type checks op code
    fn type_check_op_code(&mut self, code: &OpCode) -> Result<u8> {
        //Branch on the opcode type and check it
        match *code {
            OpCode::Lit(ref data, perm) => self.lit(data, perm),
            OpCode::Let(ref bind) => self.let_(bind),
            OpCode::Copy(value) => self.fetch(value, FetchMode::Copy),
            OpCode::Move(value) => self.fetch(value, FetchMode::Consume),
            OpCode::Return(ref values) => self._return(values),
            OpCode::Discard(value) => self.discard(value),
            OpCode::DiscardMany(ref values) => self.discard_many(values),
            OpCode::Unpack(value, perm) => self.unpack(value, perm, FetchMode::Consume),
            OpCode::CopyUnpack(value, perm) => self.unpack(value, perm, FetchMode::Copy),
            OpCode::Inspect(value, perm, ref cases) => self.switch(value, perm, cases, None),
            OpCode::Switch(value, perm, ref cases) => self.switch(value, perm, cases, Some(FetchMode::Consume)),
            OpCode::CopySwitch(value, perm, ref cases) => self.switch(value, perm, cases, Some(FetchMode::Copy)),
            OpCode::Pack(perm, tag, ref values) => self.pack(perm, tag, values, FetchMode::Consume),
            OpCode::CopyPack(perm, tag, ref values) => self.pack(perm, tag, values, FetchMode::Copy),
            OpCode::Invoke(perm, ref vals) => self.invoke(perm,vals),
            OpCode::TryInvoke(perm, ref vals, ref suc, ref fail) => self.try_invoke(perm,vals, suc, fail),
            OpCode::Field(value, perm, field) => self.field(value, perm, field, FetchMode::Consume),
            OpCode::CopyField(value, perm, field) => self.field(value, perm, field, FetchMode::Copy),
            OpCode::RollBack(ref consumes, ref produces) => self.rollback(consumes, produces),
            OpCode::Project(typ, val) =>  self.project(typ,val),
            OpCode::UnProject(val) =>  self.un_project(val),
            OpCode::InvokeSig(fun,perm,  ref vals) => self.invoke_sig(fun, perm, vals),
            OpCode::TryInvokeSig(fun,perm,  ref vals, ref suc, ref fail) => self.try_invoke_sig(fun, perm, vals, suc, fail)

        }
    }

    fn lit(&mut self, data:&LargeVec<u8>, perm:PermRef) -> Result<u8> {
        //fetch the permission
        let r_perm = perm.fetch(&self.context)?;
        //check that it is of the right type
        if !r_perm.check_permission(Permission::Create) {
            //expected type does not much provided type
            return error(||"Wrong Permission supplied")
        }
        //get the literals size
        let size = r_perm.get_lit_size()?;
        //Check that tis type can be generated from the provided Byte stream
        if size as usize != data.0.len() {
            return error(||"Supplied byte stream has wrong size for literal construction")
        }
        //Tell the Stack that an element has appeared out of nowhere
        self.stack.provide(r_perm.get_type()?.clone())?;
        Ok(1)
    }

    //_ as let is keyword
    fn let_(&mut self, bind:&Exp) -> Result<u8> {
        //capture frame start for clean up
        let start_height = self.stack.stack_depth();
        //Tell the stack that a new scope has started
        let block = self.stack.start_block();
        //Type check the content of the Let
        let results = self.type_check_exp(bind)?;
        //discard unneeded items
        self.clean_frame(results,start_height)?;
        //Close the scope leaving only the results on the Stack
        self.stack.end_block(block,results)?;
        Ok(results)
    }

    fn fetch(&mut self, value:ValueRef, mode:FetchMode) -> Result<u8> {
        if mode == FetchMode::Copy {
            //Get the Resolved Type of the source
            let v_typ = self.stack.value_of(value)?;
            //Copy is only allowed for types with the Copy capability
            if !v_typ.get_caps().contains(Capability::Copy) {
                return error(||"Copy requires copy capability for input")
            }
        }

        //Move or borrow the value to the top of the stack
        self.stack.fetch(value,mode)?;
        Ok(1)
    }

    fn project(&mut self, typ:TypeRef, value:ValueRef) -> Result<u8> {
        //get the input type
        let v_typ = self.stack.value_of(value)?;
        //wrap the type into an image
        let n_typ = typ.fetch(&self.context)?;
        // project(project(x)) = project(x)
        if let ResolvedType::Projection{ .. } = *v_typ {
            //check that it is of the right type
            if n_typ != v_typ {
                return error(||"Specified type mismatches input type")
            }
        } else if let ResolvedType::Projection{ ref un_projected, .. } = *n_typ {
            //check that it is of the right type
            if un_projected != &v_typ {
                return error(||"Specified type mismatches input type")
            }
        } else {
            return error(||"Specified type is not the projection")
        }

        //Copy the value on top with another type
        self.stack.transform(value, n_typ,FetchMode::Copy)?;
        Ok(1)
    }

    fn un_project(&mut self, value:ValueRef) -> Result<u8> {
        //get the input type
        let v_typ = self.stack.value_of(value)?;
        //check that it is a nested image
        match *v_typ  {
            ResolvedType::Projection {ref un_projected, ..} => {
                assert!(if let ResolvedType::Projection{..} = **un_projected {false} else {true});
                if !un_projected.get_caps().contains(Capability::Primitive) {
                    error(||"Un-project requires primitive capability for output")
                } else {
                    //Copy the value on top with another type
                    self.stack.transform(value, un_projected.clone(),FetchMode::Copy)?;
                    Ok(1)
                }
            }
            _ => error(||"Only projections can be un-projected")
        }

    }


    fn discard(&mut self, value:ValueRef) -> Result<u8> {
        //Get the Resolved Type of the target
        let v_typ = self.stack.value_of(value)?;
        //Drop is only allowed for types with the Drop capability
        if !v_typ.get_caps().contains(Capability::Drop) {
            return error(||"Discard requires drop capability for input")
        }
        //Tell the stack that the value is discarded so he can check the linearity constraints
        self.stack.drop(value)?;
        Ok(0)
    }

    fn discard_many(&mut self, value:&[ValueRef]) -> Result<u8> {
        for v in value {
            self.discard(*v)?;
        }
        Ok(0)
    }

    fn unpack(&mut self, value:ValueRef, perm:PermRef, mode:FetchMode) -> Result<u8> {
        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value)?;
        //get the perm
        let r_perm = perm.fetch(&self.context)?;
        //check that it is of the right type
        if !r_perm.check_value_permission(&r_typ, Permission::Consume) {
            //expected type does not much provided type
            return error(||"Wrong Permission supplied")
        }
        //fetch the ctr
        let r_ctr = r_perm.get_ctrs()?;
        //check that it is a valid unpack target
        if r_ctr.len() != 1 {
            return error(||"Unpack must target a data type with a single constructor")
        };

        //Get the resolved constructors
        if FetchMode::Copy == mode {
            //Copied values need the copy capability
            if !r_typ.get_caps().contains(Capability::Copy) {
                return error(||"Copy unpack requires copy capability for input")
            }
        }
        //Tell the stack to execute the operation (will take care of borrow vs consume)
        self.stack.unpack(value, &r_ctr[0],mode)?;
        assert!(r_ctr[0].len() <= u8::max_value() as usize);
        Ok(r_ctr[0].len() as u8)
    }

    fn field(&mut self, value:ValueRef, perm:PermRef, field:u8, mode:FetchMode) -> Result<u8> {
        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value)?;
        //get the perm
        let r_perm = perm.fetch(&self.context)?;
        //Calc the required permission
        let perm_type = match mode {
            FetchMode::Consume => Permission::Consume,
            FetchMode::Copy => Permission::Inspect,
        };

        //check that it is of the right type
        if !r_perm.check_value_permission(&r_typ, perm_type) {
            //expected type does not much provided type
            return error(||"Wrong Permission supplied")
        }

        //Get the resolved constructors
        let r_ctr = r_perm.get_ctrs()?;

        //Field get is not defined for types with less then one field in a single ctr
        if r_ctr.len() == 0{
            return error(||"Requested field does not exist")
        };

        //get the value typ
        let typ = r_ctr[0 as usize][field as usize].clone();

        if mode != FetchMode::Consume {
            //Non-fetched values need the drop capability
            for (idx,field_type) in r_ctr[0 as usize].iter().enumerate() {
                if idx != field as usize && !field_type.get_caps().contains(Capability::Drop) {
                    return error(||"Consume field requires drop capability for not accessed fields")
                }
            }
        } else {
            //value needs the copy capability
            if !typ.get_caps().contains(Capability::Copy) {
                return error(||"Copy field requires copy capability for accessed field")
            }
        }
        //Tell the stack to execute the operation (will take care of borrow vs consume)
        self.stack.field(value, typ, mode)?;
        Ok(1)
    }


    //None is inspect
    fn switch(&mut self, value:ValueRef, perm:PermRef, cases:&[Exp], mode:Option<FetchMode>) -> Result<u8> {
        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value)?;
        //get the perm
        let r_perm = perm.fetch(&self.context)?;

        //Calc the required permission
        let perm_type = match mode {
            Some(_) => Permission::Consume,
            None => Permission::Inspect,
        };

        //check that it is of the right type
        if !r_perm.check_value_permission(&r_typ,perm_type) {
            //expected type does not much provided type
            return error(||"Wrong Permission supplied")
        }

        //Get the resolved constructors
        let r_ctr = r_perm.get_ctrs()?;

        //Check that their is exactly one case per potential constructor
        if r_ctr.len() != cases.len() {
            return error(||"Requested constructor does not exist")
        };

        //check that we can copy if it is required
        if Some(FetchMode::Copy) == mode {
            //Copied values need the copy capability
            if !r_typ.get_caps().contains(Capability::Copy) {
                return error(||"Copy unpack requires copy capability for input")
            }
        }

        //capture frame start for clean up
        let start_height = self.stack.stack_depth();
        //just a helper to make the loop simpler -- represents the types from the previous case (loop iter)
        let mut loop_res:Option<u8> = None;
        //Tell the stack that a the control flow branches
        let mut branching = match mode {
            Some(_) => self.stack.start_branching(cases.len()),
            None => self.stack.start_locked_branching( cases.len(),value)?
        };
        //Process all the branches
        // Note: The stack ensures that each branch returns the same Elements (this includes their type)
        for (i,case) in cases.iter().enumerate() {
            //if this is not the first iter then tell the stack that the next branch will be processed (will restore stack)
            if let Some(res) = loop_res {
                //discard unneeded items
                self.clean_frame( res,start_height)?;
                //go to next branch
                self.stack.next_branch( &mut branching, res)?;
            }
            //Tell the stack to execute the operation
            match mode {
                Some(f_mode) => self.stack.unpack(value, &r_ctr[i],f_mode)?,
                None => self.stack.inspect(value, &r_ctr[i])?,
            };

            //remaining operations are specified by branch code and now type checked
            let res = self.type_check_exp(case)?;
            //pass intermediary result to next iter
            loop_res = Some(res);
        }
        //extract res
        let res = loop_res.unwrap();
        //discard unneeded items
        self.clean_frame( res,start_height)?;
        //finish the branching, leaves the stack with the common elements
        self.stack.end_branching(branching, res)?;
        Ok(res)
    }

    fn pack(&mut self, perm:PermRef, Tag(t):Tag, values:&[ValueRef], mode:FetchMode) -> Result<u8> {
        //fetch the permission
        let r_perm = perm.fetch(&self.context)?;
        //check that it is of the right type
        if !r_perm.check_permission(Permission::Create) {
            //expected type does not much provided type
            return error(||"Wrong Permission supplied")
        }

        //Get the Resolved Constructors
        let r_ctr = r_perm.get_ctrs()?;

        //check if applicable
        if r_ctr.len() == 0 {
            return error(||"Requested constructor does not exist")
        }

        //check that the case exists and has the right number of fields
        if (t as usize) >= r_ctr.len() || r_ctr[t as usize].len() != values.len() {
            return error(||"Requested constructor does not exist")
        }

        //check that each param is ok
        for (i,v) in values.iter().enumerate() {
            //fetch the type of the param
            let r_v = self.stack.value_of(*v)?;

            //check that the value has the copy if required
            if mode == FetchMode::Copy && !r_v.get_caps().contains(Capability::Copy){
                return error(||"Copy pack requires copy capability for each constructor parameter")
            }

            //Check that the type of the param matches
            if r_ctr[t as usize][i] != r_v {
                return error(||"Parameter for data constructor has wrong type")
            }
        }
        //Tell the stack to pack the value and place the result onto the stack
        self.stack.pack(&values, r_perm.get_type()?.clone(), mode)?;
        Ok(1)
    }

    fn rollback(&mut self, consumes:&[ValueRef], produces:&[TypeRef]) -> Result<u8> {
        //Consume all inputs
        for c in consumes {
            self.stack.drop(*c)?;
        }
        //Push all the produces
        for p in produces {
            self.stack.provide(p.fetch(&self.context)?)?;
        }
        assert!(produces.len() <= u8::max_value() as usize);
        Ok(produces.len() as u8)
    }

    fn invoke_sig(&mut self, value:ValueRef, perm:PermRef, vals:&[ValueRef]) -> Result<u8> {
        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value)?;
        //fetch the permission
        let r_perm = perm.fetch(&self.context)?;
        //check that it is of the right type
        if !r_perm.check_value_permission(&r_typ, Permission::Call) {
            //expected type does not much provided type
            return error(||"Wrong Permission supplied")
        }

        //Get the Resolved Signature of the call target
        let sig = r_perm.get_sig()?;

        //consume it
        self.stack.consume(value)?;
        //check the call
        self.invoke_direct( &sig, vals)
    }

    fn try_invoke_sig(&mut self, value:ValueRef, perm:PermRef, vals:&[(bool,ValueRef)], suc:&Exp, fail:&Exp) -> Result<u8> {
        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value)?;
        //fetch the permission
        let r_perm = perm.fetch(&self.context)?;
        //check that it is of the right type
        if !r_perm.check_value_permission(&r_typ, Permission::Call) {
            //expected type does not much provided type
            return error(||"Wrong Permission supplied")
        }

        //Get the Resolved Signature of the call target
        let sig = r_perm.get_sig()?;
        //consume it
        self.stack.consume(value)?;
        //check the call
        self.invoke_try(&sig, vals, suc, fail)
    }

    fn invoke(&mut self, perm:PermRef, vals:&[ValueRef]) -> Result<u8> {
        //fetch the permission
        let r_perm = perm.fetch(&self.context)?;
        //check that it is of the right type
        if !r_perm.check_permission(Permission::Call) {
            //expected type does not much provided type
            return error(||"Wrong Permission supplied")
        }
        //Get the fun sig
        let sig = r_perm.get_sig()?;
        //check the sig
        self.invoke_direct( &sig, vals)
    }

    fn try_invoke(&mut self, perm:PermRef, vals:&[(bool,ValueRef)], suc:&Exp, fail:&Exp) -> Result<u8> {
        //fetch the permission
        let r_perm = perm.fetch(&self.context)?;
        //check that it is of the right type
        if !r_perm.check_permission(Permission::Call) {
            //expected type does not much provided type
            return error(||"Wrong Permission supplied")
        }
        //check that it is not an implement (the are not try callabel)
        if let ResolvedCallable::Implement { .. }  = **r_perm.get_fun()? {
            return error(||"Signature generation can not be used over Try Call")
        }

        //Get the fun sig
        let sig = r_perm.get_sig()?;
        //check the sig
        self.invoke_try(&sig, vals, suc, fail)
    }

    fn invoke_direct(&mut self, signature:&ResolvedSignature, vals:&[ValueRef]) -> Result<u8> {
        //Check that the right amount of arguments are supplied for the call
        if signature.params.len() != vals.len() {
            return error(||"Wrong number of parameter for function call")
        }

        if signature.transactional && !self.transactional {
            return error(||"Transactional functions must be called with a try invoke or inside another transactional function")
        }

        //Prepare the Inputs
        let inputs:Vec<(ValueRef,bool)> = vals.iter().zip(signature.params.iter()).map(|(v,p)| {
            //Ensure tat the argument has the expected type
            if self.stack.value_of(*v)? != p.typ {
                error(||"Parameter for function call has wrong type")
            } else {
                Ok((*v, p.consumes))
            }
        }).collect::<Result<_>>()?;

        //consume the params for the call
        self.stack.consume_params(&inputs)?;
        //Advice the stack to produce the returns
        for ret in &signature.returns {
            self.stack.provide(ret.clone())?;
        }
        assert!(vals.len() <= u8::max_value() as usize);
        Ok(signature.returns.len() as u8)
    }

    fn invoke_try(&mut self, signature:&ResolvedSignature, vals:&[(bool, ValueRef)], succ:&Exp, fail:&Exp) -> Result<u8> {
        //Check that the right amount of arguments are supplied for the call
        if signature.params.len() != vals.len() {
            return error(||"Wrong number of parameter for function call")
        }

        if !signature.transactional{
            return error(||"Only transactional functions can be used with try invoke")
        }

        //Prepare the Inputs
        let inputs:Vec<(ValueRef,bool)> = vals.iter().zip(signature.params.iter()).map(|((essential,v),p)| {
            //Ensure tat the argument has the expected type
            if self.stack.value_of(*v)? != p.typ {
                return error(||"Parameter for function call has wrong type")
            }
            if *essential {
                if !p.consumes {
                    return error(||"Only consumed params can be returned on a failure")
                }

                if !p.typ.get_caps().contains(Capability::Value) {
                    return error(||"Only Value params can be returned on a failure")
                }
            } else if !p.consumes && !p.typ.get_caps().contains(Capability::Drop){
                return error(||"Consumed params must be returned on a failure or be dropped")
            }

            Ok((*v, p.consumes))
        }).collect::<Result<_>>()?;

        //consume the params for the call
        self.stack.consume_params(&inputs)?;
        //capture frame start for clean up
        let start_height = self.stack.stack_depth();
        //start the branching for the success case
        let mut branching = self.stack.start_branching(2);
            //Produce the returns
            //Advice the stack to produce the returns
            for ret in &signature.returns {
                self.stack.provide(ret.clone())?;
            }
            //on success operations are specified by branch code and now type checked
            let suc_res = self.type_check_exp(succ)?;
            //discard unneeded items
            self.clean_frame( suc_res,start_height)?;
        //go to the failure case branch
        self.stack.next_branch( &mut branching, suc_res)?;
            //Advice the stack to recover the essential params (the non essentials are implicitly dropped or where not consumed in the first place)
            for (_, param) in vals.iter().zip(signature.params.iter()).filter(|((e,_),_)|*e) {
                self.stack.provide(param.typ.clone())?;
            }
            //on failure operations are specified by branch code and now type checked
            let fail_res = self.type_check_exp(fail)?;
            //discard unneeded items
            self.clean_frame( fail_res,start_height)?;
        //end the branch
        self.stack.end_branching(branching, fail_res)?;
        Ok(fail_res)
    }

    fn _return(&mut self, vals:&[ValueRef]) -> Result<u8> {
        //Consume the Inputs
        for (i,ValueRef(idx)) in vals.iter().enumerate() {
            //push it on top (the +i counteracts the already pushed ones)
            if *idx as usize + i > u16::max_value() as usize {
                return error(||"Size limit reached")
            }
            self.stack.fetch(ValueRef(idx+i as u16), FetchMode::Consume)?;
        }
        assert!(vals.len() <= u8::max_value() as usize);
        Ok(vals.len() as u8)
    }
}
