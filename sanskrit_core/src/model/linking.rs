use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use crate::model::resolved::*;
use sanskrit_common::errors::*;
use sanskrit_common::model::Hash as CHash;

use sanskrit_common::encoding::*;
use crate::loader::{FetchCache, Loader};
use sanskrit_common::utils::Crc;
use crate::resolver::Context;
use sanskrit_common::model::ModuleLink;
use core::slice::from_ref;
use sanskrit_common::supplier::{store_hash, Supplier};
use crate::model::{BodyImport, CallableImpl, CallRef, DataComponent, DataLink, FuncLink, FunctionComponent, Generic, ImplementComponent, ModRef, Module, Param, PermissionImport, PermRef, PublicImport, SigComponent, TypeRef};
#[cfg(feature = "multi-thread")]
use spin::Mutex;



//The ref trait allows to fetch the target it reference from a context
pub trait Ref<T, S:Supplier<Module>> {
    fn fetch(self, ctx:&Context<S>) -> Result<T>;      //Gets the target and throws if in existent
}

//ModRef is a Ref to a ModuleLink
impl<S:Supplier<Module>> Ref<FastModuleLink,S> for ModRef {
    fn fetch(self, ctx:&Context<S>) ->  Result<FastModuleLink> {
        ctx.get_mod(self)
    }
}

//TypeRef is a Ref to a Type
impl<S:Supplier<Module>> Ref<Crc<ResolvedType>,S> for TypeRef {
    fn fetch(self, ctx: &Context<S>) ->  Result<Crc<ResolvedType>> {
        ctx.get_type(self)
    }
}

//PermRef is a Ref to a Type
impl<S:Supplier<Module>> Ref<Crc<ResolvedPermission>,S> for PermRef {
    fn fetch(self, ctx: &Context<S>) ->  Result<Crc<ResolvedPermission>> {
        ctx.get_perm(self)
    }
}

//FuncRef is a Ref to a FunctionImport
impl<S:Supplier<Module>> Ref<Crc<ResolvedCallable>,S> for CallRef {
    fn fetch(self, ctx: &Context<S>) ->  Result<Crc<ResolvedCallable>> {
        ctx.get_callable(self)
    }
}

//A link is allows to fetch the corresponding target from Storage
pub trait Link<T> {
    // Gets the target
    fn resolve<'b, S:Supplier<Module> + 'b>(self, context:&Context<'b,S>) -> Result<T>;
    //checks if the target resides in the current module
    fn is_local_link(self) -> bool;
}

//A FuncLink resolves to a FuncCache containing an FunctionComponent
impl Link<FetchCache<FunctionComponent>> for FuncLink {
    //Gets the Function Cache
    fn resolve<'b,  S:Supplier<Module> + 'b>(self, context:&Context<'b,S>) -> Result<FetchCache<FunctionComponent>> {
        let mod_link =  self.module.fetch(&context)?;
        context.store.get_component(&mod_link,self.offset)

    }

    //Checks if the function is from the this module (Mod0)
    fn is_local_link(self) -> bool {
        self.module.0 == 0
    }
}

//A AdtLink resolves to a AdtCache containing an AdtComponent
impl Link<FetchCache<DataComponent>> for DataLink {
    //Gets the Adt Cache
    fn resolve<'b, S:Supplier<Module> + 'b>(self, context:&Context<'b,S>) -> Result<FetchCache<DataComponent>> {
        let mod_link = self.module.fetch(&context)?;
        context.store.get_component(&mod_link, self.offset)
    }

    //Checks if the adt is from the this module (Mod0)
    fn is_local_link(self) -> bool {
        self.module.0 == 0
    }
}

pub trait Component {
    fn get(module:&Module, offset:u8) -> &Self;
    fn num_elems(module:&Module) -> usize;
    fn get_local_limit<'a,S:Supplier<Module>+'a>(cache:&Loader<'a, S>) -> usize;
    fn get_signature_byte_size(&self) -> Result<usize>;
    fn get_full_byte_size(&self) -> Result<usize>;
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

    fn get_local_limit<'a, S: Supplier<Module> + 'a>(cache: &Loader<S>) -> usize {
        cache.this_deployed_data.get()
    }

    fn get_signature_byte_size(&self) -> Result<usize> {
        match self.byte_size {
            None => error(||"Byte size is missing"),
            Some(size) => Ok(size),
        }
    }

    fn get_full_byte_size(&self) -> Result<usize> {
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

    fn get_local_limit<'a, S: Supplier<Module> + 'a>(cache: &Loader<S>) -> usize {
        cache.this_deployed_sigs.get()
    }

    fn get_signature_byte_size(&self) -> Result<usize> {
        match self.byte_size {
            None => error(|| "Byte size is missing"),
            Some(size) => Ok(size),
        }
    }

    fn get_full_byte_size(&self) -> Result<usize> {
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

    fn get_local_limit<'a, S: Supplier<Module> + 'a>(cache: &Loader<S>) -> usize {
        cache.this_deployed_functions.get()
    }

    fn get_signature_byte_size(&self) -> Result<usize> {
        match self.byte_size {
            None => error(||"Byte size is missing"),
            Some(size) => match self.body {
                CallableImpl::Internal { byte_size:Some(body_size), .. } => Ok(size - body_size),
                _ => Ok(size)
            },
        }
    }

    fn get_full_byte_size(&self) -> Result<usize> {
        match self.byte_size {
            None => error(||"Byte size is missing"),
            Some(size) => Ok(size),
        }
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

    fn get_local_limit<'a, S: Supplier<Module> + 'a>(cache: &Loader<S>) -> usize {
        cache.this_deployed_implements.get()
    }

    fn get_signature_byte_size(&self) -> Result<usize> {
        match self.byte_size {
            None => error(||"Byte size is missing"),
            Some(size) => match self.body {
                CallableImpl::Internal { byte_size:Some(body_size), .. } => Ok(size - body_size),
                _ => Ok(size)
            },
        }
    }

    fn get_full_byte_size(&self) -> Result<usize> {
        match self.byte_size {
            None => error(||"Byte size is missing"),
            Some(size) => Ok(size),
        }
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
pub struct FastModuleLink(Crc<ModuleLink>, Rc<RefCell<Option<Crc<Module>>>>);

impl FastModuleLink {
    pub fn new(link:Crc<ModuleLink>, cache:Option<Crc<Module>>) -> Self{
        FastModuleLink(link,Rc::new(RefCell::new(cache)))
    }

    fn new_shared(link:Crc<ModuleLink>, cache:Rc<RefCell<Option<Crc<Module>>>>) -> Self{
        FastModuleLink(link,cache)
    }

    pub fn load<'b, S:Supplier<Module> + 'b>(&self, store:&Loader<'b,S>) -> Result<Crc<Module>> {
        if self.1.borrow().is_some() {
            Ok(self.1.borrow().to_owned().unwrap())
        } else {
            let module = store.get_module(self.0.to_hash())?;
            let _ = self.1.borrow_mut().insert(module.clone());
            Ok(module)
        }
    }
}

//A Module Link resolves to a Rc<Module>
impl Link<Crc<Module>> for &FastModuleLink {
    //Gets the Module Link
    fn resolve<'b, S:Supplier<Module> + 'b>(self, context:&Context<'b,S>) -> Result<Crc<Module>> {
        self.load(&context.store)
    }

    //Checks if this is the module
    fn is_local_link(self) -> bool {
        match *self.0 {
            ModuleLink::Remote(_) => false,
            ModuleLink::This(_) => true,
        }
    }
}

impl Clone for FastModuleLink {
    fn clone(&self) -> Self {
        FastModuleLink(self.0.clone(),self.1.clone())
    }
}

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

#[cfg(not(feature = "multi-thread"))]
static mut EAGER_FAST_LINKS: BTreeMap<CHash, FastModuleLink> = BTreeMap::new();
#[cfg(not(feature = "multi-thread"))]
pub fn dedup_link(link:CHash) -> FastModuleLink {
    unsafe {EAGER_FAST_LINKS.entry(link)}.or_insert_with_key(|link|{
        let module_link = Crc{elem:Rc::new(ModuleLink::Remote(link.clone()))};
        FastModuleLink::new(module_link,None)
    }).to_owned()
}

#[cfg(feature = "multi-thread")]
static EAGER_FAST_LINKS: Mutex<BTreeMap<CHash, FastModuleLink>> = Mutex::new(BTreeMap::new());
#[cfg(feature = "multi-thread")]
pub fn dedup_link(link:CHash) -> FastModuleLink {
    let mut map = EAGER_FAST_LINKS.lock();
    map.entry(link).or_insert_with_key(|link|{
        let module_link = Crc{elem:Rc::new(ModuleLink::Remote(link.clone()))};
        FastModuleLink::new(module_link,None)
    }).to_owned()
}

impl Parsable for FastModuleLink {
    fn parse(p: &mut Parser) -> Result<Self> {
        Ok(dedup_link(CHash::parse(p)?))
    }
}

impl Serializable for FastModuleLink {
    fn serialize(&self, s: &mut Serializer) -> Result<()> {
        self.0.serialize(s)
    }
}

pub struct FastCacheSupplier;

impl FastCacheSupplier {
    pub fn new() -> Self { FastCacheSupplier }
    pub fn add(&mut self, data:&[u8], as_local:bool) -> Result<FastModuleLink> {
        let key = store_hash(&[data]);
        let fast_link = dedup_link(key);
        if fast_link.1.borrow().is_none() {
            let module: Module = Parser::parse_fully(data)?;
            let _ = fast_link.1.borrow_mut().insert(Crc{elem:Rc::new(module)});
        }
        if as_local {
            let local_link = Crc{elem:Rc::new(ModuleLink::This(fast_link.0.to_hash()))};
            Ok(FastModuleLink::new_shared(local_link,fast_link.1))
        } else {
            Ok(fast_link)
        }
    }
}

impl Supplier<Module> for FastCacheSupplier {
    fn unique_get(&self, key: &CHash) -> Result<Crc<Module>> {
        //println!("cycle-tracker-report-start: fetch module");
        let fast_link = dedup_link(key.clone());
        let res = if fast_link.1.borrow().is_some() {
            Ok(fast_link.1.borrow().to_owned().unwrap())
        } else {
            error(||"Required module is missing")
        };
        //println!("cycle-tracker-report-end: fetch module");
        return res;
    }
}