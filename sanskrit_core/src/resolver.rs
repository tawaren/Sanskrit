use sanskrit_common::errors::*;
use crate::loader::Loader;
use sanskrit_common::utils::Crc;
use crate::model::resolved::*;
use crate::model::*;
use alloc::vec::Vec;
use crate::model::linking::{Component, CallableComponent, FastModuleLink, Link};
use alloc::rc::Rc;
use sanskrit_common::supplier::Supplier;
use crate::model::bitsets::{CapSet, BitSet};

//All things that can be cached in an import
pub struct CachedImports {
    mref_cache:Vec<FastModuleLink>,                  //Cached Modules
    tref_cache:Vec<Crc<ResolvedType>>,               //Cached Types
    pref_cache:Vec<Crc<ResolvedPermission>>,
    cref_cache:Vec<Crc<ResolvedCallable>>,           //Cached Callables
}


impl CachedImports {
    //Generates a fresh Empty Cache
    pub fn new(imports:&[Imports]) -> Result<Self> {
        let mut num_modules = 0;
        let mut num_types = 0;
        let mut num_permission = 0;
        let mut num_callables = 0;

        for imp in imports {
            match imp {
                Imports::Module(_) => num_modules+=1,
                Imports::Generics(gens) => num_types+=gens.len(),
                Imports::Public(public) => {
                    num_modules+=public.modules.len();
                    num_types+=public.types.len();
                },
                Imports::Body(body) => {
                    num_modules+=body.public.modules.len();
                    num_types+=body.public.types.len();
                    num_permission+=body.permissions.len();
                    num_callables+=body.callables.len();
                },
            }
        }

        if num_modules > 256 {
            return error(||"Can not import more than 256 Modules")
        }

        if num_types > 256 {
            return error(||"Can not import more than 256 Types")
        }

        if num_permission > 256 {
            return error(||"Can not import more than 256 Permisisons")
        }

        if num_callables > 256 {
            return error(||"Can not import more than 256 Callables")
        }

        Ok(CachedImports {
            mref_cache: Vec::with_capacity(num_modules),
            tref_cache: Vec::with_capacity(num_types),
            pref_cache: Vec::with_capacity(num_permission),
            cref_cache: Vec::with_capacity(num_callables)
        })
    }
}

pub struct Context<'b, S: Supplier<Module> + 'b> {
    //The resolved imports
    pub cache: CachedImports,
    //The Backend Store
    pub store: &'b Loader<'b, S>,
}


pub fn top_level_subs(generics:&[Generic]) -> Vec<Crc<ResolvedType>> {
    //Note: We do not account / deduplicate for the generics as these are:
    //         1: implicit
    //         2: unique (each is its own and should not be deduplicated)
    //            meaning 2 generics with the same offset are different
    //              unless they are the exact same generic (used in the same body)
    generics.iter().enumerate().map(|(i,c)| match *c {
        Generic::Phantom => Crc{elem:Rc::new(ResolvedType::Generic { caps: CapSet::empty(), offset:i as u8, is_phantom:true})},
        Generic::Physical(caps) => {
            Crc{elem:Rc::new(ResolvedType::Generic { caps, offset:i as u8, is_phantom:false})}
        },
    }).collect()
}

impl<'b, S:Supplier<Module> + 'b> Context<'b, S> {

    //Generates a top level local Context for a Component from input
    pub fn from_module_component<T:Component>(comp:&T, module:&FastModuleLink, use_body:bool, store:&'b Loader<'b,S>) -> Result<Self> {
        assert!(comp.get_generics().len() <= u8::MAX as usize);
        let top = top_level_subs(comp.get_generics());
        let public = comp.get_public_import();
        if use_body {
            match comp.get_body_import() {
                None => { }
                Some(body) => {
                    //Generate a local context
                    return Self::create_and_resolve(&[
                            Imports::Module(module),
                            Imports::Generics(&top),
                            Imports::Public(public),
                            Imports::Body(body),
                        ], store)
                },
            }
        }
        //Generate a local context
        return Self::create_and_resolve(&[
                Imports::Module(module),
                Imports::Generics(&top),
                Imports::Public(public),
            ], store)
    }

    //Generates a top level local Context for a standalone Component from Input
    pub fn from_top_component<T:Component>(comp:&T, store:&'b Loader<'b,S>) -> Result<Self> {
        assert_eq!(comp.get_generics().len(), 0);
        let public = comp.get_public_import();
        match comp.get_body_import() {
            None => { }
            Some(body) => {
                return Self::create_and_resolve(&[
                        Imports::Public(public),
                        Imports::Body(body)
                    ], store)
            },
        }
        return Self::create_and_resolve(&[Imports::Public(public)], store)
    }

    //Gets and resolves a Module from the local context
    pub fn get_mod(&self, mref:ModRef) -> Result<FastModuleLink> {
        unpack_or_error(self.cache.mref_cache.get(mref.0 as usize).cloned(),||"Requested module not imported")
    }

    pub fn get_type(&self, tref:TypeRef) -> Result<Crc<ResolvedType>> {
        unpack_or_error(self.cache.tref_cache.get(tref.0 as usize).cloned(),||"Requested type not imported")
    }

    pub fn get_callable(&self, cref:CallRef) -> Result<Crc<ResolvedCallable>> {
        unpack_or_error(self.cache.cref_cache.get(cref.0 as usize).cloned(),||"Requested callable not imported")
    }

    //Gets and resolves a type from the local context
    pub fn get_perm(&self, pref:PermRef) -> Result<Crc<ResolvedPermission>> {
        unpack_or_error(self.cache.pref_cache.get(pref.0 as usize).cloned(),||"Requested permission not imported")
    }

    //Gets and resolves a Module from the local context
    pub fn list_mods(&self) -> &[FastModuleLink] {
        &self.cache.mref_cache
    }

    pub fn list_types(&self) -> &[Crc<ResolvedType>] {
        &self.cache.tref_cache
    }

    pub fn list_callables(&self) -> &[Crc<ResolvedCallable>] {
        &self.cache.cref_cache
    }

    //Gets and resolves a type from the local context
    pub fn list_perms(&self) -> &[Crc<ResolvedPermission>] {
        &self.cache.pref_cache
    }

    pub fn create_and_resolve(imports:&[Imports], store:&'b Loader<'b,S>) -> Result<Self> {
        let mut context = Context {
            cache: CachedImports::new(&imports)?,
            store,
        };
        context.resolve_all(imports)?;
        Ok(context)
    }

    fn resolve_all(&mut self, imports:&[Imports]) -> Result<()> {
        for import in imports {
            match *import {
                Imports::Module(module) => {
                    self.resolve_module(module.clone())?;
                },
                Imports::Generics(gens) => {
                    self.resolve_generics(gens)?;
                },
                Imports::Public(public) => {
                    self.resolve_modules(&public.modules)?;
                    self.resolve_types(&public.types)?;
                },
                Imports::Body(body) => {
                    self.resolve_modules(&body.public.modules)?;
                    self.resolve_types(&body.public.types)?;
                    self.resolve_callables(&body.callables)?;
                    self.resolve_perms(&body.permissions)?;
                },
            }
        }
        Ok(())
    }

    fn resolve_module(&mut self, res:FastModuleLink) -> Result<()> {
        //Make sure the module is loaded and accounted even if not used
        if !res.is_local_link() { res.resolve(self)?; };
        self.cache.mref_cache.push(res);
        Ok(())
    }

    fn resolve_generics(&mut self, generics:&[Crc<ResolvedType>]) -> Result<()>  {
        for gens in generics {
            self.cache.tref_cache.push(gens.clone());
        }
        Ok(())
    }

    fn resolve_modules(&mut self, imps:&[FastModuleLink]) -> Result<()>  {
        for imp in imps {
            self.resolve_module(imp.clone())?;
        }
        Ok(())
    }

    fn resolve_types(&mut self, imps:&[TypeImport]) -> Result<()> {
        for imp in imps {
            let res = match imp {
                TypeImport::Data{ link:DataLink {module, offset}, applies} => self.resolve_data_type(*module, *offset, &applies),
                //Its a general
                TypeImport::Sig{link:SigLink{module, offset}, applies} => self.resolve_sig_type(*module, *offset, applies),
                //its an image type
                TypeImport::Projection{typ} => self.resolve_projection_type(*typ),
                //A special virtual type
                TypeImport::Virtual(hash) => Ok(self.store.dedup_type(ResolvedType::Virtual(*hash))),
            }?;
            self.cache.tref_cache.push(res);
        }
        Ok(())
    }

    fn resolve_callables(&mut self, imps:&[CallableImport]) -> Result<()> {
        for imp in imps {
            let res = match imp {
                CallableImport::Function { link: FuncLink{module, offset}, ref applies,.. } => self.resolve_function_callable(*module,*offset, applies),
                CallableImport::Implement { link: ImplLink{module, offset}, ref applies,.. } => self.resolve_implement_callable(*module,*offset, applies),
            }?;
            self.cache.cref_cache.push(res);
        }
        Ok(())
    }

    fn resolve_perms(&mut self, imps:&[PermissionImport]) -> Result<()> {
        for imp in imps {
            let res = match imp {
                PermissionImport::Type(perm,tref) => {
                    let typ = self.get_type(*tref)?;
                    match *typ.get_target() {
                        ResolvedType::Data { .. } => self.store.dedup_permission(ResolvedPermission::TypeData{
                            perm: *perm,
                            ctrs:self.resolve_ctr_from_type(&typ)?,
                            typ
                        }),
                        ResolvedType::Sig { .. } => self.store.dedup_permission(ResolvedPermission::TypeSig{
                            perm: *perm,
                            signature:self.resolve_signature_from_type(&typ)?,
                            typ
                        }),
                        ResolvedType::Lit { .. } =>  self.store.dedup_permission(ResolvedPermission::TypeLit{
                            perm: *perm,
                            size:self.resolve_size_from_type(&typ)?,
                            typ
                        }),
                        _ => return error(||"Provided type does not support Permissions")
                    }
                },
                PermissionImport::Callable(perm, cref) => {
                    let fun = self.get_callable(*cref)?;
                    self.store.dedup_permission(ResolvedPermission::FunSig{
                        perm: *perm,
                        signature:self.resolve_signature_from_callable(&fun)?,
                        fun
                    })
                }
            };
            self.cache.pref_cache.push(res);
        }
        Ok(())
    }

    fn resolve_data_type(&self, module:ModRef, offset:u8, applies:&[TypeRef]) -> Result<Crc<ResolvedType>>{
        //Fetch the link
        let module_link =  self.get_mod(module)?;
        //Load the Adt
        let adt_cache = self.store.get_component::<DataComponent>(&module_link, offset)?;
        // get the adt
        let adt = adt_cache.retrieve();
        //prepare the applies vector
        let result:Vec<Crc<ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect::<Result<_>>()?;
        //check that the number of generics match
        //Note we do this here so nobody can cause extra iterations in are_phantom & apply_types
        if adt.generics.len() != applies.len() {
            return error(||"Applied types mismatch required generics")
        }
        //Resolve the type
        // calc the caps after application & check constraints
        //prepare the applies vector
        let are_phantom = adt.generics.iter().map(|generic|match *generic {
            Generic::Phantom => true,
            Generic::Physical(_) => false,
        });
        let (generic_caps,caps) = apply_types(adt.provided_caps,are_phantom,&result);

        //Construct the Type
        match adt.body {
            DataImpl::Internal { .. } => {
                Ok(self.store.dedup_type(ResolvedType::Data {
                    caps,
                    generic_caps,
                    module:module_link,
                    offset,
                    applies:result,
                }))
            },
            DataImpl::External(size) => {
                Ok(self.store.dedup_type(ResolvedType::Lit {
                    caps,
                    generic_caps,
                    module:module_link,
                    offset,
                    size,
                    applies:result,
                }))
            },
        }
    }

    fn resolve_sig_type(&self, module:ModRef, offset:u8,  applies:&[TypeRef]) -> Result<Crc<ResolvedType>>{
        //build the type
        //Fetch the link
        let module_link = self.get_mod(module)?;
        //Load the Sig
        let sig_cache = self.store.get_component::<SigComponent>(&module_link,offset)?;
        // get the function
        let sig = sig_cache.retrieve();
        //prepare the applies vector
        let result:Vec<Crc<ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect::<Result<_>>()?;
        //check that the number of generics match
        //Note we do this here so nobody can cause extra iterations in are_phantom & apply_types
        if sig.shared.generics.len() != applies.len() {
            return error(||"Applied types mismatch required generics")
        }

        //Signatures ignore the caps of generics as a signature does not store values of these type (just take them as param & return them)
        // A Signature can Capture (store) these types but captures are checked on implement side so that they are covered by the signatures capabilities
        //  This is needed any way as they could contain anything with any cap (like adt fields)
        //   But unlike adt fields we can not abstract over them as they may depend on things not declared in the Signature
        //let (generic_caps,caps) = apply_types(sig.provided_caps,are_phantom,&result);

        //Construct the Type
        Ok(self.store.dedup_type(ResolvedType::Sig {
            caps: sig.provided_caps,
            module:module_link,
            offset,
            applies:result
        }))
    }

    fn project(&self, inner:&Crc<ResolvedType>, nesting:u8) -> Crc<ResolvedType>{
        //Resolve the type
        let depth = match **inner {
            ResolvedType::Projection { depth, .. } => depth+nesting,
            _ => nesting
        };
        self.store.dedup_type(ResolvedType::Projection { depth, un_projected:get_crc_target(inner).clone()})
    }

    fn resolve_projection_type(&self, inner:TypeRef) -> Result<Crc<ResolvedType>>{
        //get the inner type
        Ok(self.project(&self.get_type(inner)?, 1))
    }

    fn resolve_function_callable(&self, module:ModRef, offset:u8,  applies:&[TypeRef]) -> Result<Crc<ResolvedCallable>>{
        //Fetch the link
        let module_link =  self.get_mod(module)?;
        // prepare the applies vector
        let result:Vec<Crc<ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect::<Result<_>>()?;

        Ok(self.store.dedup_callable(ResolvedCallable::Function {
            offset,
            module: module_link, //Get the Module
            //Resolve the applied types
            applies:result
        }))
    }

    fn resolve_implement_callable(&self, module:ModRef, offset:u8,  applies:&[TypeRef]) -> Result<Crc<ResolvedCallable>>{
        //Fetch the link
        let module_link =  self.get_mod(module)?;
        // prepare the applies vector
        let result:Vec<Crc<ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect::<Result<_>>()?;

        Ok(self.store.dedup_callable(ResolvedCallable::Implement {
            offset,
            module: module_link, //Get the Module
            //Resolve the applied types
            applies:result
        }))
    }


    fn resolve_signature_from_component<C:CallableComponent>(&self, module:&FastModuleLink, offset:u8, applies:&[Crc<ResolvedType>]) -> Result<Crc<ResolvedSignature>> {
        //Load the Comp
        let comp_cache = self.store.get_component::<C>(module, offset)?;
        // get the Comp
        let comp = comp_cache.retrieve();
        //get its context with the applies as substitutions
        let context = comp_cache.substituted_context(&applies,&self.store)?;
        //Create the sig
        Ok(self.store.dedup_signature(ResolvedSignature {
            //Map the params to the resolved type
            params: comp.get_params().iter().map(|c|{
                let typ = context.get_type(c.typ)?;
                Ok(ResolvedParam{ typ, consumes: c.consumes })
            }).collect::<Result<_>>()?,
            //Map the returns to the resolved type
            returns: comp.get_returns().iter().map(|rt|{
                context.get_type(*rt)
            }).collect::<Result<_>>()?,
            transactional: comp.is_transactional()
        }))
    }

    fn resolve_signature_from_callable(&self, fun:&Crc<ResolvedCallable>) -> Result<Crc<ResolvedSignature>> {
        match **fun {
            ResolvedCallable::Function {  ref module, offset, ref applies, .. } => {
                //create the sig
                self.resolve_signature_from_component::<FunctionComponent>(module, offset, applies)
            },
            ResolvedCallable::Implement {  ref module, offset, ref applies, .. } => {
                //create the sig
                self.resolve_signature_from_component::<ImplementComponent>(module, offset, applies)
            }
        }
    }

    fn resolve_signature_from_type(&self, typ:&Crc<ResolvedType>) -> Result<Crc<ResolvedSignature>> {
        match **typ {
            ResolvedType::Sig { ref module, offset, ref applies, .. }  => {
                //create the sig
                self.resolve_signature_from_component::<SigComponent>(module, offset, applies)
            },

            ResolvedType::Generic { .. }
            | ResolvedType::Projection { .. }
            | ResolvedType::Lit { .. }
            | ResolvedType::Data { .. }
            | ResolvedType::Virtual(_) => error(||"Only sig types and its projection have signatures ")
        }
    }


    fn resolve_size_from_type(&self, typ:&Crc<ResolvedType>) -> Result<u16> {
        match **typ {
            ResolvedType::Lit { size, .. } => Ok(size),
            ResolvedType::Projection { ref un_projected, .. } => match **un_projected {
                ResolvedType::Lit { size, .. } => Ok(size),
                _ => error(||"Only literals and its projection have sizes")
            },
            _ => error(||"Only literals and its projection have sizes")
        }
    }

    fn resolve_ctr_from_type(&self, typ:&Crc<ResolvedType>) -> Result<Crc<Vec<Vec<Crc<ResolvedType>>>>> {
        match **typ {
            ResolvedType::Data { ref module, offset, ref applies, .. }  => {
                //Load the Adt
                let adt_cache = self.store.get_component::<DataComponent>(module, offset)?;
                // get the adt
                let adt = adt_cache.retrieve();
                //get its context with the applies as substitutions
                let context = adt_cache.substituted_context(&applies,&self.store)?;
                //Create the Ctr
                match adt.body {
                    DataImpl::Internal { ref constructors, .. } => {
                        Ok(self.store.dedup_ctr(constructors.iter().map(|c|{
                            c.fields.iter().map(|t|context.get_type(t.typ)).collect::<Result<_>>()
                        }).collect::<Result<_>>()?))
                    }
                    DataImpl::External(_) => error(||"Extrnal data does not have ctrs")
                }
            },
            ResolvedType::Projection { depth, ref un_projected, .. } => {
                assert!(if let ResolvedType::Projection{..} = **un_projected {false} else {true});
                //recurses only once (assert checks assumption)
                let ctrs = self.resolve_ctr_from_type(un_projected)?;
                Ok(self.store.dedup_ctr(ctrs.iter().map(|cases| cases.iter().map(|field|{
                    self.project(field, depth)
                }).collect()).collect()))
            },

            ResolvedType::Generic { .. }
            | ResolvedType::Lit { .. }
            | ResolvedType::Sig { .. }
            | ResolvedType::Virtual(_) => error(||"Only data and its projection have ctrs ")
        }
    }
}

fn unpack_or_error<T,F:FnOnce()-> &'static str>(opt:Option<T>, msg:F) -> Result<T>{
    match opt {
        None => error(msg),
        Some(t) => Ok(t),
    }
}


pub fn apply_types(base_caps:CapSet, are_phantom:impl Iterator<Item=bool>, applies:&[Crc<ResolvedType>]) -> (CapSet,CapSet) {
    //initial caps
    let mut generic_caps = base_caps;
    let mut caps = base_caps;
    for (is_phantom,typ) in  are_phantom.zip(applies.iter()) {
        //update caps
        if !is_phantom{
            //combine caps
            generic_caps = generic_caps.intersect(typ.get_generic_caps());
            caps = caps.intersect(typ.get_caps());
        }
    }
    (generic_caps,caps)
}
