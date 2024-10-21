use alloc::boxed::Box;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::NonNull;
use crate::arena::URef;

impl<T> URef<'static, T> {
    pub unsafe fn identity_leak(t:T) -> Self {
        URef::new(Box::leak(Box::new(t)) as *const T)
    }
}

//Alt to Rc as we are in a No Dealloc Env
#[derive(PartialEq, Debug, Eq)]
pub struct SRef<T> {
    ptr: NonNull<T>,
    phantom:PhantomData<T>
}

impl<T> Copy for SRef<T>{}

impl<T> Clone for SRef<T> {
    fn clone(&self) -> Self {
        SRef{ptr:self.ptr, phantom:PhantomData}
    }
}

impl<T> SRef<T> {
    pub fn new(value: T) -> Self {
        SRef {
            ptr: unsafe {NonNull::new_unchecked(Box::leak(Box::new(value)))},
            phantom: PhantomData
        }
    }
}

impl<T> Deref for SRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe{self.ptr.as_ref()}
    }
}