use alloc::rc::Rc;
use utils::Cache;
use sanskrit_common::errors::*;
use sanskrit_common::capabilities::*;
use loader::StorageCache;
use sanskrit_common::store::Store;
use utils::Crc;
use model::resolved::*;
use model::*;
use alloc::vec::Vec;
use sanskrit_common::model::*;
use context::*;

//All things that can be cached in an import
pub struct Cachings {
    mref_cache:Cache<ModRef, Rc<ModuleLink>>,                   //Cached Modules
    erref_cache:Cache<ErrorRef, Rc<ResolvedErr>>,               //Cached Errors
    tref_cache:Cache<TypeRef, Crc<ResolvedType>>,               //Cached Types
    ctr_cache:Cache<TypeRef, Rc<Vec<Vec<Crc<ResolvedType>>>>>,  //Cached Constructors
    lit_cache:Cache<TypeRef, u16>,                              //Cached Literal sizes
    fref_cache:Cache<FuncRef, Rc<ResolvedFunction>>,            //Cached Functions
    f_sig_cache:Cache<FuncRef, Rc<ResolvedSignature>>,          //Cached Signatures
    sig_cache:Cache<TypeRef, Rc<ResolvedSignature>>,            //Cached Signatures

}


impl Cachings {
    //Generates a fresh Empty Cache
    pub fn new(ctx:&InputContext) -> Self {
        Cachings{
            mref_cache: Cache::new(ctx.num_modules()),
            erref_cache: Cache::new(ctx.num_error_imports()),
            tref_cache: Cache::new(ctx.num_types()),
            ctr_cache: Cache::new(ctx.num_types()),
            lit_cache: Cache::new(ctx.num_types()),
            fref_cache: Cache::new(ctx.num_function_imports()),
            f_sig_cache: Cache::new(ctx.num_function_imports()),
            sig_cache: Cache::new(ctx.num_types()),
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
    pub fn from_input_adt(adt:&'a DataComponent, link:Hash, store:&'b StorageCache<'b,S>) -> Result<Self> {
        Self::from_adt(adt,ModuleLink::This(link),store,true)
    }

    //Generates a top level local Context for an Adt fromminput
    pub fn from_store_adt(adt:&'a DataComponent, link:Hash, store:&'b StorageCache<'b,S>) -> Result<Self> {
        Self::from_adt(adt,ModuleLink::This(link),store,false)
    }

    fn from_adt(adt:&'a DataComponent, link:ModuleLink, store:&'b StorageCache<'b,S>, checking:bool) -> Result<Self> {
        //Get the input context for the Adt
        let ctx = InputContext::from_data(adt, link);
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

    pub fn from_top_func(fun:&'a FunctionComponent, store:&'b StorageCache<'b,S>) -> Result<Self> {
        Self::from_func(fun,None,store,true)
    }

    pub fn from_input_func(fun:&'a FunctionComponent, link:Hash, store:&'b StorageCache<'b,S>) -> Result<Self> {
        Self::from_func(fun,Some(ModuleLink::This(link)),store,true)
    }

    pub fn from_store_func(fun:&'a FunctionComponent, link:Hash, store:&'b StorageCache<'b,S>) -> Result<Self> {
        Self::from_func(fun,Some(ModuleLink::This(link)),store,false)
    }

    fn from_func(fun:&'a FunctionComponent, link:Option<ModuleLink>, store:&'b StorageCache<'b,S>, checking:bool) -> Result<Self> {
        //Get the input context for the Adt
        let ctx = InputContext::from_function(fun,link);
        //check that the size is ok (should be given)
        assert!(fun.shared.generics.len() <= u8::max_value() as usize);

        //Generate a local context
        Ok(Context {
            //The substitutions are just top level generics
            subs:top_level_subs(&fun.shared.generics),
            cache: Rc::new(Cachings::new(&ctx)),
            ctx,
            store,
            checking,
        })
    }

    pub fn from_input_sig(sig:&'a SigComponent, link:Hash, store:&'b StorageCache<'b,S>) -> Result<Self> {
        Self::from_sig(sig,ModuleLink::This(link),store,true)
    }

    pub fn from_store_sig(sig:&'a SigComponent, link:Hash, store:&'b StorageCache<'b,S>) -> Result<Self> {
        Self::from_sig(sig,ModuleLink::This(link),store,false)
    }

    fn from_sig(sig:&'a SigComponent, link:ModuleLink, store:&'b StorageCache<'b,S>, checking:bool) -> Result<Self> {
        //Get the input context for the Adt
        let ctx = InputContext::from_sig(sig,link);
        //check that the size is ok (should be given)
        assert!(sig.shared.generics.len() <= u8::max_value() as usize);

        //Generate a local context
        Ok(Context {
            //The substitutions are just top level generics
            subs:top_level_subs(&sig.shared.generics),
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
                        BaseType::Data(DataLink {module, offset}) => {
                            //Fetch the link
                            let module_link =  self.get_mod(*module)?;
                            //Load the Adt
                            let adt_cache = self.store.get_data_type(&*module_link, *offset)?;
                            // get the adt
                            let adt = adt_cache.retrieve();
                            //prepare the applies vector
                            let result:Vec<Crc<ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect::<Result<_>>()?;
                            //check that the type is legit
                            if self.checking { self.check_generic_constraints(&adt.generics,&result)?; }
                            //Resolve the type
                            Ok(self.resolve_data_type(adt, module_link, *offset, &result))
                        },
                        BaseType::Sig(SigLink{module, offset}) => {
                            //Fetch the link
                            let module_link =  self.get_mod(*module)?;
                            //Load the Sig
                            let sig_cache = self.store.get_sig(&*module_link,*offset)?;
                            // get the adt
                            let sig = sig_cache.retrieve();
                            //prepare the applies vector
                            let result:Vec<Crc<ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect::<Result<_>>()?;
                            //check that the type is legit
                            if self.checking {
                                self.check_generic_constraints(&sig.shared.generics,&result)?;
                                self.check_sig_generic_constraints(&sig.local_generics,&result)?;
                            }
                            Ok(self.resolve_sig_type(sig, module_link, *offset, &result))
                        }
                    }
                },
                //its an image type
                Some(Type::Image(inner)) => Ok(Crc::new(ResolvedType::Image { typ: self.get_type(*inner)? })),
                //its a generic, do a substitution
                Some(Type::Generic(gref)) => match self.subs.get(gref.0 as usize) {
                    None => type_does_not_exist_error(),
                    Some(res) => Ok(res.clone()),
                },
                //ups no such type
                None => type_does_not_exist_error()
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
                Some(err) => ResolvedErr{
                    offset: err.link.offset,
                    module: self.get_mod(err.link.module)?, //Get the Module
                },
                None => return error_does_not_exist_error(),
            }))
        })
    }

    //Gets and resolves an Function from the local context
    pub fn get_func(&self, fref:FuncRef) -> Result<Rc<ResolvedFunction>> {
        //only do this once and then cache it (detects cycles as well)
        self.cache.fref_cache.cached(fref, ||{
            //Get the Input Function
            let fun_imp = match self.ctx.get_function_import(fref.0) {
                //todo: more precise error
                None => return item_not_found(),
                Some(f) => f,
            };

            //Fetch the link
            let module_link =  self.get_mod(fun_imp.link.module)?;
            //Fetch the Cache Entry
            let fun_cache = self.store.get_func(&*module_link,fun_imp.link.offset)?;
            //Retrieve the function from the cache
            let fun= fun_cache.retrieve();
            // prepare the applies vector
            let result:Vec<Crc<ResolvedType>> = fun_imp.applies.iter().map(|appl|self.get_type(*appl)).collect::<Result<_>>()?;
            //check the amount of generics is ok
            if self.checking { self.check_generic_constraints(&fun.shared.generics,&result)?; }

            Ok(Rc::new(ResolvedFunction{
                offset: fun_imp.link.offset,
                module: module_link, //Get the Module
                //Resolve the applied types
                applies:result,
            }))
        })
    }
    
    

    //Gets a signature for a sig
    pub fn get_type_sig(&self, sref:TypeRef, store:&StorageCache<S>) -> Result<Rc<ResolvedSignature>> {
        //only do this once and then cache it (detects cycles as well)
        self.cache.sig_cache.cached(sref, ||{
            //Get the Input Function
            Ok(Rc::new(match *self.get_type(sref)? {
                ResolvedType::Generic { .. }
                | ResolvedType::Image { .. }
                | ResolvedType::Lit { .. }
                | ResolvedType::Data { .. } => return only_sig_types_have_fun_sigs_error(),
                ResolvedType::Sig { ref module, offset, ref applies, ..} => {
                    //Get the cache
                    let sig_cache = store.get_sig(&**module,offset)?;
                    //get the function
                    let sig = sig_cache.retrieve();
                    //Get its context
                    let context = sig_cache.substituted_context(applies.iter().map(|ra|ra.typ.clone()).collect(), store)?;
                    //Build the signature
                    ResolvedSignature {
                        //Map the params to the resolved type
                        params: sig.shared.params.iter().map(|c|{
                            let typ = context.get_type(c.typ)?;
                            Ok(ResolvedParam{ typ, consumes: c.consumes })
                        }).collect::<Result<_>>()?,
                        //Map the returns to the resolved type
                        returns: sig.shared.returns.iter().map(|r|{
                            let typ = context.get_type(r.typ)?;
                            Ok(ResolvedReturn{ typ, borrows: r.borrows.clone() })
                        }).collect::<Result<_>>()?,
                        //Map the risks to the resolved risks
                        risks: sig.shared.risk.iter().map(|err|context.get_err(*err)).collect::<Result<_>>()?,
                    }
                }

            }))
        })
    }
    
    //Get the ResolvedImplement
    pub fn get_func_impl(&self, fref:FuncRef, offset:u8, store:&StorageCache<S>) -> Result<Rc<ResolvedImpl>> {
        let r_fun = self.get_func(fref)?;
        //Get the cache
        let func_cache = store.get_func(&*r_fun.module,r_fun.offset)?;
        //get the function
        let fun = func_cache.retrieve();
        //check that impl exists
        if offset as usize >= fun.implements.len() {
            return impl_does_not_exist_error()
        }
        //get the impl
        let imp = &fun.implements[offset as usize];
        //Get its context
        let context = func_cache.substituted_context(r_fun.applies.clone(), store)?;
        //Get the sig type -- validator has checked that it matches
        let sig_type = context.get_type(imp.typ)?;
        //resolve the captures
        let capture_types = imp.captures.iter().map(|pos| match context.get_type(fun.shared.params[*pos as usize].typ){
            //wrap the types into Captures that now their position
            Ok(typ) => Ok(Rc::new(ResolvedCapture{ typ, pos:*pos })),
            Err(err) => Err(err)
        }).collect::<Result<_>>()?;
        //create it
        Ok(Rc::new(ResolvedImpl {
            sig_type,
            capture_types
        }))
    }

    //Gets and resolves a Function signature from the local context
    pub fn get_func_sig(&self, fref:FuncRef, store:&StorageCache<S>) -> Result<Rc<ResolvedSignature>> {
        //only do this once and then cache it (detects cycles as well)
        self.cache.f_sig_cache.cached(fref, ||{
            //Get the Input Function
            let r_fun = self.get_func(fref)?;
            //Get the cache
            let func_cache = store.get_func(&*r_fun.module,r_fun.offset)?;
            //get the function
            let fun = func_cache.retrieve();
            //Get its context
            let context = func_cache.substituted_context(r_fun.applies.clone(), store)?;
            //Build the signature
            Ok(Rc::new(ResolvedSignature {
                //Map the params to the resolved type
                params: fun.shared.params.iter().map(|c|{
                    let typ = context.get_type(c.typ)?;
                    Ok(ResolvedParam{ typ, consumes: c.consumes })
                }).collect::<Result<_>>()?,
                //Map the returns to the resolved type
                returns: fun.shared.returns.iter().map(|r|{
                    let typ = context.get_type(r.typ)?;
                    Ok(ResolvedReturn{ typ, borrows: r.borrows.clone() })
                }).collect::<Result<_>>()?,
                //Map the risks to the resolved risks
                risks: fun.shared.risk.iter().map(|err|context.get_err(*err)).collect::<Result<_>>()?,
            }))
        })
    }

    //Gets and resolves an Adt Constructor from the local context
    pub fn get_ctrs(&self, tref:TypeRef, store:&StorageCache<S>) -> Result<Rc<Vec<Vec<Crc<ResolvedType>>>>> {
        //helper so that the image case can share this code
        fn get_ctrs<S:Store>(typ:&Crc<ResolvedType>, store:&StorageCache<S>) -> Result<Vec<Vec<Crc<ResolvedType>>>> {
            Ok(match **typ {
                ResolvedType::Generic { .. } => return generics_dont_have_ctrs_error(),
                ResolvedType::Sig { .. } => return sigs_dont_have_ctrs_error(),
                ResolvedType::Lit { .. } => return no_ctr_available(), //todo: better error
                //If an image then its ctr is the ctr of the inner but with image fields
                ResolvedType::Image { ref typ} => {
                    //get the inners Ctr
                    let inner = get_ctrs(typ,store)?;
                    //wrap the fields in Images
                    inner.into_iter().map(|ctr|ctr.into_iter().map(|field|Crc::new(ResolvedType::Image {typ:field})).collect()).collect()
                }
                //If an import we have to import it

                ResolvedType::Data { ref module , offset, ref applies, .. } => {
                    //Get the cache
                    let adt_cache = store.get_data_type(&**module, offset)?;
                    //Get the adt
                    let adt = adt_cache.retrieve();

                    match adt.body {
                        //todo: really unreachable i do not think so
                        DataImpl::Lit(_) | DataImpl::ExternalLit(_,_) | DataImpl::ExternalAdt(_) => unreachable!(),
                        DataImpl::Adt { ref constructors} => {
                            //get its context with the applies as substitutions
                            let context = adt_cache.substituted_context(applies.iter().map(|apl|apl.typ.clone()).collect(),store)?;
                            //build the ctrs by retriving their fields
                            constructors.iter().map(|c|{
                                c.fields.iter().map(|t|context.get_type(*t)).collect::<Result<_>>()
                            }).collect::<Result<_>>()?
                        }
                    }
                }
            })
        }
        //only do this once and then cache it (detects cycles as well)
        self.cache.ctr_cache.cached(tref, ||{
            //Get the Input Type and extract ctrs
            Ok(Rc::new(get_ctrs(&self.get_type(tref)?, store)?))
        })
    }

    //Gets and resolves an Adt Constructor from the local context
    pub fn get_lit_size(&self, tref:TypeRef, store:&StorageCache<S>) -> Result<u16> {
        fn get_lit<S:Store>(typ:&Crc<ResolvedType>, store:&StorageCache<S>) -> Result<u16> {
            //Get the Input Type and extract lits
            Ok(match **typ {
                ResolvedType::Generic { .. } => return literal_data_error(), //todo: better error
                ResolvedType::Sig { .. } => return literal_data_error(), //todo: better error
                ResolvedType::Data { .. } => return literal_data_error(), //todo: better error
                //If an image then its ctr is the ctr of the inner but with image fields
                ResolvedType::Image { ref typ} => get_lit(typ,store)?,
                //If an lit we have to import it
                ResolvedType::Lit { ref module , offset, ref applies, .. } => {
                    //Get the cache
                    let adt_cache = store.get_data_type(&**module, offset)?;
                    //Get the adt
                    let adt = adt_cache.retrieve();

                    match adt.body {
                        //todo: really unreachable i do not think so
                        DataImpl::Adt{..} | DataImpl::ExternalAdt(_) => unreachable!(),
                        DataImpl::Lit(size) |DataImpl::ExternalLit(_,size) => size
                    }
                }
            })
        }

        //only do this once and then cache it (detects cycles as well)
        self.cache.lit_cache.cached(tref, ||{
            //Get the Input Type and extract size
            Ok(get_lit(&self.get_type(tref)?, store)?)
        })
    }

    fn check_generic_constraints(&self, generics:&[Generic], applies:&[Crc<ResolvedType>]) -> Result<()>{
        // check that the number of applies is correct
        if generics.len() != applies.len() {
            return num_applied_generics_error();
        }

        for (generic,typ) in  generics.iter().zip(applies.iter()) {
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

    fn check_sig_generic_constraints(&self, is_local:&[u8], applies:&[Crc<ResolvedType>]) -> Result<()>{
        //check that the locals are local
        for local in is_local{
            if *local as usize >= applies.len() {
                return item_not_found();
            }
            if !applies[*local as usize].is_local() {
                return can_not_apply_non_local_error()
            }
        }
        Ok(())
    }

    fn resolve_data_type(&self, adt:&DataComponent, module:Rc<ModuleLink>, offset:u8, applies:&[Crc<ResolvedType>]) -> Crc<ResolvedType>{
        // calc the caps after application & check constraints
        //prepare the applies vector
        let are_phantom = adt.generics.iter().map(|generic|match *generic {
            Generic::Phantom => true,
            Generic::Physical(_) => false,
        });
        let (extended_caps,caps,result) = apply_types(adt.provided_caps,are_phantom,applies);
        //Construct the Type
        match adt.body {
            DataImpl::Adt { .. }
            | DataImpl::ExternalAdt(_) => {
                Crc::new(ResolvedType::Data {
                    caps,
                    extended_caps,
                    base_caps:adt.provided_caps,
                    module,
                    offset,
                    applies:result,
                })
            },
            DataImpl::ExternalLit(_, size)
            | DataImpl::Lit(size) => {
                Crc::new(ResolvedType::Lit {
                    caps,
                    extended_caps,
                    base_caps:adt.provided_caps,
                    module,
                    offset,
                    size,
                    applies:result,
                })
            },
        }
    }

    fn resolve_sig_type(&self, sig:&SigComponent, module:Rc<ModuleLink>, offset:u8,  applies:&[Crc<ResolvedType>]) -> Crc<ResolvedType>{
        // calc the caps after application & check constraints
        //prepare the applies vector
        let are_phantom = sig.shared.generics.iter().map(|generic|match *generic {
            Generic::Phantom => true,
            Generic::Physical(_) => false,
        });
        let (extended_caps,caps,result) = apply_types(sig.provided_caps,are_phantom,applies);
        //Construct the Type
        Crc::new(ResolvedType::Sig {
            caps,
            extended_caps,
            base_caps:sig.provided_caps,
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