use alloc::rc::Rc;
use utils::Cache;
use sanskrit_common::errors::*;
use sanskrit_common::capabilities::*;
use native::base::get_native_type_constructors;
use loader::StorageCache;
use sanskrit_common::store::Store;
use utils::Crc;
use native::fun::resolved_native_function;
use native::base::resolved_native_type;
use native::base::check_native_type_constraints;
use model::resolved::*;
use model::*;
use native::fun::check_native_function_constraints;
use alloc::prelude::*;
use sanskrit_common::model::*;
use context::*;

//All things that can be cached in an import
pub struct Cachings {
    mref_cache:Cache<ModRef, Rc<ModuleLink>>,                   //Cached Modules
    erref_cache:Cache<ErrorRef, Rc<ResolvedErr>>,               //Cached Errors
    tref_cache:Cache<TypeRef, Crc<ResolvedType>>,               //Cached Types
    ctr_cache:Cache<TypeRef, Rc<Vec<Vec<Crc<ResolvedType>>>>>,  //Cached Constructors
    fref_cache:Cache<FuncRef, Rc<ResolvedFunction>>,            //Cached Functions
    sig_cache:Cache<FuncRef, Rc<ResolvedSignature>>,            //Cached Signatures
}


impl Cachings {
    //Generates a fresh Empty Cache
    pub fn new(ctx:&InputContext) -> Self {
        Cachings{
            mref_cache: Cache::new(ctx.num_modules()),
            erref_cache: Cache::new(ctx.num_error_imports()),
            tref_cache: Cache::new(ctx.num_types()),
            ctr_cache: Cache::new(ctx.num_types()),
            fref_cache: Cache::new(ctx.num_function_imports()),
            sig_cache: Cache::new(ctx.num_function_imports()),
        }
    }
}

pub struct Context<'a,'b, S:Store + 'b> {
    pub ctx: InputContext<'a>,          //The Plain Input Context
    pub subs: Vec<Crc<ResolvedType>>,   //The substitutions to use to resolve generic parameters
    pub cache: Rc<Cachings>,            //The Caches
    pub store: &'b StorageCache<'b, S>, //The Store
    pub checking: bool,
}

pub fn top_level_subs(generics:&[Generic]) -> Vec<Crc<ResolvedType>> {
    generics.iter().enumerate().map(|(i,c)| match *c {
        Generic::Phantom => Crc::new(ResolvedType::Generic { extended_caps: CapSet::empty(), caps: CapSet::empty(), offset:i as u8, is_phantom:true}),
        Generic::Physical(caps) => {
            let extended_caps = caps.union(CapSet::recursive());
            Crc::new(ResolvedType::Generic { extended_caps, caps, offset:i as u8, is_phantom:false})
        },
    }).collect()
}

impl<'a,'b, S:Store + 'b> Context<'a,'b, S> {
    //Generates a top level local Context for an Adt fromminput
    pub fn from_input_adt(adt:&'a AdtComponent, link:Hash, store:&'b StorageCache<'b,S>) -> Result<Self> {
        Self::from_adt(adt,ModuleLink::This(link),store,true)
    }

    //Generates a top level local Context for an Adt fromminput
    pub fn from_store_adt(adt:&'a AdtComponent, link:Hash, store:&'b StorageCache<'b,S>) -> Result<Self> {
        Self::from_adt(adt,ModuleLink::This(link),store,false)
    }

    fn from_adt(adt:&'a AdtComponent, link:ModuleLink, store:&'b StorageCache<'b,S>, checking:bool) -> Result<Self> {
        //Get the input context for the Adt
        let ctx = InputContext::from_adt(adt,link);
        //check that the size is ok (should be given)
        assert!(adt.generics.len() <= u8::max_value() as usize);

        //Generate a local context
        Ok(Context {
            //The substitutions are just top level generics
            subs: top_level_subs(&adt.generics),
            cache: Rc::new(Cachings::new(&ctx)),
            ctx,
            store,
            checking,
        })
    }

    pub fn from_input_func(fun:&'a FunctionComponent, link:Hash, store:&'b StorageCache<'b,S>) -> Result<Self> {
        Self::from_func(fun,ModuleLink::This(link),store,true)
    }

    pub fn from_store_func(fun:&'a FunctionComponent, link:Hash, store:&'b StorageCache<'b,S>) -> Result<Self> {
        Self::from_func(fun,ModuleLink::This(link),store,false)
    }

    fn from_func(fun:&'a FunctionComponent, link:ModuleLink, store:&'b StorageCache<'b,S>, checking:bool) -> Result<Self> {
        //Get the input context for the Adt
        let ctx = InputContext::from_function(fun,link);
        //check that the size is ok (should be given)
        assert!(fun.generics.len() <= u8::max_value() as usize);

        //Generate a local context
        Ok(Context {
            //The substitutions are just top level generics
            subs:top_level_subs(&fun.generics),
            cache: Rc::new(Cachings::new(&ctx)),
            ctx,
            store,
            checking,
        })
    }

    //Gets and resolves a type from the local context
    pub fn get_type(&self, tref:TypeRef) -> Result<Crc<ResolvedType>> {
        //only do this once and then cache it (detects cycles as well)
        self.cache.tref_cache.cached(tref,||{
            //get the input type from the Input Context
            match self.ctx.get_type(tref.0) {
                //Its a general
                Some(Type::Real(base_type, applies)) => {
                   //build the type
                    match base_type {
                        BaseType::Module(AdtLink{module, offset}) => {
                            //Fetch the link
                            let module_link =  self.get_mod(*module)?;
                            //Load the Adt
                            let adt_cache = self.store.get_adt(&*module_link,*offset)?;
                            // get the adt
                            let adt = adt_cache.retrieve();
                            //prepare the applies vector
                            let result:Vec<Crc<ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect::<Result<_>>()?;
                            //check that the type is legit
                            if self.checking { self.check_adt_constraints(adt,&result)?; }
                            //Resolve the type
                            Ok(self.resolve_adt_type(adt, module_link, *offset, &result))
                        },
                        BaseType::Native(typ) => {
                            //prepare the applies vector
                            let result:Vec<Crc<ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect::<Result<_>>()?;
                            //check that the type is legit
                            if self.checking { check_native_type_constraints(*typ,&result)?; }
                            //Resolve the type
                            Ok(resolved_native_type(*typ, &result))
                        }
                    }
                },
                //its a generic, do a substitution
                Some(Type::Generic(gref)) => match self.subs.get(gref.0 as usize) {
                    None => return type_does_not_exist_error(),
                    Some(res) => Ok(res.clone()),
                },
                //ups no such type
                None => return type_does_not_exist_error()
            }
        })
    }

    //Gets and resolves a Module from the local context
    pub fn get_mod(&self, mref:ModRef) -> Result<Rc<ModuleLink>> {
        //only do this once and then cache it (detects cycles as well)
        self.cache.mref_cache.cached(mref, ||{
            //Get the Input Module link
            Ok(Rc::new(*match self.ctx.get_module(mref.0) {
                Some(module) => module,
                None => return module_does_not_exist_error(),
            }))
        })
    }

    //Gets and resolves an Error from the local context
    pub fn get_err(&self, erref:ErrorRef) -> Result<Rc<ResolvedErr>> {
        //only do this once and then cache it (detects cycles as well)
        self.cache.erref_cache.cached(erref, ||{
            //Get the Input error
            Ok(Rc::new(match self.ctx.get_error_import(erref.0) {
                //Construct the resolved type
                Some(ErrorImport::Module(link)) => ResolvedErr::Import{
                    offset: link.offset,
                    module: self.get_mod(link.module)?, //Get the Module
                },
                Some(ErrorImport::Native(err)) => ResolvedErr::Native { err:*err },
                None => return error_does_not_exist_error(),
            }))
        })
    }

    //Gets and resolves an Function from the local context
    pub fn get_func(&self, fref:FuncRef) -> Result<Rc<ResolvedFunction>> {
        //only do this once and then cache it (detects cycles as well)
        self.cache.fref_cache.cached(fref, ||{
            //Get the Input Function
            Ok(Rc::new(match self.ctx.get_function_import(fref.0) {
                Some(FunctionImport::Module(FuncLink{module, offset}, applies)) => {
                    //Fetch the link
                    let module_link =  self.get_mod(*module)?;
                    //Fetch the Cache Entry
                    let fun_cache = self.store.get_func(&*module_link,*offset)?;
                    //Retrieve the function from the cache
                    let fun= fun_cache.retrieve();
                    // prepare the applies vector
                    let result:Vec<Crc<ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect::<Result<_>>()?;
                    //check the amount of generics is ok
                    if self.checking { self.check_func_constraints(fun,&result)?; }

                    ResolvedFunction::Import{
                        offset: *offset,
                        module: module_link, //Get the Module
                        //Resolve the applied types
                        applies:result,
                    }
                },
                Some(FunctionImport::Native(func, applies)) =>{
                    // prepare the applies vector
                    let result:Vec<Crc<ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect::<Result<_>>()?;
                    // delegate the check to the native module
                    if self.checking {  check_native_function_constraints(*func, &result)?; }
                    // build the type
                    ResolvedFunction::Native {
                        typ:*func,
                        //Resolve the applied types
                        applies: result,
                    }
                }
                None => return error_does_not_exist_error(),
            }))
        })
    }

    //Gets and resolves a Function signature from the local context
    pub fn get_func_sig(&self, fref:FuncRef, store:&StorageCache<S>) -> Result<Rc<ResolvedSignature>> {
        //only do this once and then cache it (detects cycles as well)
        self.cache.sig_cache.cached(fref, ||{
            //Get the Input Function
            Ok(Rc::new(match *self.get_func(fref)? {
                //forward native to native module
                ResolvedFunction::Native { typ, ref applies } => resolved_native_function(typ, applies)?,
                //If an import we have to import it
                ResolvedFunction::Import { ref module, offset, ref applies } =>{
                    //Get the cache
                    let func_cache = store.get_func(&**module,offset)?;
                    //get the function
                    let fun = func_cache.retrieve();
                    //Get its context
                    let context = func_cache.substituted_context(applies.clone(), store)?;
                    //Build the signature
                    ResolvedSignature {
                        //Map the params to the resolved type
                        params: fun.params.iter().map(|c|{
                            let typ = context.get_type(c.typ)?;
                            Ok(ResolvedParam{ typ, consumes: c.consumes })
                        }).collect::<Result<_>>()?,
                        //Map the returns to the resolved type
                        returns: fun.returns.iter().map(|r|{
                            let typ = context.get_type(r.typ)?;
                            Ok(ResolvedReturn{ typ, borrows: r.borrows.clone() })
                        }).collect::<Result<_>>()?,
                        //Map the risks to the resolved risks
                        risks: fun.risk.iter().map(|err|context.get_err(*err)).collect::<Result<_>>()?,
                    }
                }
            }))
        })
    }


    //Gets and resolves an Adt Constructor from the local context
    pub fn get_ctrs(&self, tref:TypeRef, store:&StorageCache<S>) -> Result<Rc<Vec<Vec<Crc<ResolvedType>>>>> {
        //only do this once and then cache it (detects cycles as well)
        self.cache.ctr_cache.cached(tref, ||{
            //Get the Input Type
            Ok(Rc::new(match *self.get_type(tref)? {
                ResolvedType::Generic { .. } => return generics_dont_have_ctrs_error(),
                //Forward to the native module
                ResolvedType::Native { typ, ref applies, .. } =>  get_native_type_constructors(typ,applies)?,
                //If an import we have to import it
                ResolvedType::Import { ref module , offset, ref applies, .. } => {
                    //Get the cache
                    let adt_cache = store.get_adt(&**module,offset)?;
                    //Get the adt
                    let adt = adt_cache.retrieve();
                    //get its context with the applies as substitutions
                    let context = adt_cache.substituted_context(applies.iter().map(|apl|apl.typ.clone()).collect(),store)?;
                    //build the ctrs by retriving their fields
                    adt.constructors.iter().map(|c|{
                        c.fields.iter().map(|t|context.get_type(*t)).collect::<Result<_>>()
                    }).collect::<Result<_>>()?
                }
            }))
        })
    }

    fn check_adt_constraints(&self, adt:&AdtComponent, applies:&[Crc<ResolvedType>]) -> Result<()>{
        // check that the number of applies is correct
        if adt.generics.len() != applies.len() {
            return num_applied_generics_error();
        }

        for (generic,typ) in  adt.generics.iter().zip(applies.iter()) {
            //update caps
            if let Generic::Physical(l_caps) = *generic{
                //A Phantom generic can only be applied to a non-phantom generic
                if let ResolvedType::Generic { is_phantom:true,  .. }  = **typ {
                    return can_not_apply_phantom_to_physical_error()
                }
                //when a Physical is applied the applier must have all the required capabilities
                if !l_caps.is_subset_of(typ.get_caps()){
                    return type_apply_constraint_violation()
                }
            }
        }
        Ok(())
    }

    fn check_func_constraints(&self, fun:&FunctionComponent, applies:&[Crc<ResolvedType>]) -> Result<()> {
        //check the amount of generics is ok
        if (fun.generics.len()) != applies.len()  {return num_applied_generics_error()}
        //Check that the phantom & cap constraints hold
        for (c, appl) in fun.generics.iter().zip(applies.iter()) {
            if let Generic::Physical(caps) = *c {
                //A Phantom generic can only be applied to a non-phantom generic
                if let ResolvedType::Generic { is_phantom:true,  .. }  = **appl {
                    return can_not_apply_phantom_to_physical_error()
                }
                //when a Physical is applied the applier must have all the required capabilities
                if !caps.is_subset_of(appl.get_caps()){
                    return type_apply_constraint_violation()
                }
            }
        }
        Ok(())
    }

    fn resolve_adt_type(&self, adt:&AdtComponent, module:Rc<ModuleLink>, offset:u8,  applies:&[Crc<ResolvedType>]) -> Crc<ResolvedType>{
        // calc the caps after application & check constraints
        //prepare the applies vector
        let are_phantom = adt.generics.iter().map(|generic|match *generic {
            Generic::Phantom => true,
            Generic::Physical(_) => false,
        });
        let (extended_caps,caps,result) = apply_types(adt.provided_caps,are_phantom,applies);
        //Construct the Type
        Crc::new(ResolvedType::Import {
            caps,
            extended_caps,
            base_caps:adt.provided_caps,
            module,
            offset,
            applies:result,
        })
    }
}


pub fn apply_types(base_caps:CapSet, are_phantom:impl Iterator<Item=bool>, applies:&[Crc<ResolvedType>]) -> (CapSet,CapSet, Vec<ResolvedApply>) {
    //prepare the applies vector
    let mut result = Vec::with_capacity(applies.len());
    //initial caps
    let mut extended_caps = base_caps;
    let mut caps = base_caps;
    for (is_phantom,typ) in  are_phantom.zip(applies.iter()) {
        //update caps
        if !is_phantom{
            //combine caps
            extended_caps = extended_caps.intersect(typ.get_extended_caps());
            caps = caps.intersect(typ.get_caps());
            //push the result
            result.push(ResolvedApply{ is_phantom: false, typ:typ.clone() })
        } else {
            //push the result
            result.push(ResolvedApply{ is_phantom: true, typ:typ.clone() })
        }
    }
    //finalize caps
    extended_caps = extended_caps.union(base_caps.intersect(CapSet::non_recursive()));
    caps = caps.union(base_caps.intersect(CapSet::non_recursive()));
    (extended_caps,caps,result)
}