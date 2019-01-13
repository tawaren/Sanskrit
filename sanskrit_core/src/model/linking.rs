use model::*;
use model::resolved::*;
use sanskrit_common::store::Store;
use sanskrit_common::errors::*;
use alloc::rc::Rc;
use loader::AdtCache;
use loader::FuncCache;
use utils::Crc;
use resolver::Context;
use sanskrit_common::model::ModuleLink;

//Cache keys are used to provide an highly efficient O(1) cache when the amount of slots is known
pub trait CacheKey {
    fn as_key(self) -> usize;
}

//u8 can be used as cache key
impl CacheKey for u8 {
    fn as_key(self) -> usize {
        self as usize
    }
}

//ModRef can be used as cache key
impl CacheKey for ModRef {
    fn as_key(self) -> usize {
        self.0 as usize
    }
}

//TypeRef can be used as cache key
impl CacheKey for TypeRef {
    fn as_key(self) -> usize {
        self.0 as usize
    }
}

//GenRef can be used as cache key
impl CacheKey for GenRef {
    fn as_key(self) -> usize {
        self.0 as usize
    }
}

//FuncRef can be used as cache key
impl CacheKey for FuncRef {
    fn as_key(self) -> usize {
        self.0 as usize
    }
}

//ErrorRef can be used as cache key
impl CacheKey for ErrorRef {
    fn as_key(self) -> usize {
        self.0 as usize
    }
}

//The ref trait allows to fetch the target it reference from a context
pub trait Ref<T, S:Store> {
    fn fetch(self, ctx:&Context<S>) -> Result<T>;      //Gets the target and throws if in existent
}

//ModRef is a Ref to a ModuleLink
impl<S:Store> Ref<Rc<ModuleLink>,S> for ModRef {
    fn fetch(self, ctx:&Context<S>) ->  Result<Rc<ModuleLink>> {
        ctx.get_mod(self)
    }
}

//TypeRef is a Ref to a Type
impl<S:Store> Ref<Crc<ResolvedType>,S> for TypeRef {
    fn fetch(self, ctx: &Context<S>) ->  Result<Crc<ResolvedType>> {
        ctx.get_type(self)
    }
}

//FuncRef is a Ref to a FunctionImport
impl<S:Store> Ref<Rc<ResolvedFunction>,S> for FuncRef {
    fn fetch(self, ctx: &Context<S>) ->  Result<Rc<ResolvedFunction>> {
        ctx.get_func(self)
    }
}

//ErrorRef is a Ref to a ErrorImport
impl<S:Store> Ref<Rc<ResolvedErr>,S> for ErrorRef {
    fn fetch(self, ctx: &Context<S>) ->  Result<Rc<ResolvedErr>> {
        ctx.get_err(self)
    }
}

//A link is allows to fetch the corresponding target from Storage
pub trait Link<T> {
    // Gets the target
    fn resolve<'a, 'b:'a, S:Store + 'b>(self, context:&Context<'a,'b,S>) -> Result<T>;
    //checks if the target resides in the current module
    fn is_local_link(self) -> bool;
}

//A Module Link resolves to a Rc<Module>
impl Link<Rc<Module>> for ModuleLink {
    //Gets the Module Link
    fn resolve<'a, 'b:'a, S:Store + 'b>(self, context:&Context<'a,'b,S>) -> Result<Rc<Module>> {
        context.store.get_module(&self.to_hash())
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
impl Link<FuncCache> for FuncLink {
    //Gets the Function Cache
    fn resolve<'a, 'b:'a, S:Store + 'b>(self, context:&Context<'a,'b,S>) -> Result<FuncCache> {
        let mod_link =  self.module.fetch(&context)?;
        context.store.get_func(&mod_link,self.offset)

    }

    //Checks if the function is from the this module (Mod0)
    fn is_local_link(self) -> bool {
        self.module.0 == 0
    }
}



//A AdtLink resolves to a AdtCache containing an AdtComponent
impl Link<AdtCache> for AdtLink {
    //Gets the Adt Cache
    fn resolve<'a, 'b:'a, S:Store + 'b>(self, context:&Context<'a,'b,S>) -> Result<AdtCache> {
        let mod_link = self.module.fetch(&context)?;
        context.store.get_adt(&mod_link,self.offset)
    }

    //Checks if the adt is from the this module (Mod0)
    fn is_local_link(self) -> bool {
        self.module.0 == 0
    }
}