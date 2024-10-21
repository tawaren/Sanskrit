use alloc::borrow::ToOwned;
use alloc::rc::Rc;
use core::ops::Deref;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::clone::Clone;
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use sp1_zkvm_col::map::HintMap;

//A complexity counting Rc to prevent complexity based attacks
//it tracks the Elems and depth and produces an error if the Limits are reached
#[derive(Debug)]
pub struct Crc<T>{
    pub elem:Rc<T>  //the actual Element
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

//Allows to call functions on Crc's by dereferencing to their element
impl<T> Deref for Crc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.elem.deref()
    }
}

//Allows to borrow out of a Crc
impl<T: ?Sized> Borrow<T> for Crc<T> where T:Sized {
    fn borrow(&self) -> &T {
        self.elem.borrow()
    }
}

//Allow to clone a Crc (clones only the rc pointer not the value)
impl<T> Clone for Crc<T> {
    fn clone(&self) -> Self {
        Crc{ elem: self.elem.clone() }
    }
}

//Allows to compare two Crc's (uses ptr equality as a shortcut)
impl<T> Eq for Crc<T> {}
impl<T> PartialEq for Crc<T> {
    fn eq(&self, other: &Self) -> bool {
        //if same pointer look no further
        return same_ref_internal::<T>(self.elem.as_ref(), other.elem.as_ref())
    }
}

impl<T> Ord for Crc<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        //if same pointer look no further
        return compare_ref_internal::<T>(self.elem.as_ref(), other.elem.as_ref())
    }
}
impl<T> PartialOrd for Crc<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Hash for Crc<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_ref(self.elem.as_ref(), state)
    }
}

pub struct CrcDeDup<E> where E:Eq+Ord {
    //Todo: why RC as key and not E?
    elems:HintMap<Rc<E>,Crc<E>>
}

/*
pub static mut dedup_max:usize = 0;
pub static mut dedup_count:usize = 0;
pub static mut dedup_miss_count:usize = 0;
*/
impl<E:Ord+Eq> CrcDeDup<E> {
    pub fn new() -> Self {
        Self {
            elems: HintMap::new()
        }
    }

    pub const fn new_unvalidated() -> Self {
        Self {
            elems: HintMap::new_unvalidated()
        }
    }

    pub fn dedup(&mut self, elem:E) -> Crc<E> {
        //unsafe {dedup_count+=1};
        let rc = Rc::new(elem);
        let res = self.elems.insert_if_missing(rc,|rc|{
            //unsafe {dedup_miss_count+=1};
            Crc{ elem: rc.clone() }
        }).to_owned();
        //unsafe {dedup_max = dedup_max.max(self.elems.len())};
        res
    }

    pub fn validate(&self) {
        self.elems.validate()
    }
}

pub struct CtrDedup<K,V> where K:Eq+Ord {
    dedup:RefCell<HintMap<K,Crc<V>>>
}

impl<K:Ord+Eq,V> CtrDedup<K,V> {
    pub fn dedup<F:FnOnce(&K)-> V>(&self, key:K, ctr:F) -> Crc<V>{
        //unsafe{dedup_count+=1}
        let res = self.dedup.borrow_mut().insert_if_missing(key,|k|{
            let res = ctr(k);
            //unsafe {dedup_miss_count+=1};
            Crc{elem:Rc::new(res)}
        }).to_owned();
        //unsafe {dedup_max = dedup_max.max(self.dedup.borrow().len())};
        res
    }

    pub fn new() -> Self{
        CtrDedup{dedup:RefCell::new(HintMap::new())}
    }

    pub const fn new_unvalidated() -> Self{
        CtrDedup{dedup:RefCell::new(HintMap::new_unvalidated())}
    }

    pub fn validate(&self) {
        self.dedup.borrow().validate()
    }

}

impl<K:Ord+Eq,V> Default for CtrDedup<K,V> {
    fn default() -> Self {
        CtrDedup::new()
    }
}


//Helper to calc the key for a storage slot
pub fn store_hash(data:&[&[u8]]) -> crate::model::Hash {
    //Make a 20 byte digest hascher
    let mut context = crate::hashing::Hasher::new();
    //push the data into it
    for d in data {
        context.update(*d);
    }
    //calc the Hash
    context.finalize()
}