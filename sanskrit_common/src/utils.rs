use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use core::ops::Deref;
use core::borrow::Borrow;
use core::clone::Clone;
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

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

pub struct CrcDeDup<E>  {
    //Todo: why RC as key and not E?
    elems:BTreeMap<Rc<E>,Crc<E>>
}

impl<E:Ord+Eq> CrcDeDup<E> {
    pub fn new() -> Self {
        Self {
            elems: BTreeMap::new()
        }
    }

    pub fn dedup(&mut self, elem:E) -> Crc<E> {
        let rc = Rc::new(elem);
        self.elems.entry(rc).or_insert_with_key(|rc|Crc{ elem: rc.clone() }).to_owned()
    }
}
