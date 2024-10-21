use alloc::boxed::Box;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::Deref;
use crate::{Seekable, Select, IdSelect};

const BASE_RESERVE:usize = 128;

#[derive(Debug)]
pub struct URef<'a, T>(*const T, PhantomData<&'a T>);

impl<'a, T> URef<'a, T> {
    pub(crate) unsafe fn new(r:*const T) -> Self {
        URef(r,PhantomData)
    }
}

impl<'a, T> Copy for URef<'a,T> {}
impl<'a, T> Clone for URef<'a, T> {
    fn clone(&self) -> Self {
        URef(self.0, self.1)
    }
}



//Checks if two pointers point to the same memory address
#[inline]
fn same_ref_internal<T>(a: *const T, b: *const T) -> bool {
    a == b
}

//Compares two pointers by their memory address
#[inline]
fn compare_ref_internal<T>(a: *const T, b: *const T) -> Ordering {
    a.cmp(&b)
}

#[inline]
fn hash_ref<H: Hasher,T>(a: *const T, state: &mut H) {
    state.write_usize(a as usize);
}

//Allows to call functions on URef's by dereferencing to their element
impl<'a, T> Deref for URef<'a, T> {
    type Target = T;
    fn deref(&self) -> &'a T {
        unsafe{&*self.0}
    }
}

//Allows to borrow out of a URef
impl<'a, T: ?Sized> Borrow<T> for URef<'a, T> where T:Sized {
    fn borrow(&self) -> &'a T {
        unsafe{&*self.0}
    }
}

//Allows to compare two URef's (uses ptr equality as a shortcut)
impl<'a, T> Eq for URef<'a, T> {}
impl<'a, T> PartialEq for URef<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        //if same pointer look no further
        return same_ref_internal::<T>(self.0, other.0)
    }
}

impl<'a, T> Ord for URef<'a, T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        //if same pointer look no further
        return compare_ref_internal::<T>(self.0, other.0)
    }
}
impl<'a, T> PartialOrd for URef<'a, T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T> Hash for URef<'a, T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_ref(self.0, state)
    }
}

//Just to share some core functionality
pub struct UniqueBaseArena<T>{store:RefCell<Vec<Box<T>>>}

impl<T> UniqueBaseArena<T> {
    fn new() -> Self {
        UniqueBaseArena{store:RefCell::new(Vec::with_capacity(BASE_RESERVE))}
    }

    //Will not init capacity, calling enforce capacity may help
    const fn new_const() -> Self {
        UniqueBaseArena{store:RefCell::new(Vec::new())}
    }

    unsafe fn unvalidated_leak(self) {
        let UniqueBaseArena{store} = self;
        store.take().leak();
    }

    #[inline]
    unsafe fn get_ptr<V,S:Select<T,V>>(&self, index:usize) -> *const V {
        S::select(&*self.store.borrow()[index]) as *const V
    }

    #[inline]
    fn get<V,S:Select<T,V>>(&self, index:usize) -> &V{
        unsafe {&*self.get_ptr::<V,S>(index)}
    }

    #[inline]
    unsafe fn get_unique<V,S:Select<T,V>>(&self, index:usize) -> URef<V>{
        URef::new(self.get_ptr::<V,S>(index))
    }

    fn append(&self, elem:T) {
        self.store.borrow_mut().push(Box::new(elem))
    }

    pub fn reserve_capacity(&self, cap:usize){
        self.store.borrow_mut().reserve(cap);
    }

    pub fn reserve_base_capacity(&self){
        self.reserve_capacity(BASE_RESERVE)
    }

    pub fn len(&self) -> usize {
        self.store.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.borrow().is_empty()
    }
}

impl<T> Seekable<T> for UniqueBaseArena<T>  {
    type I = Box<T>;
    #[inline]
    fn deref(inner: &Self::I) -> &T { &*inner }
    #[inline]
    fn with_store<F: FnOnce(&[Box<T>]) -> ()>(&self, f: F) {
        let store = self.store.borrow();
        f(&store[..])
    }
}

pub struct IdentityArena<T>(UniqueBaseArena<T>);

impl<T> IdentityArena<T> {
    pub fn new() -> Self {
        IdentityArena(UniqueBaseArena::new())
    }

    pub const fn new_const() -> Self {
        IdentityArena(UniqueBaseArena::new_const())
    }

    pub fn alloc(&self, elem:T) -> URef<T> {
        let index = self.len();
        self.append(elem);
        unsafe {self.get_unique::<T,IdSelect>(index)}
    }
}

impl<T> Deref for IdentityArena<T> {
    type Target = UniqueBaseArena<T> ;
    fn deref(&self) -> &Self::Target { &self.0 }
}

pub struct WeakUniqueArena<T>(UniqueBaseArena<T>);

impl<T:Eq> WeakUniqueArena<T> {

    pub fn new() -> Self {
        WeakUniqueArena(UniqueBaseArena::new())
    }

    pub const fn new_const() -> Self {
        WeakUniqueArena(UniqueBaseArena::new_const())
    }

    pub fn alloc_unique(&self, elem:T) -> URef<T> {
        let index = self.unconstrained_seek::<T,IdSelect>(&elem);
        if index < self.len() {
            assert!(*self.get::<T,IdSelect>(index) == elem);
        } else {
            assert!(index == self.len());
            //if self.store.capacity() == 0 {self.store.reserve(IC)}
            self.append(elem);
        }
        unsafe {self.get_unique::<T,IdSelect>(index)}
    }

    pub unsafe fn unvalidated_leak(self) {
        let WeakUniqueArena(store) = self;
        store.unvalidated_leak();
    }
}

impl<T:Eq> Deref for WeakUniqueArena<T> {
    type Target = UniqueBaseArena<T> ;
    fn deref(&self) -> &Self::Target { &self.0 }
}

pub struct UniqueArena<T>(WeakUniqueArena<T>) where T:Eq+Ord;

impl<T:Eq+Ord> Deref for UniqueArena<T> {
    type Target = WeakUniqueArena<T> ;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T:Eq+Ord> UniqueArena<T> {
    /// **Warning**: If indirectly owned by a static value make sure to call `validate()` manually before the program exits.
    ///              If the collection is dropped regularly then 'drop()' will call `validate()`
    ///              An attacker can craft hints that make the collection behave wrongly
    ///              Only by calling `validate()` are those attempts detected
    pub fn new() -> Self {
        UniqueArena(WeakUniqueArena::new())
    }
    /// Use this constructor if you need a static instance.
    /// **Warning**: Make sure to call `validate()` manually before the program exits.
    ///              An attacker can craft hints that make the collection behave wrongly
    ///              Only by calling `validate()` are those attempts detected
    pub const fn new_unvalidated() -> Self {
        UniqueArena(WeakUniqueArena::new_const())
    }

    /// Panics if wrong hints were supplied
    /// Takes O((n+d)*k) time where
    ///     n = number of entries
    ///     d = number of deleted entries
    ///     k = size of the keys
    pub fn validate(&self) { self.validate_unique_entries::<T,IdSelect>() }
}

impl<T:Eq+Ord> Drop for UniqueArena<T>{
    fn drop(&mut self) {
        self.validate()
    }
}

pub trait Embedding<T> {
    type Auxiliary;
    fn embed(self, extra:&Self::Auxiliary) -> T;
    fn extract(embedded:&T) -> &Self;
}

struct EmbedSelect;

impl<T,E:Embedding<T>> Select<T,E> for EmbedSelect {
    fn select(main: &T) -> &E {
        E::extract(main)
    }
}

pub struct UniqueEmbeddableArena<E:Embedding<T>, T>(UniqueBaseArena<T>, PhantomData<E>) where E:Eq+Ord;

impl<T,E:Embedding<T>+Eq+Ord> UniqueEmbeddableArena<E,T> {

    pub fn new() -> Self {
        UniqueEmbeddableArena(UniqueBaseArena::new(), PhantomData)
    }

    pub const fn new_unvalidated() -> Self {
        UniqueEmbeddableArena(UniqueBaseArena::new_const(), PhantomData)
    }

    pub fn alloc_unique(&self, param:E, extra:&E::Auxiliary) -> URef<T> {
        let index = self.unconstrained_seek::<E,EmbedSelect>(&param);
        if index < self.len() {
            assert!(*self.get::<E,EmbedSelect>(index) == param);
        } else {
            assert!(index == self.len());
            //if self.store.capacity() == 0 {self.store.reserve(IC)}
            self.append(param.embed(extra));
        }
        unsafe {self.get_unique::<T,IdSelect>(index)}
    }

    /// Panics if wrong hints were supplied
    /// Takes O((n+d)*k) time where
    ///     n = number of entries
    ///     d = number of deleted entries
    ///     k = size of the keys
    pub fn validate(&self) { self.validate_unique_entries::<E,EmbedSelect>() }
}

impl<T,E:Embedding<T>+Eq+Ord> Deref for UniqueEmbeddableArena<E,T> {
    type Target = UniqueBaseArena<T> ;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T,E:Embedding<T>+Eq+Ord> Drop for UniqueEmbeddableArena<E,T>{
    fn drop(&mut self) {
        self.validate()
    }
}

//Todo: PartialMatchArena
//      Matches on subset of value
//      Can be used to build memoisation where te return captures all the parameters
//       Benefit vs classical memoisation is that the parameters do not have to be cloned
//      Can be used to build classical memoisation by using a Memo<P,R>(P,R) & Param<P>(P) with:
//      //used during get_ptr
//      impl<P,R> Select<Memo<P,R>> for R {
//          #[inline]
//          fn select(main:&Memo<P,R>) -> &Self {
//              main.1
//          }
//      }
//      //Used during seek & validate
//      impl<P,R> Select<Memo<P,R>> for P {
//          #[inline]
//          fn select(main:&Memo<P,R>) -> &Self {
//              main.0
//          }
//      }
