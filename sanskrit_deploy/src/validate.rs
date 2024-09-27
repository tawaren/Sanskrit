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
use crate::code_type_checker::TypeCheckerContext;
use sanskrit_common::model::ModuleLink;
use sanskrit_common::model::Hash;
use sanskrit_core::model::bitsets::{CapSet, BitSet, PermSet};

pub fn validate_top_function<S:Store>(data:&[u8], store:&S) -> Result<()>{
    //Parse the function
    let fun:FunctionComponent = Parser::parse_fully::<FunctionComponent,NoCustomAlloc>(data, usize::max_value(),&NoCustomAlloc())?;
    //Prepare the cache for this iteration
    let resolver = Loader::new_complete(store);
    //Prepare the context
    let context = Context::from_top_component(&fun, &resolver)?;

    //let context = match Context::from_top_component(&fun, &resolver)
    //Ensure Transaction specific parts are correct
    validate_transaction(&fun, &context)?;
    //Do the type checking of the code in the function body
    if let CallableImpl::Internal {ref code, ..} = fun.body {
        let mut checker = TypeCheckerContext::<S>::new(context);
        checker.type_check_function(&fun, code)?;
    }
    Ok(())
}

pub fn validate<S:Store>(data:&[u8], store:&S, link:Hash, system_mode_on:bool) -> Result<()> {
    //Parse the module
    let parsed: Module = Parser::parse_fully::<Module, NoCustomAlloc>(data, usize::MAX, &NoCustomAlloc())?;
    //Prepare the cache for this iteration
    let resolver = Loader::new_incremental(store, link, parsed);
    //Get a reference to the cached Module
    let module = resolver.get_module(link)?;
    // get the module lnk
    let module_link = resolver.dedup_module_link(ModuleLink::This(link));

    //Mange current sigs and adts for forward reference checking
    let mut cur_adt_offset = 0;
    let mut cur_sig_offset = 0;
    #[cfg(feature = "forward_type_ref")]
    resolver.this_deployed_data.set( module.data.len());
    #[cfg(feature = "forward_type_ref")]
    resolver.this_deployed_sigs.set( module.sigs.len());

    for sel in &module.data_sig_order.0 {
        if *sel {
            //check it is their
            if cur_adt_offset >= module.data.len() {
                return error(||"Orderer addresses unavailable Data Component")
            }
            //get it
            let d = &module.data[cur_adt_offset];
            //Prepare the context
            let context = Context::from_module_component(d, &module_link, false,&resolver)?;
            //Ensure the input is formally correct and has the expected properties
            validate_adt(d, &context, system_mode_on, #[cfg(feature = "forward_type_ref")] cur_adt_offset)?;
            cur_adt_offset += 1;
            //Hint the cache that a new Adt is available in the current module
            #[cfg(not(feature = "forward_type_ref"))]
            resolver.this_deployed_data.set(cur_adt_offset);
        } else {
            //check it is their
            if cur_sig_offset >= module.sigs.len() {
                return error(||"Orderer addresses unavailable Signature Component")
            }
            //get it
            let s = &module.sigs[cur_sig_offset];
            //Prepare the context
            let context = Context::from_module_component(s, &module_link, false, &resolver)?;
            //Ensure the input is formally correct and has the expected properties
            validate_sig(s, &context)?;
            cur_sig_offset += 1;
            //Hint the cache that a new Signature is available in the current module
            #[cfg(not(feature = "forward_type_ref"))]
            resolver.this_deployed_sigs.set(cur_sig_offset);
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
                let mut checker = TypeCheckerContext::<S>::new(context);
                checker.type_check_function(f, code)?;
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
                let mut checker = TypeCheckerContext::<S>::new(context);
                checker.type_check_implement(i, code)?;
            }
            //Hint the cache that a new Implement is available in the current module
            resolver.this_deployed_implements.set(tdi+1);
        }
    }

    Ok(())
}

//Checks that an Adt declaration is semantically valid
fn validate_adt<S:Store>(adt:&DataComponent, context:&Context<S>, system_mode_on:bool, #[cfg(feature = "forward_type_ref")] cur_adt_offset:usize) -> Result<()>{
    //Check the import section
    //ensure that no self referencing is used in applies unless they are phantom or literal
    #[cfg(feature = "forward_type_ref")]
    check_save_self_referencing(context, cur_adt_offset)?;
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
            check_ctr_fields(adt.provided_caps, constructors, context, #[cfg(feature = "forward_type_ref")] cur_adt_offset)?
        },
        DataImpl::External(_) => if !system_mode_on {
            return error(||"Deploying externals requires system mode")
        },
    }

    //check visibility
    check_accessibility_integrity(&adt.create_scope, adt.generics.len())?;
    check_accessibility_integrity(&adt.consume_scope, adt.generics.len())?;
    check_accessibility_integrity(&adt.inspect_scope, adt.generics.len())

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
    check_accessibility_integrity(&sig.call_scope, sig.shared.generics.len())?;
    check_accessibility_integrity(&sig.implement_scope, sig.shared.generics.len())

}

fn check_body<S:Store>(fun:&CallableImpl, ctx:&Context<S>, system_mode_on:bool) -> Result<()> {    //load everithing
    match fun {
        CallableImpl::External => if !system_mode_on {return error(||"Deploying externals requires system mode")},
        CallableImpl::Internal { .. } => {
            //Check integret of imports
            check_function_import_integrity(ctx)?;
            //check that the imported functions are visible
            check_callable_import_accessibility(ctx, system_mode_on)?;
            //check permissions
            check_permission_accessibility(ctx, system_mode_on)?;
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
    check_accessibility_integrity(&fun.scope, fun.shared.generics.len())
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
    check_accessibility_integrity(&fun.scope, fun.shared.generics.len())
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
    check_accessibility_integrity(&imp.scope, imp.generics.len())

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


//Checks that there are no cycles in the apply graph
#[cfg(feature = "forward_type_ref")]
fn check_save_self_referencing<S:Store>(context:&Context<S>, cur_adt_offset:usize) -> Result<()> {
    //iterate over all type imports
    for typ in context.list_types() {
        //resolve the type to ensure it is legit
        match **typ {
            //Nothing has to be checked for Generics and Virtuals they have no applies
            ResolvedType::Generic { .. }
            | ResolvedType::Virtual(_)
            //Literals & sigs are the only allowed forward reference
            // This is save as they have a Fixed Size independent of their applies
            | ResolvedType::Lit { .. }
            | ResolvedType::Sig { .. } => { }

            //We need to ensure that their is no cycle in the data graph
            // This is needed as the applies can appear in the constructor
            // leading to infinitely large data structures
            ResolvedType::Data { ref module, offset, ref applies, .. } => {
                //Fetch the Cache Entry
                let data_cache = context.store.get_component::<DataComponent>(&*module, offset)?;
                //Retrieve the data component from the cache
                let data_comp = data_cache.retrieve();

                for (g,apply) in data_comp.generics.iter().zip(applies.iter()) {
                   match g {
                       Generic::Phantom => {},
                       Generic::Physical(_) => ensure_backwards_ref(apply, cur_adt_offset)?
                   }
               }
            },
            ResolvedType::Projection { ref un_projected,.. } => {
                ensure_backwards_ref(un_projected, cur_adt_offset)?;
            }

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
            //Nothing has to be checked for Generics and Virtuals
            ResolvedType::Generic { .. }
            | ResolvedType::Virtual(_) => {},
            ResolvedType::Projection { ref un_projected, .. } => {
                //only value types can be projected
                if !un_projected.get_caps().contains(Capability::Value) {
                    return error(||"Only value types can be projected")
                }
            }
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
    //check the target has all the necessary caps
    //Note: a top level generic in an adt context does always have all the rec caps
    if !must_have_caps.is_subset_of(caps) {
        return error(||"Capabilities of type must full fill the constraints")
    }
    Ok(())
}

//Checks that the fields in the constructor do not use phantoms as type and do not violate the recursive generics declared on the adt
fn check_ctr_fields<S:Store>(provided_caps:CapSet, ctrs:&[Case], context:&Context<S>, #[cfg(feature = "forward_type_ref")] cur_adt_offset:usize) -> Result<()> {
    //Go over all constructors
    for ctr in ctrs {
        //Go over all fields
        for field in &ctr.fields {
            let typ = field.typ.fetch(context)?;
            //checks that the constructor fields are not self referential
            #[cfg(feature = "forward_type_ref")]
            ensure_backwards_ref(&*typ, cur_adt_offset)?;
            //Resolve the field type
            match *typ {
                //if the type is generic ensure it is not a phantom
                // Externals are always Phantom types
                ResolvedType::Generic { is_phantom: true, .. }
                | ResolvedType::Virtual(_) => return error(|| "Phantom types can not be used as constructor fields"),
                // Note: generic recursive-caps is delayed (rechecked) to apply side to allow Option[T] (or even Option[Option[T]] instead of requiring DropOption[Drop T] ... PersistOption[Persist T] etc...
                //if a regular type on non phantom generic check that it does support the caps
                ResolvedType::Sig { caps: generic_caps, .. }
                | ResolvedType::Lit { generic_caps, .. }
                | ResolvedType::Data { generic_caps, .. } => {
                    check_cap_constraints(provided_caps, generic_caps)?
                },
                ResolvedType::Generic { .. } => {},
                //We have all capabilities on a projection
                ResolvedType::Projection { ref un_projected, .. } => { },
            }
        }
    }
    Ok(())
}

//ensures that their are no cycles in type references or adt compositions
// this is achieved by requiring that types can not point to the current or future definition in general
#[cfg(feature = "forward_type_ref")]
fn ensure_backwards_ref(typ:&ResolvedType, cur_adt_offset:usize) -> Result<()> {
    match *typ {
        //Data types additionally need to check that the constructor fields are not self referential
        ResolvedType::Data { ref module, offset, .. } => {
            if module.is_local_link() && cur_adt_offset <= offset as usize {
                return error(||"Data constructors can not contain forward references involving other data types")
            }
        },
        ResolvedType::Projection { ref un_projected, .. } => {
            ensure_backwards_ref(un_projected, cur_adt_offset)?
        },
        _ => {}
    }
    Ok(())
}

//check the functions visibility
fn check_access(comp_access:&Accessibility, comp_module:&Crc<ModuleLink>, comp_applies:&[Crc<ResolvedType>], system_mode_on:bool) -> Result<()> {
    //We have always access to types defined in the same module or if we are in system mode
    if comp_module.is_local_link() || system_mode_on {
        return Ok(())
    }
    match comp_access {
        //public can always be imported
        Accessibility::Global => {},
        //private can only be imported if from the same module (already checked)
        Accessibility::Local => return error(||"A private permission must be from the current module"),
        //Protected can only be imported if the guarded types are owned
        Accessibility::Guarded(ref guards) => {
            //check that all protected types are ok
            for &GenRef(index) in guards {
                //check the ownership protection
               if !comp_applies[index as usize].is_local() {
                    return error(||"A type from the current module is required to be applied to a guarded generic")
               }
            }
        }
    }
    Ok(())
}

//Checks the visibility constraints
fn check_callable_import_accessibility<S:Store>(context:&Context<S>, system_mode_on:bool) -> Result<()> {
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
                check_access(&imp_fun_comp.scope, module, &applies, system_mode_on)?;
            },

            ResolvedCallable::Implement  { ref module,offset, ref applies, ..} => {
                //Fetch the Cache Entry
                let imp_cache = context.store.get_component::<ImplementComponent>(&*module, offset)?;
                //Retrieve the function from the cache
                let imp_comp = imp_cache.retrieve();
                //check it
                check_access(&imp_comp.scope, module, &applies, system_mode_on)?;
            }
        }
    }
    Ok(())
}

//Checks the visibility constraints
fn check_permission_accessibility<S:Store>(context:&Context<S>, system_mode_on:bool) -> Result<()> {
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
                            check_access(&fun.scope, module, &applies, system_mode_on)?;
                        }
                    },
                    ResolvedCallable::Implement {ref module,offset, ref applies,..} => {
                        if perm.contains(Permission::Call) {
                            //Fetch the Cache Entry
                            let imp_cache = context.store.get_component::<ImplementComponent>(&*module, offset)?;
                            //Retrieve the function from the cache
                            let imp = imp_cache.retrieve();
                            //check it
                            check_access(&imp.scope, module, &applies, system_mode_on)?;
                        }
                    }
                }
            },
            ResolvedPermission::TypeLit {perm, ref typ, ..} => {
                if !perm.is_subset_of(PermSet::lit_perms()) {
                    return error(||"Permissions not applicable to literal")
                }
                match **typ {
                    //We have all permissions on a projection
                    ResolvedType::Projection { .. } => {}
                    ResolvedType::Lit {ref module,offset, ..} => {
                        if perm.contains(Permission::Create) {
                            //Fetch the Cache Entry
                            let imp_data_cache = context.store.get_component::<DataComponent>(&*module, offset)?;
                            //Retrieve the function from the cache
                            let imp_data_comp = imp_data_cache.retrieve();
                            //check it
                            check_access(&imp_data_comp.create_scope, module, &[], system_mode_on)?;
                        }
                    },
                    _ =>  return error(||"Create permission must be used on a lit type")
                }
            },
            ResolvedPermission::TypeData {perm, ref typ, ..} => {
                if !perm.is_subset_of(PermSet::data_perms()) {
                    return error(|| "Permissions not applicable to data")
                }
                match **typ {
                    //We have all permissions on a projection
                    ResolvedType::Projection { .. } => {}
                    ResolvedType::Data { ref module, offset, ref applies, .. } => {
                        //Fetch the Cache Entry
                        let imp_data_cache = context.store.get_component::<DataComponent>(&*module, offset)?;
                        //Retrieve the function from the cache
                        let imp_data_comp = imp_data_cache.retrieve();
                        //check it
                        if perm.contains(Permission::Create) {
                            check_access(&imp_data_comp.create_scope, module, &applies, system_mode_on)?;
                        }
                        if perm.contains(Permission::Consume) {
                            check_access(&imp_data_comp.consume_scope, module, &applies, system_mode_on)?;
                        }
                        if perm.contains(Permission::Inspect) {
                            check_access(&imp_data_comp.inspect_scope, module, &applies, system_mode_on)?;
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
                            check_access(&imp_sig_comp.call_scope, module, &applies, system_mode_on)?;
                        }
                        if perm.contains(Permission::Implement) {
                            check_access(&imp_sig_comp.implement_scope, module, &applies, system_mode_on)?;
                        }
                    } ,
                    _ => return error(||"Call permission must be used on a not projected signature type")
                }
            }
        }
    }
    Ok(())
}

fn check_accessibility_integrity(vis:&Accessibility, num_gens:usize) -> Result<()>  {
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
fn check_txt_params_and_returns<S:Store>(_params:&[Param], returns:&[TypeRef], context:&Context<S>) -> Result<()> {

    //Nothing to do for params anymore
    //process all returns
    for typ in returns {
        let r_typ = typ.fetch(context)?;
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
            match *capture.typ.fetch(context)? {
                //if the type is generic ensure it is not a phantom
                // Externals are always Phantom types
                ResolvedType::Generic { is_phantom:true, .. }
                | ResolvedType::Virtual(_) => return error(||"Phantom types can not be used as parameter types"),
                //if a regular type on non phantom generic check that it does support the caps
                ResolvedType::Generic { caps, .. }
                | ResolvedType::Data { caps, .. }
                | ResolvedType::Sig { caps, .. }
                | ResolvedType::Lit { caps, .. } => check_cap_constraints(sig_caps,caps)?,
                //Projections have all caps
                ResolvedType::Projection { .. } => { }
            }
        }
    } else {
        return error(||"Permission supplied is not for a signature type");
    }
    Ok(())
}