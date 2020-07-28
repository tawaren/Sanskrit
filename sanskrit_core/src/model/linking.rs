use model::*;
use model::resolved::*;
use sanskrit_common::store::Store;
use sanskrit_common::errors::*;
use loader::{FetchCache, Loader};
use utils::Crc;
use resolver::Context;
use sanskrit_common::model::ModuleLink;
use core::slice::from_ref;

//The ref trait allows to fetch the target it reference from a context
pub trait Ref<T, S:Store> {
    fn fetch(self, ctx:&Context<S>) -> Result<T>;      //Gets the target and throws if in existent
}

//ModRef is a Ref to a ModuleLink
impl<S:Store> Ref<Crc<ModuleLink>,S> for ModRef {
    fn fetch(self, ctx:&Context<S>) ->  Result<Crc<ModuleLink>> {
        ctx.get_mod(self)
    }
}

//TypeRef is a Ref to a Type
impl<S:Store> Ref<Crc<ResolvedType>,S> for TypeRef {
    fn fetch(self, ctx: &Context<S>) ->  Result<Crc<ResolvedType>> {
        ctx.get_type(self)
    }
}

//PermRef is a Ref to a Type
impl<S:Store> Ref<Crc<ResolvedPermission>,S> for PermRef {
    fn fetch(self, ctx: &Context<S>) ->  Result<Crc<ResolvedPermission>> {
        ctx.get_perm(self)
    }
}

//FuncRef is a Ref to a FunctionImport
impl<S:Store> Ref<Crc<ResolvedCallable>,S> for CallRef {
    fn fetch(self, ctx: &Context<S>) ->  Result<Crc<ResolvedCallable>> {
        ctx.get_callable(self)
    }
}

//A link is allows to fetch the corresponding target from Storage
pub trait Link<T> {
    // Gets the target
    fn resolve<'b, S:Store + 'b>(self, context:&Context<'b,S>) -> Result<T>;
    //checks if the target resides in the current module
    fn is_local_link(self) -> bool;
}

//A Module Link resolves to a Rc<Module>
impl Link<Crc<Module>> for ModuleLink {
    //Gets the Module Link
    fn resolve<'b, S:Store + 'b>(self, context:&Context<'b,S>) -> Result<Crc<Module>> {
        context.store.get_module(self.to_hash())
    }

    //Checks if this is the this module
    fn is_local_link(self) -> bool {
        match self {
            ModuleLink::Remote(_) => false,
            ModuleLink::This(_) => true,
        }
    }
}

//A FuncLink resolves to a FuncCache containing an FunctionComponent
impl Link<FetchCache<FunctionComponent>> for FuncLink {
    //Gets the Function Cache
    fn resolve<'b,  S:Store + 'b>(self, context:&Context<'b,S>) -> Result<FetchCache<FunctionComponent>> {
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
    fn resolve<'b, S:Store + 'b>(self, context:&Context<'b,S>) -> Result<FetchCache<DataComponent>> {
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
    fn get_local_limit<'a,S:Store+'a>(cache:&Loader<'a, S>) -> usize;
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

    fn get_local_limit<'a, S: Store + 'a>(cache: &Loader<S>) -> usize {
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

    fn get_local_limit<'a, S: Store + 'a>(cache: &Loader<S>) -> usize {
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

    fn get_local_limit<'a, S: Store + 'a>(cache: &Loader<S>) -> usize {
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

    fn get_local_limit<'a, S: Store + 'a>(cache: &Loader<S>) -> usize {
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
                PermissionImport::Type(_, ref typRef) => from_ref(typRef),
            },
        }
    }

    fn is_transactional(&self) -> bool { false }
}
