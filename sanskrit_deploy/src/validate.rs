//! A Validator that checks certain condition on the input and the linked Components
//!

use sanskrit_core::model::*;
use sanskrit_core::model::linking::*;
use sanskrit_core::model::resolved::*;
use sanskrit_core::resolver::*;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::*;
use sanskrit_common::store::Store;
use sanskrit_core::utils::Crc;
use sanskrit_core::loader::Loader;
use code_type_checker::TypeCheckerContext;
use sanskrit_common::model::ModuleLink;
use sanskrit_common::model::Hash;
use sanskrit_core::model::bitsets::{CapSet, BitSet, PermSet};
use sanskrit_core::accounting::Accounting;

pub fn validate_top_function<S:Store>(data:&[u8], store:&S, accounting:&Accounting) -> Result<()>{
    //Parse the function
    let fun:FunctionComponent = Parser::parse_fully::<FunctionComponent,NoCustomAlloc>(data, usize::max_value(),&NoCustomAlloc())?;
    //Prepare the cache for this iteration
    let resolver = Loader::new_complete(store, &accounting);
    //Prepare the context
    let context = Context::from_top_component(&fun, &resolver)?;
    //Ensure Transaction specific parts are correct
    validate_transaction(&fun, &context)?;
    //Do the type checking of the code in the function body
    if let CallableImpl::Internal {ref code, ..} = fun.body {
        TypeCheckerContext::new(accounting, context).type_check_function(&fun, code)?;
    }
    Ok(())
}

pub fn validate<S:Store>(data:&[u8], store:&S, accounting:&Accounting, link:Hash, system_mode_on:bool) -> Result<()> {
    //Parse the module
    let parsed: Module = Parser::parse_fully::<Module, NoCustomAlloc>(data, usize::max_value(), &NoCustomAlloc())?;
    //Prepare the cache for this iteration
    let resolver = Loader::new_incremental(store, link, parsed, &accounting);
    //Get a reference to the cached Module
    let module = resolver.get_module(link)?;
    // get the module lnk
    let module_link = resolver.dedup_module_link(ModuleLink::This(link));

    for sel in &module.data_sig_order.0 {
        if *sel {
            let tdd = resolver.this_deployed_data.get();
            //check it is their
            if tdd >= module.data.len() {
                return error(||"Orderer addresses unavailable Data Component")
            }
            //get it
            let d = &module.data[tdd];
            //Prepare the context
            let context = Context::from_module_component(d, &module_link, false,&resolver)?;
            //Ensure the input is formally correct and has the expected properties
            validate_adt(d, &context, system_mode_on)?;
            //Hint the cache that a new Adt is available in the current module
            resolver.this_deployed_data.set(tdd + 1);
        } else {
            let tds = resolver.this_deployed_sigs.get();
            //check it is their
            if tds >= module.sigs.len() {
                return error(||"Orderer addresses unavailable Signature Component")
            }
            //get it
            let s = &module.sigs[tds];
            //Prepare the context
            let context = Context::from_module_component(s, &module_link, false, &resolver)?;
            //Ensure the input is formally correct and has the expected properties
            validate_sig(s, &context)?;
            //Hint the cache that a new Signature is available in the current module
            resolver.this_deployed_sigs.set(tds + 1);
        }
    }

    for sel in &module.fun_impl_order.0 {
        if *sel {
            let tdf = resolver.this_deployed_functions.get();
            //check it is their
            if tdf >= module.functions.len() {
                return error(||"Orderer addresses unavailable Function Component")
            }
            //get it
            let f = &module.functions[tdf];
            let context = Context::from_module_component(f, &module_link, true, &resolver)?;
            //Ensure the input is formally correct and has the expected properties
            validate_function(f, &context, system_mode_on)?;
            //Do the type checking of the code in the function body
            if let CallableImpl::Internal {ref code, ..} = f.body {
                TypeCheckerContext::new(accounting, context).type_check_function(f, code)?;
            }
            //Hint the cache that a new Function is available in the current module
            resolver.this_deployed_functions.set(tdf + 1);
        } else {
            let tdi = resolver.this_deployed_implements.get();
            //check it is their
            if tdi >= module.implements.len() {
                return error(||"Orderer addresses unavailable Implement Component")
            }
            //get it
            let i = &module.implements[tdi];
            //Prepare the context
            let context = Context::from_module_component(i, &module_link, true,&resolver)?;
            //Ensure the input is formally correct and has the expected properties
            validate_implement(i, &context, system_mode_on)?;
            //Do the type checking of the code in the function body
            if let CallableImpl::Internal {ref code, ..} = i.body {
                TypeCheckerContext::new(accounting, context).type_check_implement(i, code)?;
            }
            //Hint the cache that a new Implement is available in the current module
            resolver.this_deployed_implements.set(tdi+1);
        }
    }

    Ok(())
}

//Checks that an Adt declaration is semantically valid
fn validate_adt<S:Store>(adt:&DataComponent, context:&Context<S>, system_mode_on:bool ) -> Result<()>{
    //ensure only system deploys tops
    if adt.top && !system_mode_on {
        return error(||"system mode is required for deployment of top data type")
    }

    //Check the import section
    //we allow no protection forwarding as adts have 2 visibilities & should not need to import create, consumes & implements
    check_type_import_integrity(context)?;

    //Check capability constraints
    check_provided_capability_constraints(adt)?;
    check_generic_capability_constraints(&adt.generics)?;

    //Check the body
    match adt.body {
        DataImpl::Internal { ref constructors, .. } => {
            //All caps are allowed
            //Check constructors for phantoms and enforce recursive capabilities
            check_ctr_fields(adt.provided_caps, constructors, context)?
        },
        DataImpl::External(_) => if !system_mode_on {
            return error(||"Deploying externals requires system mode")
        },
    }

    //check visibility
    check_visibility_integrity(&adt.create_scope, adt.generics.len())?;
    check_visibility_integrity(&adt.consume_scope, adt.generics.len())

}



//Checks that an signature declaration is semantically valid
fn validate_sig<S:Store>(sig:&SigComponent, ctx:&Context<S>) -> Result<()>{
    //Check the import section
    //we allow no protection forwarding as sigs have 2 visibilities & should not need to import create, consumes & implements
    check_type_import_integrity(ctx)?;

    //Check capability constraints
    check_provided_sig_capability_constraints(&sig)?;
    check_generic_capability_constraints(&sig.shared.generics)?;

    //check param and return for phantoms
    check_params_and_returns(&sig.shared.params, &sig.shared.returns, ctx)?;

    //check visibility
    check_visibility_integrity(&sig.call_scope, sig.shared.generics.len())?;
    check_visibility_integrity(&sig.implement_scope, sig.shared.generics.len())

}

fn check_body<S:Store>(fun:&CallableImpl, ctx:&Context<S>, system_mode_on:bool) -> Result<()> {    //load everithing
    match fun {
        CallableImpl::External => if !system_mode_on {return error(||"Deploying externals requires system mode")},
        CallableImpl::Internal { .. } => {
            //Check integret of imports
            check_function_import_integrity(ctx)?;
            //check that the imported functions are visible
            check_callable_import_visibility(ctx)?;
            //check permissions
            check_permission_visibility(ctx)?;
        },
    }
    Ok(())
}

fn validate_transaction<S:Store>(fun:&FunctionComponent, ctx:&Context<S>) -> Result<()> {
    //Transactions must be public
    match fun.scope {
        Accessibility::Guarded(_) | Accessibility::Local => return error(||"Transactions functions must be public"),
        Accessibility::Global => {},
    }

    //Check the import section
    check_type_import_integrity(ctx)?;

    //Transactions can not have Generics
    if fun.shared.generics.len() != 0 {
        return error(||"Transactions functions can not be generic")
    }

    //Check the Body
    check_body(&fun.body, ctx, false)?;

    //check param and return for phantoms
    check_txt_params_and_returns(&fun.shared.params, &fun.shared.returns, ctx)?;

    //check visibility
    check_visibility_integrity(&fun.scope, fun.shared.generics.len())
}

//Checks that an function declaration is semantically valid
fn validate_function<S:Store>(fun:&FunctionComponent, ctx:&Context<S>, system_mode_on:bool) -> Result<()> {
    //Check the import section
    check_type_import_integrity(ctx)?;

    //Check capability constraints
    check_generic_capability_constraints(&fun.shared.generics)?;

    //check the body
    check_body(&fun.body , ctx, system_mode_on)?;

    //check param and return for phantoms
    check_params_and_returns(&fun.shared.params, &fun.shared.returns, ctx)?;

    //check visibility
    check_visibility_integrity(&fun.scope, fun.shared.generics.len())
}


//Checks that an function declaration is semantically valid
fn validate_implement<S:Store>(imp:&ImplementComponent, ctx:&Context<S>, system_mode_on:bool) -> Result<()> {
    //Check the import section
    check_type_import_integrity(ctx)?;

    //Check capability constraints
    check_generic_capability_constraints(&imp.generics)?;

    //check the captures and permission
    check_implement_constraints(imp, ctx)?;

    //check the body
    check_body(&imp.body , ctx, system_mode_on)?;

    //check param and return for phantoms
    check_params_and_returns(&imp.params, &[], ctx)?;

    //check visibility
    check_visibility_integrity(&imp.scope, imp.generics.len())

}

fn check_provided_capability_constraints(adt:&DataComponent) -> Result<()> {
    adt.provided_caps.check_constraints()?;
    if adt.provided_caps.contains(Capability::Primitive) {
        if adt.create_scope != Accessibility::Global || adt.consume_scope != Accessibility::Global || adt.inspect_scope != Accessibility::Global {
            return error(||"A primitive data type can only have public permissions")
        }
    }
    Ok(())
}

fn check_provided_sig_capability_constraints(sig:&SigComponent) -> Result<()> {
    sig.provided_caps.check_constraints()?;
    if !sig.provided_caps.intersect(CapSet::signature_prohibited()).is_empty() {
        return error(||"Only the drop capability is allowed on signature types")
    }

    Ok(())
}


fn check_generic_capability_constraints(gens:&[Generic]) -> Result<()> {
    for g in gens {
        match g {
            Generic::Phantom => {}
            Generic::Physical(caps) => caps.check_constraints()?
        }
    }
    Ok(())
}


fn check_generic_constraints(generics:&[Generic], applies:&[Crc<ResolvedType>]) -> Result<()>{
    // check that the number of applies is correct
    if generics.len() != applies.len() {
        return error(||"Applied types mismatch required generics")
    }

    for (generic,typ) in  generics.iter().zip(applies.iter()) {
        //update caps
        if let Generic::Physical(l_caps) = *generic{
            //A Phantom generic or virtual can only be applied to a phantom generic
            match **typ {
                ResolvedType::Generic { is_phantom:true,  .. }
                | ResolvedType::Virtual(_) => return error(||"Phantom types can not be used as to apply non phantom generics"),
                _ => {}
            }

            //when a Physical is applied the applier must have all the required capabilities
            check_cap_constraints(l_caps, typ.get_caps())?;
        }
    }
    Ok(())
}

//Checks that native imports are allowed and have correct amount and kind of arguments
//Checks that normal imports have correct amount and kind of arguments
fn check_type_import_integrity<S:Store>(context:&Context<S>) -> Result<()> {
    //iterate over all type imports
    for typ in context.list_types() {
        //resolve the type to ensure it is legit
        match **typ {
            //Nothing has to be checked for Generics, Projections and Virtuals
            ResolvedType::Generic { .. }
            | ResolvedType::Projection { .. }
            | ResolvedType::Virtual(_) => {},
            //we need to check implement and call visibility
            ResolvedType::Sig { ref module, offset, ref applies, .. } => {
                //Fetch the Cache Entry
                let imp_sig_cache = context.store.get_component::<SigComponent>(&*module, offset)?;
                //Retrieve the function from the cache
                let imp_sig_comp = imp_sig_cache.retrieve();
                //check it
                check_generic_constraints(&imp_sig_comp.shared.generics, applies)?;
            },
            ResolvedType::Data { ref module, offset, ref applies, .. } => {
                //Fetch the Cache Entry
                let imp_data_cache = context.store.get_component::<DataComponent>(&*module, offset)?;
                //Retrieve the function from the cache
                let imp_data_comp = imp_data_cache.retrieve();
                //check it
                check_generic_constraints(&imp_data_comp.generics, applies)?;
            }
            //An imported type (can be from same module if Module == This)
            ResolvedType::Lit { ref module, offset, ref applies, .. } => {
                //Fetch the Cache Entry
                let imp_data_cache = context.store.get_component::<DataComponent>(&*module, offset)?;
                //Retrieve the function from the cache
                let imp_data_comp = imp_data_cache.retrieve();
                //check it
                check_generic_constraints(&imp_data_comp.generics, applies)?;
            }
        }
    }
    Ok(())
}

fn check_function_import_integrity<S:Store>(context:&Context<S>) -> Result<()> {
    //iterate over all type imports
    for c in context.list_callables() {
        //resolve the type to ensure it is legit
        match **c {
            //we need to check implement consraints
            ResolvedCallable::Implement { ref module, offset, ref applies, .. } => {
                //Fetch the Cache Entry
                let imp_cache = context.store.get_component::<ImplementComponent>(&*module, offset)?;
                //Retrieve the function from the cache
                let imp_comp = imp_cache.retrieve();
                //check it
                check_generic_constraints(&imp_comp.generics, applies)?;
            },
            ResolvedCallable::Function { ref module, offset, ref applies, .. } => {
                //Fetch the Cache Entry
                let imp_fun_cache = context.store.get_component::<FunctionComponent>(&*module, offset)?;
                //Retrieve the function from the cache
                let imp_fun_comp = imp_fun_cache.retrieve();
                //check it
                check_generic_constraints(&imp_fun_comp.shared.generics, applies)?;
            }
        }
    }
    Ok(())
}

//check that capp full fills constraints
fn check_cap_constraints(must_have_caps:CapSet, caps:CapSet) -> Result<()>{
    //check the target has all the necessary caps + embed
    //Note: a top level generic in an adt context does always have all the rec caps
    if !must_have_caps.is_subset_of(caps) {
        return error(||"Capabilities of type must full fill the constraints")
    }
    Ok(())
}

//Checks that the fields in the constructor do not use phantoms as type and do not violate the recursive generics declared on the adt
fn check_ctr_fields<S:Store>(provided_caps:CapSet, ctrs:&[Case], context:&Context<S>) -> Result<()> {
    //Go over all constructors
    for ctr in ctrs {
        //Go over all fields
        for field in &ctr.fields {
            //resolve field type
            match **field.fetch(context)?.get_target() {
                //if the type is generic ensure it is not a phantom
                // Externals are always Phantom types
                ResolvedType::Generic { is_phantom:true, .. }
                | ResolvedType::Virtual(_) => return error(||"Phantom types can not be used as constructor fields"),
                // Note: generic recursive-caps is delayed (rechecked) to apply side to allow Option[T] (or even Option[Option[T]] instead of requiring DropOption[Drop T] ... PersistOption[Persist T] etc...
                //if a regular type on non phantom generic check that it does support the caps
                ResolvedType::Data { generic_caps, .. }
                | ResolvedType::Sig { generic_caps, .. }
                | ResolvedType::Lit { generic_caps, .. } => check_cap_constraints(provided_caps,generic_caps)?,
                ResolvedType::Generic { .. } => {},
                ResolvedType::Projection { .. }  => unreachable!()
            }
        }
    }
    Ok(())
}

//check the functions visibility
fn check_visibility(comp_vis:&Accessibility, comp_module:&Crc<ModuleLink>, comp_applies:&[Crc<ResolvedType>]) -> Result<()> {
    //check the functions visibility
    match comp_vis {
        //public can always be imported
        Accessibility::Global => {},
        //private can only be imported if from the same module
        Accessibility::Local => if !comp_module.is_local_link() {
            return error(||"A private permission must be from the current module")
        },
        //Protected can only be imported if the protected types are owned or are declared as protected as well
        Accessibility::Guarded(ref guards) => {
            //check that all protected types are ok
            for &GenRef(index) in guards {
                //check the ownership protection
               if !comp_applies[index as usize].is_local() {
                    return error(||"A type from the current module is required to be applied to a protected generic")
               }
            }
        }
    }
    Ok(())
}

//Checks the visibility constraints
fn check_callable_import_visibility<S:Store>(context:&Context<S>) -> Result<()> {
    //iterate over all imported functions
    for c in context.list_callables() {
        //fetch the function
        match **c {
            ResolvedCallable::Function { ref module,offset, ref applies, ..} => {
                //Fetch the Cache Entry
                let imp_fun_cache = context.store.get_component::<FunctionComponent>(&*module, offset)?;
                //Retrieve the function from the cache
                let imp_fun_comp = imp_fun_cache.retrieve();
                //check it
                check_visibility(&imp_fun_comp.scope, module, &applies)?;
            },

            ResolvedCallable::Implement  { ref module,offset, ref applies, ..} => {
                //Fetch the Cache Entry
                let imp_cache = context.store.get_component::<ImplementComponent>(&*module, offset)?;
                //Retrieve the function from the cache
                let imp_comp = imp_cache.retrieve();
                //check it
                check_visibility(&imp_comp.scope, module, &applies)?;
            }
        }
    }
    Ok(())
}

//Checks the visibility constraints
fn check_permission_visibility<S:Store>(context:&Context<S>) -> Result<()> {
    //iterate over all imported functions
    for p in context.list_perms() {
        //fetch the function
        match **p {
            ResolvedPermission::FunSig {perm, ref fun, ..} => {
                if !perm.is_subset_of(PermSet::callable_perms()) {
                    return error(||"Permissions not applicable to callable")
                }
                match **fun {
                    ResolvedCallable::Function {ref module,offset, ref applies,..} => {
                        if perm.contains(Permission::Call) {
                            //Fetch the Cache Entry
                            let fun_cache = context.store.get_component::<FunctionComponent>(&*module, offset)?;
                            //Retrieve the function from the cache
                            let fun = fun_cache.retrieve();
                            //check it
                            check_visibility(&fun.scope, module, &applies)?;
                        }
                    },
                    ResolvedCallable::Implement {ref module,offset, ref applies,..} => {
                        if perm.contains(Permission::Call) {
                            //Fetch the Cache Entry
                            let imp_cache = context.store.get_component::<ImplementComponent>(&*module, offset)?;
                            //Retrieve the function from the cache
                            let imp = imp_cache.retrieve();
                            //check it
                            check_visibility(&imp.scope, module, &applies)?;
                        }
                    }
                }
            },
            ResolvedPermission::TypeLit {perm, ref typ, ..} => {
                if !perm.is_subset_of(PermSet::lit_perms()) {
                    return error(||"Permissions not applicable to literal")
                }
                match **typ.get_target() {
                    ResolvedType::Lit {ref module,offset, ..} => {
                        if perm.contains(Permission::Create) {
                            //Fetch the Cache Entry
                            let imp_data_cache = context.store.get_component::<DataComponent>(&*module, offset)?;
                            //Retrieve the function from the cache
                            let imp_data_comp = imp_data_cache.retrieve();
                            //check it
                            check_visibility(&imp_data_comp.create_scope, module, &[])?;
                        }
                    },
                    _ =>  return error(||"Create permission must be used on a lit type")
                }
            },
            ResolvedPermission::TypeData {perm, ref typ, ..} => {
                if !perm.is_subset_of(PermSet::data_perms()) {
                    return error(|| "Permissions not applicable to data")
                }
                match **typ.get_target() {
                    ResolvedType::Data { ref module, offset, ref applies, .. } => {
                        //Fetch the Cache Entry
                        let imp_data_cache = context.store.get_component::<DataComponent>(&*module, offset)?;
                        //Retrieve the function from the cache
                        let imp_data_comp = imp_data_cache.retrieve();
                        //check it
                        if perm.contains(Permission::Create) {
                            check_visibility(&imp_data_comp.create_scope, module, &applies)?;
                        }
                        if perm.contains(Permission::Consume) {
                            check_visibility(&imp_data_comp.consume_scope, module, &applies)?;
                        }
                        if perm.contains(Permission::Inspect) {
                            check_visibility(&imp_data_comp.inspect_scope, module, &applies)?;
                        }
                    },
                    _ => return error(|| "TypeData permissions must be used on a data type")
                }
            },

            ResolvedPermission::TypeSig {perm, ref typ, ..} => {
                if !perm.is_subset_of(PermSet::sig_perms()) {
                    return error(|| "Permissions not applicable to signature")
                }
                match **typ {
                    ResolvedType::Sig {ref module,offset, ref applies,..} => {
                        //Fetch the Cache Entry
                        let imp_sig_cache = context.store.get_component::<SigComponent>(&*module, offset)?;
                        //Retrieve the function from the cache
                        let imp_sig_comp = imp_sig_cache.retrieve();
                        //check it
                        if perm.contains(Permission::Call) {
                            check_visibility(&imp_sig_comp.call_scope, module, &applies)?;
                        }
                        if perm.contains(Permission::Implement) {
                            check_visibility(&imp_sig_comp.implement_scope, module, &applies)?;
                        }
                    } ,
                    _ => return error(||"Call permission must be used on a not projected signature type")
                }
            }
        }
    }
    Ok(())
}

fn check_visibility_integrity(vis:&Accessibility, num_gens:usize) -> Result<()>  {
    match vis {
        Accessibility::Guarded(ref guards) => {
            //check that all protected types are ok
            for &GenRef(index) in guards {
                //fetch the corresponding type in the importer context and check it
                if index as usize >= num_gens {
                    return error(||"Generic type used in protection declaration not found")
                }
            }
        }
        Accessibility::Local | Accessibility::Global => {},
    }
    Ok(())
}

//Checks that params and return to not phantoms
fn check_params_and_returns<S:Store>(params:&[Param], returns:&[TypeRef], context:&Context<S>) -> Result<()> {

    //process all params
    for &Param{ typ, ..} in params {
        match *typ.fetch(context)?  {
            ResolvedType::Generic { is_phantom:true, .. }
            | ResolvedType::Virtual(_) => return error(||"Phantom types can not be used as parameter types"),
            _ => {}
        }
    }

    //process all returns
    for typ in returns {
        let r_typ = typ.fetch(context)?;
        match *r_typ  {
            ResolvedType::Generic { is_phantom:true, .. }
            | ResolvedType::Virtual(_) => return error(||"Phantom types can not be used as return types"),
            _ => { }
        }
        if !r_typ.get_caps().contains(Capability::Unbound) {
            return error(||"Returning a value requires the unbound capability")
        }
    }
    Ok(())
}


//Checks that params and return to not phantoms
fn check_txt_params_and_returns<S:Store>(params:&[Param], returns:&[TypeRef], context:&Context<S>) -> Result<()> {

    //process all params
    for &Param{ typ, ..} in params {
        match **typ.fetch(context)?.get_target() {
            ResolvedType::Data {caps, ref module, offset, ..}
            | ResolvedType::Lit { caps, ref module, offset, ..} => {
                if !caps.contains(Capability::Primitive) {
                    //Fetch the Cache Entry
                    let data_cache = context.store.get_component::<DataComponent>(&*module, offset)?;
                    //Retrieve the function from the cache
                    let data = data_cache.retrieve();
                    if !data.top {
                        return error(||"transaction parameter must be primitives or top values")
                    }
                }
            },
            _ => return error(||"transaction parameter must be primitives or top values")
        }
    }

    //process all returns
    for typ in returns {
        let r_typ = typ.fetch(context)?;
        match **r_typ.get_target() {
            ResolvedType::Data {caps, ref module, offset, ..}
            | ResolvedType::Lit { caps, ref module, offset, ..} => {
                if !caps.contains(Capability::Primitive) {
                    //Fetch the Cache Entry
                    let data_cache = context.store.get_component::<DataComponent>(&*module, offset)?;
                    //Retrieve the function from the cache
                    let data = data_cache.retrieve();
                    if !data.top {
                        return error(||"transaction returns must be primitives or top values")
                    }
                }
            },
            _ => return error(||"transaction returns must be primitives or top values")
        }
        if !r_typ.get_caps().contains(Capability::Unbound) {
            return error(||"Returning a value requires the unbound capability")
        }
    }
    Ok(())
}


fn check_implement_constraints<S:Store>(imp:&ImplementComponent, context:&Context<S>) -> Result<()> {
    //Ensure we are permissioned
    let perm = imp.sig.fetch(context)?;
    //Check the permission type
    if !perm.check_permission(Permission::Implement) {
        return error(||"Wrong permission supplied: Implement Permission needed")
    }

    let transactional = perm.get_sig()?.transactional;

    if let ResolvedType::Sig {caps:sig_caps , ..} = **perm.get_type()? {
        for capture in &imp.params {
            //The captures must be consumed (owned)
            if !capture.consumes {
                return error(||"Implements can not have borrow parameters")
            }

            //fetch the type
            let typ = capture.typ.fetch(context)?;

            //check that the values can be dropped in case of a rollback
            if transactional && !typ.get_caps().contains(Capability::Drop) {
                return error(||"Implements of transactional signatures can only capture values with the Drop capability")
            }

            //The captures must full fill sig caps
            //resolve field type
            match **capture.typ.fetch(context)?.get_target() {
                //if the type is generic ensure it is not a phantom
                // Externals are always Phantom types
                ResolvedType::Generic { is_phantom:true, .. }
                | ResolvedType::Virtual(_) => return error(||"Phantom types can not be used as parameter types"),
                //if a regular type on non phantom generic check that it does support the caps
                ResolvedType::Generic { caps, .. }
                | ResolvedType::Data { caps, .. }
                | ResolvedType::Sig { caps, .. }
                | ResolvedType::Lit { caps, .. } => check_cap_constraints(sig_caps,caps)?,
                ResolvedType::Projection { .. } => unreachable!()
            }
        }
    } else {
        return error(||"Permission supplied is not for a signature type");
    }
    Ok(())
}