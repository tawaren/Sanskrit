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

use linear_type_stack::*;
use sanskrit_common::linear_stack::*;
use sanskrit_core::model::*;
use sanskrit_core::model::linking::*;
use sanskrit_core::model::resolved::*;
use sanskrit_common::errors::*;
use core::mem;
use alloc::collections::BTreeSet;
use alloc::rc::Rc;
use sanskrit_common::store::Store;
use alloc::prelude::*;
use sanskrit_core::resolver::Context;
use sanskrit_common::model::*;
use native::check_valid_literal_construction;
use sanskrit_core::native::base::resolved_native_type;
use sanskrit_core::utils::Crc;


pub struct TypeCheckerContext<'a,'b, S:Store + 'b> {
    context: Context<'a,'b, S>,                 //The Resolved Components from the input
    risk: BTreeSet<Rc<ResolvedErr>>,            //The currently catchable Risks
    stack: LinearTypeStack,                     //The current stack layout
}

impl<'a, 'b, S:Store + 'b> TypeCheckerContext<'a,'b,S> {
    //Creates a new Empty context
    pub fn new(context: Context<'a, 'b, S>) -> Self {
        //Define some reused types and capabilities
        TypeCheckerContext {
            context,
            risk: BTreeSet::new(),
            stack: LinearTypeStack::new(),
        }
    }

    fn clean_frame(&mut self, results:&ExprResult, start:usize) -> Result<()> {
        //prepare a set of the values for predictable cheaper lookup costs
        let mut keep_set = BTreeSet::new();
        match results {
            //if it is a return we have to check that the result types matches the function signature types
            ExprResult::Return(rets) => {
                for k in rets.iter() {
                    keep_set.insert(*k);
                }
            }
            ExprResult::Throw => {}
        }

        //how many values are on the stack
        let frame_size = self.stack.stack_depth() - start;

        //is the stack to big?
        if frame_size > u16::max_value() as usize {
            return size_limit_exceeded_error()
        }

        //check every item on the stack
        for v in 0..(frame_size as u16) {
            let target = ValueRef(v);
            //if it is not returned try to discard it
            if !keep_set.contains(&target) && !self.stack.can_be_released(target)? {
                self.discard(target)?;
            }
        }

        //all fine
        Ok(())
    }

    //Type checks a function in the current context
    pub fn type_check_function(&mut self, func:&FunctionComponent) -> Result<()>{
        //Capture risks declaration: Is later used to check if a throwing operation can be called
        self.risk = func.risk.iter().map(|r|r.fetch(&self.context)).collect::<Result<_>>()?;
        //Push the input parameters onto the stack
        for p in &func.params {
            self.stack.provide( p.typ.fetch(&self.context)?)?;
        }

        //Start a new block for th body of the function
        let block = self.stack.start_block();
        //Type check the function body
        let results = self.type_check_exp(&func.code)?;
        //Type check the Result
        match results {
            //if it is a return we have to check that the result types matches the function signature types
            ExprResult::Return(rets) => {
                //Ensure the amount is correct
                if rets.len() != func.returns.len() {return num_return_mismatch()}
                //iterate over each return (deepest first)
                for (val,t) in rets.iter().zip(func.returns.iter()) {
                    //Check if the returned value has the expected type
                    let ret_typ = t.typ.fetch(&self.context)?;
                    if self.stack.value_of(*val)? != ret_typ {return type_mismatch()}
                }

                //discard unneeded items
                self.clean_frame(&results, func.params.len())?;
                //Close the function body block (leaves just params & results on the Stack)
                self.stack.end_block(block, results)?;

                //Check that the function signature matches the resulting stack layout
                let consumes = func.params.iter().map(|p|p.consumes);
                let borrows = func.returns.iter().map(|r|&r.borrows[..]);
                self.stack.check_function_signature(consumes, borrows)
            },
            //Probably rarely used but supported (Function that always fails)
            ExprResult::Throw => {
                //discard unneeded items
                self.clean_frame(&results, func.params.len())?;
                //Close the function body block (leaves just params & results on the Stack)
                self.stack.end_block(block, ExprResult::Throw)?;

                //Check that the function params matches the resulting stack layout
                let consumes = func.params.iter().map(|p|p.consumes);
                self.stack.check_function_param_signature(consumes)
            },
        }
    }

    //Type checks an expression in the current context
    fn type_check_exp<'c>(&mut self, exp: &'c Exp) -> Result<ExprResult<'c>> {

        //Find out if the expression returns a result or a failure
        match *exp {
            //If a return process the blocks opcode
            Exp::Ret(ref op_seq, ref vals/*, ref drops*/) => {
                //Type check the opcodes leading up to this Return
                for op in &op_seq.0 {
                    self.type_check_op_code(op)?
                }
                //return the selected elements from the block
                Ok(ExprResult::Return(&vals/*, &drops*/))
            },

            //If a throw check that it is declared
            Exp::Throw(code) => {
                // resolve the error
                let err = code.fetch(&self.context)?;
                //Check if the parent scope can handle/expects the error
                if !self.risk.contains(&err) {
                    //Error is not declared
                    return risk_missing()
                }
                // Signal that this block produces an error
                Ok(ExprResult::Throw)
            },
        }
    }

    //The heavy lifter that type checks op code
    fn type_check_op_code(&mut self, code: &OpCode) -> Result<()> {
        //Branch on the opcode type and check it
        match *code {
            OpCode::Lit(ref data, typ) => self.lit(data, typ),
            OpCode::Let(ref bind) => self.let_(bind),
            OpCode::CopyFetch(value) => self.fetch(value, FetchMode::Copy),
            OpCode::Fetch(value) => self.fetch(value, FetchMode::Consume),
            OpCode::BorrowFetch(value) => self.fetch(value, FetchMode::Borrow),
            OpCode::Discard(value) => self.discard(value),
            OpCode::DiscardMany(ref values) => self.discard_many(values),
            OpCode::DiscardBorrowed(value, ref borrows) => self.discard_borrowed(value, borrows),
            OpCode::BorrowUnpack(value, base_ref) => self.unpack(value, base_ref, FetchMode::Borrow, None),
            OpCode::Unpack(value, base_ref) => self.unpack(value, base_ref, FetchMode::Consume, None),
            OpCode::CopyUnpack(value, base_ref) => self.unpack(value, base_ref, FetchMode::Copy, None),
            OpCode::BorrowSwitch(value, typ, ref cases) => self.switch(value, typ,cases, FetchMode::Borrow),
            OpCode::Switch(value, typ, ref cases) => self.switch(value, typ, cases, FetchMode::Consume),
            OpCode::CopySwitch(value, typ, ref cases) => self.switch(value, typ, cases, FetchMode::Copy),
            OpCode::BorrowPack(typ, tag, ref values) => self.pack(typ, tag, values, FetchMode::Borrow),
            OpCode::Pack(typ, tag, ref values) => self.pack(typ, tag, values, FetchMode::Consume),
            OpCode::CopyPack(typ, tag, ref values) => self.pack(typ, tag, values, FetchMode::Copy),
            OpCode::Try(ref try, ref catches) => self.try(try,catches),
            OpCode::Invoke(func, ref vals) => self.invoke(func,vals),
            OpCode::Field(value, base_ref, field) => self.field(value, base_ref, field, FetchMode::Consume),
            OpCode::CopyField(value, base_ref, field) => self.field(value, base_ref, field, FetchMode::Copy),
            OpCode::BorrowField(value, base_ref, field) => self.field(value, base_ref, field, FetchMode::Borrow),
            OpCode::ModuleIndex => self.module_index(),
            OpCode::Image(val) =>  self.image(val),
            OpCode::ExtractImage(val) =>  self.unwrap_image(val),

        }
    }

    fn lit(&mut self, data:&LargeVec<u8>, typ:TypeRef) -> Result<()> {
        //Get the Resolved  Type of the Literal
        let r_lit = typ.fetch(&self.context)?;
        //Check with the Native Module to enforce that tis type can be generated from the provided Byte stream
        check_valid_literal_construction(&(data.0), &r_lit)?;
        //Tell the Stack that an element has appeared out of nowhere
        self.stack.provide(r_lit)
    }

    fn module_index(&mut self,) -> Result<()> {
        //Tell the Stack that an element has appeared out of nowhere
        self.stack.provide(resolved_native_type(NativeType::PrivateId, &[]))
    }

    //_ as let is keyword
    fn let_(&mut self, bind:&Exp) -> Result<()> {
        //capture frame start for clean up
        let start_height = self.stack.stack_depth();
        //Tell the stack that a new scope has started
        let block = self.stack.start_block();
        //Type check the content of the Let
        let results = self.type_check_exp(bind)?;
        //discard unneeded items
        self.clean_frame(&results,start_height)?;
        //Close the scope leaving only the results on the Stack
        self.stack.end_block(block,results)
    }

    fn fetch(&mut self, value:ValueRef, mode:FetchMode) -> Result<()> {
        if mode == FetchMode::Copy {
            //Get the Resolved Type of the source
            let v_typ = self.stack.value_of(value)?;
            //Copy is only allowed for types with the Copy capability
            if !v_typ.get_caps().contains(NativeCap::Copy) {
                return capability_missing_error()
            }
        }

        //Move or borrow the value to the top of the stack
        self.stack.fetch(value,mode)
    }

    fn image(&mut self, value:ValueRef) -> Result<()> {
        //get the input type
        let v_typ = self.stack.value_of(value)?;
        //wrap the type into an image
        let n_typ = Crc::new(ResolvedType::Image { typ:v_typ });
        //Copy the value on top with another type
        self.stack.transform(value, n_typ,FetchMode::Copy)
    }

    fn unwrap_image(&mut self, value:ValueRef) -> Result<()> {
        //get the input type
        let v_typ = self.stack.value_of(value)?;
        //check that it is a nested image
        if let ResolvedType::Image {typ:ref n_typ} = *v_typ {
            if let ResolvedType::Image { .. } = **n_typ {
                //Copy the value on top with another type
                return self.stack.transform(value, n_typ.clone(),FetchMode::Copy)
            }
        }
        type_mismatch()
    }


    fn discard(&mut self, value:ValueRef) -> Result<()> {
        if self.stack.is_borrowed(value)? {
            //free the element (unlock it's dependencies)
            self.stack.free(value)
        } else {
            //Get the Resolved Type of the target
            let v_typ = self.stack.value_of(value)?;
            //Drop is only allowed for types with the Drop capability
            if !v_typ.get_caps().contains(NativeCap::Drop) {
                return capability_missing_error()
            }
            //Tell the stack that the value is discarded so he can check the linearity constraints
            self.stack.drop(value)
        }
    }

    fn discard_many(&mut self, value:&[ValueRef]) -> Result<()> {
        for v in value {
            self.discard(*v)?
        }
        Ok(())
    }

    fn discard_borrowed(&mut self, value:ValueRef, borrows:&[ValueRef]) -> Result<()> {
        //Check that trg they accessible and borrowed
        if !self.stack.is_borrowed(value)? {
            return borrow_missing()
        }

        //process each steal
        for src in borrows {
            //Check that src accessible and borrowed
            if !self.stack.is_borrowed(*src)? {
                return borrow_missing()
            }

            //steal the dependency (unlock it)
            self.stack.steal(*src,value)?
        }

        //ensure the element is freed
        if !self.stack.can_be_released(value)? {
            return steal_violation()
        }

        Ok(())
    }

    fn unpack(&mut self, value:ValueRef, typ:TypeRef, mode:FetchMode, tag:Option<Tag>) -> Result<()>{
        //Fetch the base
        let e_typ = typ.fetch(&self.context)?;
        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value)?;
        //Check that the value on the stack has the correct base
        if r_typ != e_typ {return type_mismatch();}
        //Get the resolved constructors
        let r_ctr = self.context.get_ctrs(typ, self.context.store)?;

        //check if it is a literal
        if e_typ.is_literal() {
            return requested_ctr_missing()
        }

        //Unpack is not defined for types with less then one field in a single ctr
        if r_ctr.len() == 0{
            return requested_ctr_missing()
        };

        //To unpack the correct ctr must be known (specialised type)
        let tag = match tag {
            None => {
                if r_ctr.len() != 1 {
                    return wrong_opcode()
                };
                0 as u8
            }
            Some(Tag(t)) => t,
        };


        //check that we do not use Unpack on a borrowed value
        if self.stack.is_borrowed(value)? && mode == FetchMode::Consume {
            return cannot_be_borrowed()
        }

        //Decide if we have to make an inspect or a consume (borrowed values are inspected, rest consumed)
        //Check that the correct capability is available
        match mode {
            FetchMode::Consume => {
                //Consumed values need the consume capability
                if !e_typ.get_caps().contains(NativeCap::Consume) && !e_typ.is_local() {
                    return capability_missing_error()
                }
            },
            FetchMode::Copy => {
                //Copied values need the copy capability
                if !e_typ.get_caps().contains(NativeCap::Copy) {
                    return capability_missing_error()
                }
                //Copied values need the inspect capability
                if !e_typ.get_caps().contains(NativeCap::Inspect) && !e_typ.is_local() {
                    return capability_missing_error()
                }
            },
            FetchMode::Borrow => {
                //Borrowed values need the inspect capability
                if !e_typ.get_caps().contains(NativeCap::Inspect) && !e_typ.is_local() {
                    return capability_missing_error()
                }
            },
        }
        //Tell the stack to execute the operation (will take care of borrow vs consume)
        self.stack.unpack(value, &r_ctr[tag as usize],mode)
    }

    fn field(&mut self, value:ValueRef, typ:TypeRef, field:u8, mode:FetchMode) -> Result<()>{
        //Fetch the base
        let e_typ = typ.fetch(&self.context)?;
        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value)?;
        //Check that the value on the stack has the correct base
        if r_typ != e_typ {return type_mismatch();}
        //Get the resolved constructors
        let r_ctr = self.context.get_ctrs(typ, self.context.store)?;

        //check if it is a literal
        if e_typ.is_literal() {
            return requested_ctr_missing()
        }

        //Field get is not defined for types with less then one field in a single ctr
        if r_ctr.len() == 0{
            return requested_ctr_missing()
        };

        //check that we do not use Unpack on a borrowed value
        if self.stack.is_borrowed(value)? && mode == FetchMode::Consume {
            return cannot_be_borrowed()
        }

        //Decide if we have to make an inspect or a consume (borrowed values are inspected, rest consumed)
        match mode {
            FetchMode::Consume => {
                if !e_typ.get_caps().contains(NativeCap::Consume) && !e_typ.is_local() {
                    return capability_missing_error()
                }
            },
            FetchMode::Borrow | FetchMode::Copy => {
                //Borrowed values need the inspect capability
                if !e_typ.get_caps().contains(NativeCap::Inspect) && !e_typ.is_local() {
                    return capability_missing_error()
                }
            },
        }

        let typ = r_ctr[0 as usize][field as usize].clone();

        //Decide if we have field requirements
        match mode {
            FetchMode::Consume => {
                for (idx,field_type) in r_ctr[0 as usize].iter().enumerate() {
                    if idx != field as usize && !field_type.get_caps().contains(NativeCap::Drop) {
                        return capability_missing_error()
                    }
                }
            },
            FetchMode::Copy => {
                //Borrowed values need the copy capability
                if !typ.get_caps().contains(NativeCap::Copy) {
                    return capability_missing_error()
                }
            },
            FetchMode::Borrow =>{}
        }

        //Tell the stack to execute the operation (will take care of borrow vs consume)
        self.stack.field(value, typ, mode)
    }


    fn switch(&mut self, value:ValueRef, typ:TypeRef, cases:&[Exp], mode:FetchMode) -> Result<()> {
        //switch makes only sense itf their are 2 or more ctrs
        if cases.len() <= 1 {
            return wrong_opcode()
        };

        //Get the Resolved Type of the value
        let r_typ = self.stack.value_of(value)?;
        //Get the resolved constructors
        let r_ctr = self.context.get_ctrs(typ, self.context.store)?;

        //check if it is a literal
        if r_typ.is_literal() {
            return requested_ctr_missing()
        }

        //Check that their is exactly one case per potential constructor
        if r_ctr.len() != cases.len() {
            return requested_ctr_missing()
        };


        //capture frame start for clean up
        let start_height = self.stack.stack_depth();
        //just a helper to make the loop simpler -- represents the types from the previous case (loop iter)
        let mut loop_res = None;
        //Tell the stack that a the control flow branches
        let mut branching = self.stack.start_branching();
        //Process all the branches
        // Note: The stack ensures that each branch returns the same Elements (this includes their type)
        for (i,case) in cases.iter().enumerate() {
            //if this is not the first iter then tell the stack that the next branch will be processed (will restore stack)
            if let Some(res) = loop_res {
                //discard unneeded items
                self.clean_frame( &res,start_height)?;
                //go to next branch
                self.stack.next_branch( &mut branching, res)?;
            }
            //unpack the elements into the branch
            self.unpack(value, typ, mode, Some(Tag(i as u8)))?;
            //remaining operations are specified by branch code and now type checked
            let res = self.type_check_exp(case)?;
            //pass intermediary result to next iter
            loop_res = Some(res);
        }
        //discard unneeded items
        self.clean_frame( &loop_res.unwrap(),start_height)?;
        //finish the branching, leaves the stack with the common elements
        self.stack.end_branching(branching, loop_res.unwrap())
    }

    fn pack(&mut self, typ:TypeRef, Tag(t):Tag, values:&[ValueRef], mode:FetchMode) -> Result<()> {
        //Get resolved Type
        let r_typ = typ.fetch(&self.context)?;
        // Checkt that it is local or native
        if !r_typ.get_caps().contains(NativeCap::Create) && !r_typ.is_local() {
            return capability_missing_error()
        }

        //check if it is a literal
        if r_typ.is_literal() {
            return requested_ctr_missing()
        }

        //Get the Resolved Constructors
        let r_ctr = self.context.get_ctrs(typ, self.context.store)?;

        //check if applicable
        if r_ctr.len() == 0 {
            return requested_ctr_missing()
        }

        //check that the case exists and has the right number of fields
        if (t as usize) >= r_ctr.len() || r_ctr[t as usize].len() != values.len() {
            return requested_ctr_missing()
        }

        //check that the case exists and has the right number of fields
        if r_ctr[t as usize].is_empty() && mode == FetchMode::Borrow {
            return empty_borrow_error()
        }

        //check that each param is ok
        for (i,v) in values.iter().enumerate() {
            //fetch the type of the param
            let r_v = self.stack.value_of(*v)?;

            //check that we do not use Pack on a borrowed value
            if self.stack.is_borrowed(*v)? && mode == FetchMode::Consume  {
                return cannot_be_borrowed()
            }

            //check that the value has the copy if required
            if mode == FetchMode::Copy && !r_v.get_caps().contains(NativeCap::Copy){
                return capability_missing_error()
            }

            //Check that the type of the param matches
            if r_ctr[t as usize][i] != r_v {
                return type_mismatch()
            }
        }

        //Tell the stack to pack the value and place the result onto the stack
        self.stack.pack(&values, r_typ, mode)
    }

    fn try(&mut self, try:&Exp, catches:&[(ErrorRef, Exp)]) -> Result<()> {
        // Add the risks that the try handles to the allowed risks
        let mut new_risks = self.risk.clone();
        for (err,_) in catches {
            new_risks.insert(err.fetch(&self.context)?);
        }

        //Save the currently allowed Risks
        let old_risk = mem::replace(&mut self.risk,new_risks);
        //capture frame start for clean up
        let start_height = self.stack.stack_depth();
        // tell the stack to start a branching frame
        let mut branching = self.stack.start_branching();
        //Type check the Try block
        let results1 = self.type_check_exp(try)?;
        //discard unneeded items
        self.clean_frame(&results1, start_height)?;
        //Start first catch block
        self.stack.next_branch(&mut branching,results1)?;
        //The catch blocks can no longer throw the new risks (only try can) but can still throw the old ones
        self.risk = old_risk;
        //helper for simpler loop, is the value from previous iters
        let mut loop_res = None;
        //Process each catch blocks
        //  Note: The Stack ensures that each branch returns the same elemnets (that includes their type)
        for (_,exp) in catches {
            //start new catch block if not done already
            if let Some(res) = loop_res {
                //discard unneeded items
                self.clean_frame(&res, start_height)?;
                // go to next branch
                self.stack.next_branch(&mut branching,res)?;
            }
            //type check the catch block
            let res = self.type_check_exp(exp)?;
            //pass result to next loop iter
            loop_res = Some(res);
        }
        //discard unneeded items
        self.clean_frame(&loop_res.unwrap(), start_height)?;
        //finish the try catch stack will only contain result types
        self.stack.end_branching(branching, loop_res.unwrap())
    }

    fn invoke(&mut self, func:FuncRef, vals:&[ValueRef]) -> Result<()>{
        //Get the Resolved Function & forward to the method shared wiith entry point
        let sig = self.context.get_func_sig(func, &self.context.store)?;
        self.invoke_direct( &sig, vals)
    }

    fn invoke_direct(&mut self, signature:&Rc<ResolvedSignature>, vals:&[ValueRef]) -> Result<()>{
        //Make sure that the functions risks are covered by the current function
        if !signature.risks.iter().all(|risk|self.risk.contains(risk)) {
            return risk_missing()
        }
        //Check that the right amount of arguments are supplied for the call
        if signature.params.len() != vals.len() {
            return num_param_mismatch()
        }

        //Prepare the Inputs
        let inputs:Vec<(ValueRef,bool)> = vals.iter().zip(signature.params.iter()).map(|(v,p)| {
            //Ensure tat the argument has the expected type
            if self.stack.value_of(*v)? != p.typ {
                type_mismatch()
            } else {
                Ok((*v, p.consumes))
            }
        }).collect::<Result<_>>()?;

        //Copy into the right structure
        // I do not like that this is necessry
        let outputs = signature.returns.iter()
            .map(|ResolvedReturn{ref typ, ref borrows}|(typ.clone(), &borrows[..]))
            .collect::<Vec<_>>();

        //Advice the stack to apply the Funcformation
        self.stack.apply(&inputs, &outputs)
    }
}
