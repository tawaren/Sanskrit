use crate::loader::{StateManager, Loader, ResolvedCtrs};
use crate::model::resolved::*;
use crate::model::*;
use alloc::vec::Vec;
use crate::model::linking::{Component, CallableComponent, FastModuleLink};
use sp1_zkvm_col::arena::{Embedding, URef};
use crate::model::bitsets::{CapSet, BitSet};

//All things that can be cached in an import
pub struct CachedImports {
    mref_cache:Vec<FastModuleLink>,                           //Cached Modules
    tref_cache:Vec<URef<'static,ResolvedType>>,               //Cached Types
    pref_cache:Vec<URef<'static,ResolvedPermission>>,
    cref_cache:Vec<URef<'static,ResolvedCallable>>,           //Cached Callables
}

impl CachedImports {
    //Generates a fresh Empty Cache
    pub fn new(/*imports:&[Imports]*/) -> Self {
            /*let mut num_modules = 0;
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
            //will be rechecked after initialisation in constrained mode
            assert!(num_modules <= 256, "Can not import more than 256 Modules");
            assert!(num_types <= 256, "Can not import more than 256 Types");
            assert!(num_permission <= 256, "Can not import more than 256 Permisisons");
            assert!(num_callables <= 256, "Can not import more than 256 Callables");
            */
        //let res = read_vec();
        //assert!(res.len() == 4);
        CachedImports {
            mref_cache: Vec::with_capacity(256),
            tref_cache: Vec::with_capacity(256),
            pref_cache: Vec::with_capacity(256),
            cref_cache: Vec::with_capacity(256)
        }
    }

    fn size_check(&self) {
        assert!(self.mref_cache.len() <= 256);
        assert!(self.tref_cache.len() <= 256);
        assert!(self.pref_cache.len() <= 256);
        assert!(self.cref_cache.len() <= 256);
    }
}

pub struct Context<'b, S:StateManager> {
    //The resolved imports
    pub cache: CachedImports,
    //The Backend Store
    pub store: &'b Loader<S>,
}

pub fn top_level_subs<S:StateManager>(store: &Loader<S>, generics:&[Generic]) -> Vec<URef<'static,ResolvedType>> {
    //Note: We do not account / deduplicate for the generics as these are:
    //         1: implicit
    //         2: unique (each is its own and should not be deduplicated)
    //            meaning 2 generics with the same offset are different
    //              unless they are the exact same generic (used in the same body)
    generics.iter().enumerate().map(|(i,c)| match *c {
        Generic::Phantom => store.create_generic_type(ResolvedType::Generic {caps: CapSet::empty(), offset:i as u8, is_phantom:true}),
        Generic::Physical(caps) => {
            store.create_generic_type(ResolvedType::Generic { caps, offset:i as u8, is_phantom:false})
        },
    }).collect()
}

impl<'a, S:StateManager> Context<'a,S> {
    //Generates a top level local Context for a Component from input
    pub fn from_module_component<T: Component>(comp: &T, module: &FastModuleLink, use_body: bool, store: &'a Loader<S>) -> Self {
        assert!(comp.get_generics().len() <= u8::MAX as usize);
        let top = top_level_subs(store, comp.get_generics());
        let public = comp.get_public_import();
        if use_body {
            match comp.get_body_import() {
                None => {}
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
    pub fn from_top_component<T: Component>(comp: &T, store: &'a Loader<S>) -> Self {
        assert_eq!(comp.get_generics().len(), 0);
        let public = comp.get_public_import();
        match comp.get_body_import() {
            None => {}
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
    pub fn get_mod(&self, mref: ModRef) -> FastModuleLink {
        self.cache.mref_cache[mref.0 as usize]
    }

    pub fn get_type(&self, tref: TypeRef) -> URef<'static,ResolvedType> {
        self.cache.tref_cache[tref.0 as usize]
    }

    pub fn get_callable(&self, cref: CallRef) -> URef<'static,ResolvedCallable> {
        self.cache.cref_cache[cref.0 as usize]
    }

    //Gets and resolves a type from the local context
    pub fn get_perm(&self, pref: PermRef) -> URef<'static,ResolvedPermission> {
        self.cache.pref_cache[pref.0 as usize]
    }

    //Gets and resolves a Module from the local context
    pub fn list_mods(&self) -> &[FastModuleLink] {
        &self.cache.mref_cache
    }

    pub fn list_types(&self) -> &[URef<'static,ResolvedType>] {
        &self.cache.tref_cache
    }

    pub fn list_callables(&self) -> &[URef<'static,ResolvedCallable>] {
        &self.cache.cref_cache
    }

    //Gets and resolves a type from the local context
    pub fn list_perms(&self) -> &[URef<'static,ResolvedPermission>] {
        &self.cache.pref_cache
    }

    pub fn create_and_resolve(imports: &[Imports], store: &'a Loader<S>) -> Self {
        let mut context = Context {
            cache: CachedImports::new(/*&imports*/),
            store,
        };
        context.resolve_all(imports);
        context.cache.size_check();
        context
    }

    fn resolve_all(&mut self, imports: &[Imports]) {
        for import in imports {
            match *import {
                Imports::Module(module) => {
                    self.resolve_module(module);
                },
                Imports::Generics(gens) => {
                    self.resolve_generics(gens);
                },
                Imports::Public(public) => {
                    self.resolve_modules(&public.modules);
                    self.resolve_types(&public.types);
                },
                Imports::Body(body) => {
                    self.resolve_modules(&body.public.modules);
                    self.resolve_types(&body.public.types);
                    self.resolve_callables(&body.callables);
                    self.resolve_perms(&body.permissions);
                },
            }
        }
    }

    pub fn is_this_module(&self, target: &FastModuleLink) -> bool {
        self.store.is_this_module(target)
    }

    pub fn is_local_type(&self, target: URef<'static,ResolvedType>) -> bool {
        self.store.is_local_type(target)
    }

    fn resolve_module(&mut self, res: &FastModuleLink) {
        //Make sure the module is loaded and accounted even if not used
        if !self.is_this_module(res) { res.resolve(self); };
        self.cache.mref_cache.push(*res);
    }

    fn resolve_generics(&mut self, generics: &[URef<'static,ResolvedType>]){
        for gens in generics {
            self.cache.tref_cache.push(*gens);
        }
    }

    fn resolve_modules(&mut self, imps: &[FastModuleLink]) {
        for imp in imps {
            self.resolve_module(imp);
        }
    }

    fn resolve_types(&mut self, imps: &[TypeImport]) {
        for imp in imps {
            let res= match imp {
                TypeImport::Data { link: DataLink { module, offset }, applies } => self.resolve_data_type(*module, *offset, applies),
                //Its a general
                TypeImport::Sig { link: SigLink { module, offset },  applies } => self.resolve_sig_type(*module, *offset, applies),
                //its an image type
                TypeImport::Projection { typ } => self.resolve_projection_type(*typ),
                //A special virtual type
                TypeImport::Virtual(hash) => self.store.virtual_type_dedup(ResolvedType::Virtual(*hash)),
            };
            self.cache.tref_cache.push(res);
        }
    }

    fn resolve_callables(&mut self, imps: &[CallableImport]) {
        for imp in imps {
            let res = match imp {
                CallableImport::Function { link: FuncLink { module, offset }, ref applies, .. } => self.resolve_function_callable(*module, *offset, applies),
                CallableImport::Implement { link: ImplLink { module, offset }, ref applies, .. } => self.resolve_implement_callable(*module, *offset, applies),
            };
            self.cache.cref_cache.push(res);
        }
    }

    fn resolve_perms(&mut self, imps: &[PermissionImport])  {
        for imp in imps {
            let res = match imp {
                PermissionImport::Type(perm, tref) => {
                    let typ = self.get_type(*tref);
                    match *typ.get_target() {
                        ResolvedType::Data { .. } => self.store.dedup_permission(ResolvedPermission::TypeData {
                            perm: *perm,
                            ctrs: self.resolve_ctr_from_type(typ),
                            typ
                        }),
                        ResolvedType::Sig { .. } => self.store.dedup_permission(ResolvedPermission::TypeSig {
                            perm: *perm,
                            signature: self.resolve_signature_from_type(typ),
                            typ
                        }),
                        ResolvedType::Lit { .. } => self.store.dedup_permission(ResolvedPermission::TypeLit {
                            perm: *perm,
                            size: self.resolve_size_from_type(typ),
                            typ
                        }),
                        _ => panic!("Provided type does not support Permissions")
                    }
                },
                PermissionImport::Callable(perm, cref) => {
                    let fun = self.get_callable(*cref);
                    self.store.dedup_permission(ResolvedPermission::FunSig {
                        perm: *perm,
                        signature: self.resolve_signature_from_callable(fun),
                        fun
                    })
                }
            };
            self.cache.pref_cache.push(res);
        }
    }
}

impl Embedding<ResolvedType> for ResolvedComponent {
    type Auxiliary = DataComponent;
    fn embed(self, adt:&DataComponent) -> ResolvedType {
        //Resolve the type
        // calc the caps after application & check constraints
        //prepare the applies vector
        let are_phantom = adt.generics.iter().map(|generic|match *generic {
            Generic::Phantom => true,
            Generic::Physical(_) => false,
        });
        let (generic_caps,caps) = apply_types(adt.provided_caps,are_phantom,&self.applies);

        //Construct the Type
        match adt.body {
            DataImpl::Internal { .. } => {
                ResolvedType::Data {
                    caps,
                    generic_caps,
                    base:self,
                }
            },
            DataImpl::External(size) => {
                ResolvedType::Lit {
                    caps,
                    generic_caps,
                    base:self,
                    size,
                }
            },
        }
    }

    fn extract(embedded: &ResolvedType) -> &Self {
       match embedded {
           ResolvedType::Data { base, .. }
           | ResolvedType::Lit { base, .. } => base,
           _ => panic!("unexpected type")
       }
    }
}

//New start to have the Embeding near the use
impl<'a, S:StateManager> Context<'a, S> {

    fn resolve_data_type(&self, module:ModRef, offset:u8, applies:&[TypeRef]) -> URef<'static,ResolvedType>{
        //Fetch the link
        let module_link =  self.get_mod(module);
        //Load the Adt
        let adt_cache = self.store.get_component::<DataComponent>(&module_link, offset);
        // get the adt
        let adt = adt_cache.retrieve();
        //prepare the applies vector
        let result:Vec<URef<'static,ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect();
        //check that the number of generics match
        //Note we do this here so nobody can cause extra iterations in are_phantom & apply_types
        assert!(adt.generics.len() == applies.len());
        //Check if we already have it
        self.store.data_type_dedup(ResolvedComponent{
            module: module_link,
            offset,
            applies:result
        }, adt)
    }

    fn resolve_sig_type(&self, module:ModRef, offset:u8,  applies:&[TypeRef]) -> URef<'static,ResolvedType>{
        //build the type
        //Fetch the link
        let module_link = self.get_mod(module);
        //Load the Sig
        let sig_cache = self.store.get_component::<SigComponent>(&module_link,offset);
        // get the function
        let sig = sig_cache.retrieve();
        //prepare the applies vector
        let result:Vec<URef<'static,ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect();
        //check that the number of generics match
        //Note we do this here so nobody can cause extra iterations in are_phantom & apply_types
        assert!(sig.shared.generics.len() == applies.len());
        //Check if we already have it
        //Signatures ignore the caps of generics as a signature does not store values of these type (just take them as param & return them)
        // A Signature can Capture (store) these types but captures are checked on implement side so that they are covered by the signatures capabilities
        //  This is needed any way as they could contain anything with any cap (like adt fields)
        //   But unlike adt fields we can not abstract over them as they may depend on things not declared in the Signature
        //let (generic_caps,caps) = apply_types(sig.provided_caps,are_phantom,&result);
        self.store.sig_type_dedup(ResolvedType::Sig {
            caps: sig.provided_caps,
            base: ResolvedComponent {
                module:module_link,
                offset,
                applies:result,
            }
        })
    }

    fn project(&self, inner:URef<'static,ResolvedType>, nesting:u8) -> URef<'static,ResolvedType>{
        //Resolve the type
        let depth = match *inner {
            ResolvedType::Projection { depth, .. } => depth+nesting,
            _ => nesting
        };
        let inner = get_target(inner);
        self.store.projection_type_dedup(ResolvedType::Projection { depth, un_projected:inner})
    }

    fn resolve_projection_type(&self, inner:TypeRef) -> URef<'static,ResolvedType>{
        //get the inner type
        self.project(self.get_type(inner), 1)
    }

    fn resolve_function_callable(&self, module:ModRef, offset:u8,  applies:&[TypeRef]) -> URef<'static,ResolvedCallable>{
        //Fetch the link
        let module_link =  self.get_mod(module);
        // prepare the applies vector
        let result:Vec<URef<'static,ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect();

        self.store.dedup_callable(ResolvedCallable::Function { base:ResolvedComponent{
            offset,
            module: module_link, //Get the Module
            //Resolve the applied types
            applies:result
        }})
    }

    fn resolve_implement_callable(&self, module:ModRef, offset:u8,  applies:&[TypeRef]) -> URef<'static,ResolvedCallable>{
        //Fetch the link
        let module_link =  self.get_mod(module);
        // prepare the applies vector
        let result:Vec<URef<'static,ResolvedType>> = applies.iter().map(|appl|self.get_type(*appl)).collect();

        self.store.dedup_callable(ResolvedCallable::Implement { base:ResolvedComponent{
            offset,
            module: module_link, //Get the Module
            //Resolve the applied types
            applies:result
        }})
    }


    fn resolve_signature_from_component<C:CallableComponent>(&self, base:&ResolvedComponent) -> URef<'static,ResolvedSignature> {
        //Load the Comp
        let comp_cache = self.store.get_component::<C>(&base.module, base.offset);
        // get the Comp
        let comp = comp_cache.retrieve();
        //get its context with the applies as substitutions
        let context = comp_cache.substituted_context(&base.applies,&self.store);
        //Create the sig
        self.store.dedup_signature(ResolvedSignature {
            //Map the params to the resolved type
            params: comp.get_params().iter().map(|c|{
                let typ = context.get_type(c.typ);
                ResolvedParam{ typ, consumes: c.consumes }
            }).collect(),
            //Map the returns to the resolved type
            returns: comp.get_returns().iter().map(|rt|{
                context.get_type(*rt)
            }).collect(),
            transactional: comp.is_transactional()
        })
    }

    fn resolve_signature_from_callable(&self, fun:URef<'static,ResolvedCallable>) -> URef<'static,ResolvedSignature> {
        match *fun {
            ResolvedCallable::Function {  ref base, .. } => {
                //create the sig
                self.resolve_signature_from_component::<FunctionComponent>(base)
            },
            ResolvedCallable::Implement {  ref base, .. } => {
                //create the sig
                self.resolve_signature_from_component::<ImplementComponent>(base)
            }
        }
    }

    fn resolve_signature_from_type(&self, typ:URef<'static,ResolvedType>) -> URef<'static,ResolvedSignature> {
        match *typ {
            ResolvedType::Sig { ref base, .. }  => {
                //create the sig
                self.resolve_signature_from_component::<SigComponent>(base)
            },

            ResolvedType::Generic { .. }
            | ResolvedType::Projection { .. }
            | ResolvedType::Lit { .. }
            | ResolvedType::Data { .. }
            | ResolvedType::Virtual(_) => panic!("Only sig types and its projection have signatures")
        }
    }


    fn resolve_size_from_type(&self, typ:URef<'static,ResolvedType>) -> u16 {
        match *typ {
            ResolvedType::Lit { size, .. } => size,
            ResolvedType::Projection { un_projected, .. } => match *un_projected {
                ResolvedType::Lit { size, .. } => size,
                _ => panic!("Only literals and its projection have sizes")
            },
            _ => panic!("Only literals and its projection have sizes")
        }
    }

    fn resolve_ctr_from_type(&self, typ:URef<'static,ResolvedType>) -> URef<'static,ResolvedCtrs> {
        match *typ {
            ResolvedType::Data { ref base, .. }  => {
                //Load the Adt
                let adt_cache = self.store.get_component::<DataComponent>(&base.module, base.offset);
                // get the adt
                let adt = adt_cache.retrieve();
                //get its context with the applies as substitutions
                let context = adt_cache.substituted_context(&base.applies,&self.store);
                //Create the Ctr
                match adt.body {
                    DataImpl::Internal { ref constructors, .. } => {
                        self.store.dedup_ctr(constructors.iter().map(|c|{
                            c.fields.iter().map(|t|context.get_type(t.typ)).collect()
                        }).collect())
                    }
                    DataImpl::External(_) => panic!("Extrnal data does not have ctrs")
                }
            },
            ResolvedType::Projection { depth, un_projected, .. } => {
                assert!(if let ResolvedType::Projection{..} = *un_projected {false} else {true});
                //recurses only once (assert checks assumption)
                let ctrs = self.resolve_ctr_from_type(un_projected);
                self.store.dedup_ctr(ctrs.iter().map(|cases| cases.iter().map(|field|{
                    self.project(*field, depth)
                }).collect()).collect())
            },

            ResolvedType::Generic { .. }
            | ResolvedType::Lit { .. }
            | ResolvedType::Sig { .. }
            | ResolvedType::Virtual(_) => panic!("Only data and its projection have ctrs ")
        }
    }
}


pub fn apply_types(base_caps:CapSet, are_phantom:impl Iterator<Item=bool>, applies:&[URef<'static,ResolvedType>]) -> (CapSet,CapSet) {
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
