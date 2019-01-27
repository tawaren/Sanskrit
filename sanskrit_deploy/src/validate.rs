//! A Validator that checks certain condition on the input and the linked Components
//!
//! This module has the following responsibilities:
//!  1. Check that when a non module is deployed the module it is deployed into declares the component
//!  2. Check that the declarations in an import declaration matches which the actual component they point to
//!  3. Check that the declared types are consistent, primarily that Capabilities are used as Capabilities not as Base Types and vice versa
//!  4. Check that the applied types have the correct amount and the correct kind of parameters
//!  5. That the declarations of the generic parameters is done correctly
//!  6. Check that the imports of function do full fill see the imported function (visibility constraints)
//!  7. Check that the parameter and return types are base types and not capabilities

use sanskrit_core::model::*;
use sanskrit_core::model::linking::*;
use sanskrit_core::model::resolved::*;
use sanskrit_core::resolver::*;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::*;
use sanskrit_common::store::Store;
use sanskrit_core::utils::Crc;
use sanskrit_common::capabilities::*;
use sanskrit_core::loader::StorageCache;
use code_type_checker::TypeCheckerContext;
use sanskrit_common::model::NativeCap;
use sanskrit_common::model::Hash;


pub fn validate<S:Store>(data:&[u8], store:&S, link:Hash) -> Result<()>{
    //Parse the module
    let parsed:Module = Parser::parse_fully::<Module,NoCustomAlloc>(data, usize::max_value(),&NoCustomAlloc())?;
    //Prepare the cache for this iteration
    let resolver = StorageCache::new_incremental(store, link,parsed);
    //Get a reference to the cached Module
    let module = resolver.get_module(&link)?;

    //Check each Adt for validity
    for a in &module.adts {
        //Prepare the context
        let context = Context::from_input_adt(a, link, &resolver)?;
        //Ensure the input is formally correct and has the expected properties
        validate_adt(a, &context)?;
        //Hint the cache that a new Adt is available in the current module
        let tda = resolver.this_deployed_adts.get();
        resolver.this_deployed_adts.set(tda+1);
    }

    for f in &module.functions {
        //Prepare the context
        let context = Context::from_input_func(f, link, &resolver)?;
        //Ensure the input is formally correct and has the expected properties
        validate_function(f, &context)?;
        //Do the type checking of the code in the function body
        TypeCheckerContext::new(context).type_check_function(f)?;
        //Hint the cache that a new Function is available in the current
        let tdf = resolver.this_deployed_functions.get();
        resolver.this_deployed_functions.set(tdf+1);
    }
    Ok(())
}


//Checks that an Adt declaration is semantically valid
pub fn validate_adt<S:Store>(adt:&AdtComponent, context:&Context<S>) -> Result<()>{
    //check capability implications
    check_capabilities(&adt, context)?;
    //Check the import section
    check_type_import_constraints_validity(&adt.import, context)?;
    //Check constructors for phantoms and enforce recursive capabilities
    check_ctr_fields(adt, context)
}

//Checks that an function declaration is semantically valid
pub fn validate_function<S:Store>(fun:&FunctionComponent, ctx:&Context<S>) -> Result<()>{
    //Check the import section
    check_type_import_constraints_validity(&fun.import.base, ctx)?;

    //check that the imported functions have valid generic parameter
    check_function_import_validity(&fun.import.functions, ctx)?;

    //check that the imported functions are visible
    check_function_import_visibility(fun, ctx)?;

    //check param and return for phantoms
    check_params_and_returns(fun, ctx)?;

    //check visibility
    check_function_visibility(fun)
}

fn check_capabilities<S:Store>(adt:&AdtComponent, context:&Context<S>) -> Result<()>{

    //Make sure Consume implies Inspect
    if adt.provided_caps.contains(NativeCap::Consume) && !adt.provided_caps.contains(NativeCap::Inspect) {
        return capability_implication_mismatch()
    }

    if adt.provided_caps.contains(NativeCap::Indexed) {
        //Make sure Indexed implies Persist
        if !adt.provided_caps.contains(NativeCap::Persist) {
            return capability_implication_mismatch()
        }

        //Make sure each ctr is indexed
        for ctr in &adt.constructors{
            //must have at least 1 field per ctr to be indexed
            if ctr.fields.is_empty() {
                return capability_constraints_violation()
            }

            //first field must be indexed as well
            if !ctr.fields[0].fetch(context)?.get_caps().contains(NativeCap::Indexed) {
                return capability_constraints_violation()
            }
        }
    }

    Ok(())
}

//Checks that native imports are allowed and have correct amount and kind of arguments
//Checks that normal imports have correct amount and kind of arguments
fn check_type_import_constraints_validity<S:Store>(import:&PublicImport, context:&Context<S>) -> Result<()> {
    //iterate over all type imports
    for t in 0..import.types.len() {
        //ensure that the length bound hold (should be given)
        assert!(t <= u8::max_value() as usize);
        //resolve the type to ensure it is legit
        // non-legit types can not be resolved
        // is required to ensure that even unused but imported types are legit
        // will cache the result so it can be reused
        TypeRef(t as u8).fetch(context)?;
    }
    Ok(())
}

//Checks that the fields in the constructor do not use phantoms as type and do not violate the recursive generics declared on the adt
fn check_ctr_fields<S:Store>(adt:&AdtComponent, context:&Context<S>) -> Result<()> {
    fn check_cap_constraints(must_have_caps:CapSet, caps:CapSet) -> Result<()>{
        //check the target has all the necessary caps + embed
        //Note: a top level generic in an adt context does always have all the rec caps
        if !must_have_caps.is_subset_of(caps) {
            return capability_constraints_violation()
        }
        //check that they are embeddable
        if !caps.contains(NativeCap::Embed) {
            return capability_missing_error()
        }

        Ok(())
    }

    //The fields must have all the recursive caps supported by the adt, so extract them for later use
    let must_have_caps = adt.provided_caps.intersect(CapSet::recursive());
    //Go over all constructors
    for ctr in &adt.constructors {
        //Go over all fields
        for field in &ctr.fields {
            //resolve field type
            match *field.fetch(context)? {
                //if the type is generic ensure it is not a phantom
                // Note: generic recursive-caps is delayed (rechecked) to apply side to allow Option[T] (or even Option[Option[T]] instead of requiring DropOption[Drop T] ... PersistOption[Persist T] etc...
                ResolvedType::Generic { extended_caps, is_phantom, .. } => {
                    //Check that they are real
                    if is_phantom {
                        return vals_need_to_be_real()
                    }
                    //check that the caps hold
                    check_cap_constraints(must_have_caps,extended_caps)?
                },
                //if a regular type check that it does support the caps
                ResolvedType::Import { extended_caps, .. }
                | ResolvedType::Native { extended_caps, .. } => check_cap_constraints(must_have_caps,extended_caps)?
            }
        }
    }
    Ok(())
}

//Checks the generic function param application
fn check_function_import_validity<S:Store>(funs:&[FunctionImport], context:&Context<S>) -> Result<()> {
    //iterate over all imported functions
    for f in 0..funs.len() {
        //ensure that the length bound hold (should be given)
        assert!(f <= u8::max_value() as usize);
        //resolve the function to ensure it is legit
        // non-legit function can not be resolved
        // is required to ensure that even unused but imported function are legit
        // will cache the result so it can be reused
        FuncRef(f as u8).fetch(context)?;
    }
    Ok(())
}

//Checks the visibility constraints
fn check_function_import_visibility<S:Store>(fun:&FunctionComponent, context:&Context<S>) -> Result<()> {
    //Helper to check if a base full fills the protection constraint
    // is seperate for readability purposes
    fn check_typ_protection(fun:&FunctionComponent, typ:&Crc<ResolvedType>) -> Result<()> {
        match &**typ {
            //Natives are primitive and owned collectively
            ResolvedType::Native{ .. } => visibility_violation(),
            //Imported ones must be from this Module
            ResolvedType::Import { module, .. } => {
                //We must own the type to use it under protection (shared types are collectively owned)
                //Meaning: It must either be defined in the same module(is_local_link) or collectively owned (is_shared)
                if !module.is_local_link() {
                    return visibility_violation()
                }
                Ok(())
            },
            //if the protected is itself generic check if we are protected to it as well or if it is a primitive
            ResolvedType::Generic{ offset, .. } => {
                match fun.visibility {
                    //privates and public can not call protected on its generics
                    Visibility::Private | Visibility::Public => {
                        return visibility_violation()
                    },
                    //Protected can call protected if it declares the type under investigation as well (or if it requires it to be primitive)
                    Visibility::Protected(ref prot) => if !prot.contains(&GenRef(*offset)){
                        return visibility_violation()
                    },
                }
                Ok(())
            },
        }
    }

    //iterate over all imported functions
    for f in 0..fun.import.functions.len() {
        //fetch the function
        let imp_fun = FuncRef(f as u8).fetch(context)?;
        match *imp_fun {
            ResolvedFunction::Import{ ref module , offset, ref applies } => {
                //Fetch the Cache Entry
                let imp_fun_cache = context.store.get_func(&**module, offset)?;
                //Retrieve the function from the cache
                let imp_fun_comp= imp_fun_cache.retrieve();
                //check the functions visibility
                match imp_fun_comp.visibility {
                    //public can always be imported
                    Visibility::Public => {},
                    //private can only be imported if from the same module
                    Visibility::Private => if !module.is_local_link() {return visibility_violation()},
                    //Protected can only be imported if the protected types are owned or are declared as protected as well
                    Visibility::Protected(ref guards) => {
                        //check that all protected types are ok
                        for &GenRef(index) in guards {
                            //check the ownership/forwarded protection
                            check_typ_protection(fun,&applies[index as usize])?
                        }
                    }
                }
            },
            ResolvedFunction::Native{..} => {/*For now all are public*/},
        }
    }
    Ok(())
}

fn check_function_visibility(fun:&FunctionComponent) -> Result<()>  {
    match fun.visibility {
        Visibility::Protected(ref guards) => {
            //check that all protected types are ok
            for &GenRef(index) in guards {
                //fetch the corresponding type in the importer context and check it
                if index as usize >= fun.generics.len() {return wrong_protect_declaration()}
            }
        }
        Visibility::Private | Visibility::Public => {},
    }
    Ok(())
}

//Checks that params and return to not phantoms
fn check_params_and_returns<S:Store>(fun:&FunctionComponent, context:&Context<S>) -> Result<()> {

    //process all params
    for &Param{ typ, ..} in &fun.params {
        if let ResolvedType::Generic { is_phantom:true, .. } = *typ.fetch(context)? {
            //forbid phantoms
            return vals_need_to_be_real()
        }
    }

    //process all returns
    for Ret{ typ, ..} in &fun.returns {
        if let  ResolvedType::Generic { is_phantom:true, .. } =  *typ.fetch(context)? {
            //forbid phantoms
            return vals_need_to_be_real()
        }
    }
    Ok(())
}