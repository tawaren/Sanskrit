use alloc::borrow::ToOwned;
use core::cell::RefCell;
use core::cmp::Ordering;
use core::fmt::Debug;
use core::hash::{Hash, Hasher};
use crate::model::resolved::*;
use crate::loader::{StateManager, Loader};
use crate::resolver::Context;
use sanskrit_common::model::ModuleLink;
use core::slice::from_ref;
use crate::model::{BodyImport, CallableImpl, CallRef, DataComponent, FunctionComponent, Generic, ImplementComponent, ModRef, Module, Param, PermissionImport, PermRef, PublicImport, SigComponent, TypeRef};
#[cfg(feature = "multi-thread")]
use spin::Mutex;
use sp1_zkvm_col::arena::URef;

//The ref trait allows to fetch the target it reference from a context
pub trait Ref<T, S:StateManager> {
    fn fetch(self, ctx:&Context<S>) -> T;      //Gets the target and throws if in existent
}

//ModRef is a Ref to a ModuleLink
impl<S:StateManager> Ref<FastModuleLink,S> for ModRef {
    fn fetch(self, ctx:&Context<S>) ->  FastModuleLink {
        ctx.get_mod(self)
    }
}

//TypeRef is a Ref to a Type
impl<S:StateManager> Ref<URef<'static,ResolvedType>,S> for TypeRef {
    fn fetch(self, ctx: &Context<S>) ->  URef<'static,ResolvedType> {
        ctx.get_type(self)
    }
}

//PermRef is a Ref to a Type
impl<S:StateManager> Ref<URef<'static,ResolvedPermission>,S> for PermRef {
    fn fetch(self, ctx: &Context<S>) -> URef<'static,ResolvedPermission> {
        ctx.get_perm(self)
    }
}

//FuncRef is a Ref to a FunctionImport
impl<S:StateManager> Ref<URef<'static,ResolvedCallable>,S> for CallRef {
    fn fetch(self, ctx: &Context<S>) -> URef<'static,ResolvedCallable> {
        ctx.get_callable(self)
    }
}

pub trait Component  {
    fn get(module:&Module, offset:u8) -> &Self;
    fn num_elems(module:&Module) -> usize;
    fn get_local_limit<S:StateManager>(cache:&Loader<S>) -> usize;
    fn get_signature_byte_size(&self) -> usize;
    fn get_full_byte_size(&self) -> usize;
    fn get_public_import(&self) -> &PublicImport;
    fn get_body_import(&self) -> Option<&BodyImport>;
    fn get_generics(&self) -> &[Generic];
}

pub trait CallableComponent:Component {
    fn get_params(&self) -> &[Param];
    fn get_returns(&self) -> &[TypeRef];
    fn is_transactional(&self) -> bool;
}

impl Component for DataComponent {
    fn get(module: &Module, offset: u8) -> &Self {
        &module.data[offset as usize]
    }

    fn num_elems(module: &Module) -> usize {
        module.data.len()
    }

    fn get_local_limit<S:StateManager>(cache: &Loader<S>) -> usize {
        cache.this_deployed_data.get()
    }

    fn get_signature_byte_size(&self) -> usize {
        self.byte_size.unwrap()
    }

    fn get_full_byte_size(&self) -> usize {
        self.get_signature_byte_size()
    }

    fn get_public_import(&self) -> &PublicImport {
        &self.import
    }

    fn get_body_import(&self) -> Option<&BodyImport> {
        None
    }

    fn get_generics(&self) -> &[Generic] {
        &self.generics
    }
}

impl Component for SigComponent {
    fn get(module: &Module, offset: u8) -> &Self {
        &module.sigs[offset as usize]
    }

    fn num_elems(module: &Module) -> usize {
        module.sigs.len()
    }

    fn get_local_limit<S:StateManager>(cache: &Loader<S>) -> usize {
        cache.this_deployed_sigs.get()
    }

    fn get_signature_byte_size(&self) -> usize {
        self.byte_size.unwrap()
    }

    fn get_full_byte_size(&self) -> usize {
        self.get_signature_byte_size()
    }

    fn get_public_import(&self) -> &PublicImport {
        &self.shared.import
    }

    fn get_body_import(&self) -> Option<&BodyImport> {
        None
    }

    fn get_generics(&self) -> &[Generic] {
        &self.shared.generics
    }
}

impl CallableComponent for SigComponent {
    fn get_params(&self) -> &[Param] {
        &self.shared.params
    }

    fn get_returns(&self) -> &[TypeRef] {
        &self.shared.returns
    }

    fn is_transactional(&self) -> bool {
        self.shared.transactional
    }
}

impl Component for FunctionComponent {
    fn get(module: &Module, offset: u8) -> &Self {
        &module.functions[offset as usize]
    }

    fn num_elems(module: &Module) -> usize {
        module.functions.len()
    }

    fn get_local_limit<S:StateManager>(cache: &Loader<S>) -> usize {
        cache.this_deployed_functions.get()
    }

    fn get_signature_byte_size(&self) -> usize {
        let size = self.byte_size.unwrap();
        match self.body {
            CallableImpl::Internal { byte_size:Some(body_size), .. } => size - body_size,
            _ => size
        }
    }

    fn get_full_byte_size(&self) -> usize {
        self.byte_size.unwrap()
    }

    fn get_public_import(&self) -> &PublicImport {
        &self.shared.import
    }

    fn get_body_import(&self) -> Option<&BodyImport> {
        match self.body {
            CallableImpl::External => None,
            CallableImpl::Internal {ref imports, .. } => Some(imports),
        }
    }

    fn get_generics(&self) -> &[Generic] {
        &self.shared.generics
    }
}

impl CallableComponent for FunctionComponent {
    fn get_params(&self) -> &[Param] {
        &self.shared.params
    }

    fn get_returns(&self) -> &[TypeRef] {
        &self.shared.returns
    }

    fn is_transactional(&self) -> bool {
        self.shared.transactional
    }
}

impl Component for ImplementComponent {
    fn get(module: &Module, offset: u8) -> &Self {
        &module.implements[offset as usize]
    }

    fn num_elems(module: &Module) -> usize {
        module.implements.len()
    }

    fn get_local_limit<S:StateManager>(cache: &Loader<S>) -> usize {
        cache.this_deployed_implements.get()
    }

    fn get_signature_byte_size(&self) -> usize {
        let size = self.byte_size.unwrap();
        match self.body {
            CallableImpl::Internal { byte_size:Some(body_size), .. } => size - body_size,
            _ => size
        }
    }

    fn get_full_byte_size(&self) -> usize {
        self.byte_size.unwrap()
    }

    fn get_public_import(&self) -> &PublicImport {
        &self.import
    }

    fn get_body_import(&self) -> Option<&BodyImport> {
        match self.body {
            CallableImpl::External => None,
            CallableImpl::Internal {ref imports, .. } => Some(imports),
        }
    }

    fn get_generics(&self) -> &[Generic] {
        &self.generics
    }
}

impl CallableComponent for ImplementComponent {
    fn get_params(&self) -> &[Param] {
        &self.params
    }

    fn get_returns(&self) -> &[TypeRef] {
        //sadly we do not have direct access to the return type without duplicating it in the input
        match &self.body {
            //would fail in fetching the permission when validating
            CallableImpl::External => unreachable!(),
            CallableImpl::Internal { ref imports, .. } => match imports.permissions[self.sig.0 as usize] {
                //Would fail on get type in validate
                PermissionImport::Callable(_, _) => unreachable!(),
                PermissionImport::Type(_, ref typ_ref) => from_ref(typ_ref),
            },
        }
    }

    fn is_transactional(&self) -> bool { false }
}


//A module link that allows for inline caching
//  Note: We do not due this for the other links as they use this
//        After they have the module, then its only a vector lookup away
#[derive(Debug)]
pub struct FastModuleLink(URef<'static,ModuleLink>, URef<'static,RefCell<Option<URef<'static,Module>>>>);

impl FastModuleLink {
    pub unsafe fn identity_leak(link:URef<'static,ModuleLink>, cache:Option<URef<'static,Module>>) -> Self{
        FastModuleLink(link,URef::identity_leak(RefCell::new(cache)))
    }

    pub fn load<S:StateManager>(&self, store:&Loader<S>) -> URef<'static,Module> {
        if self.1.borrow().is_some() {
            self.1.borrow().to_owned().unwrap()
        } else {
            let module = store.get_module(self.0);
            let _ = self.1.borrow_mut().insert(module);
            module
        }
    }

    pub fn resolve<'b, S:StateManager>(&self, context:&Context<'b,S>) -> URef<'static,Module> {
        self.load(&context.store)
    }

    pub fn get_module_link(&self) -> &ModuleLink{
        &self.0
    }

    pub fn get_cache(&self) -> URef<'static, RefCell<Option<URef<'static, Module>>>>{
        self.1
    }
}

impl Clone for FastModuleLink {
    fn clone(&self) -> Self {
        FastModuleLink(self.0.clone(),self.1.clone())
    }
}

impl Copy for FastModuleLink {}

impl PartialEq for FastModuleLink {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for FastModuleLink { }

impl PartialOrd for FastModuleLink {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for FastModuleLink {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for FastModuleLink {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

