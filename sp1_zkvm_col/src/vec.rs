use alloc::vec::Vec;
use crate::{IdSelect, Seekable, SeekMode};

pub struct UniqueVecBuilder<T>(Vec<T>) where T:Eq+Ord;

impl<T:Eq+Ord> UniqueVecBuilder<T> {
    pub const fn new() -> Self {
        UniqueVecBuilder(Vec::new())
    }

    pub fn with_capacity(cap:usize) -> Self {
        UniqueVecBuilder(Vec::with_capacity(cap))
    }

    pub fn add<M:SeekMode<T>>(&mut self, elem:T){
        let index = M::seek::<T,IdSelect>(self,&elem);
        if index < self.0.len() {
            assert!(self.0[index] == elem);
        } else {
            assert!(index == self.0.len());
            //if self.store.capacity() == 0 {self.store.reserve(IC)}
            self.0.push(elem);
        }
    }

    pub fn finalize(self) -> Vec<T> {
        self.validate_unique_entries::<T,IdSelect>();
        self.0
    }

    //Todo: have a sorted finalize
    //      uses a version of validate_unique_entries which returns a swap vector
    //      same as permutation but takes swapping into account.
    //      for i = 0..len {mem::swap(vec[i], vec[swap[i]])} (+ the less than check)

}

impl<T:Eq+Ord> Seekable<T> for UniqueVecBuilder<T> {
    type I = T;
    fn deref(inner: &Self::I) -> &T { inner }
    fn with_store<R, F: FnOnce(&[Self::I]) -> R>(&self, f: F) -> R {
        f(&self.0)
    }
}

