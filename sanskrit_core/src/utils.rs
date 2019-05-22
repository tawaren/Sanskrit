use alloc::vec::Vec;
use alloc::rc::Rc;
use core::ops::Deref;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::marker::PhantomData;
use model::linking::CacheKey;
use alloc::collections::BTreeSet;
use core::clone::Clone;
use sanskrit_common::errors::*;

//A complexity counting Rc to prevent complexity based attacks
//it tracks the Elems and depth and produces an error if the Limits are reached
#[derive(Ord, PartialOrd, Debug)]
pub struct Crc<T>{
    pub elem:Rc<T>  //the actual Element
}

//Helper to speed up comparison by checking the memory address
pub fn same_ref<T>(first:&Crc<T>, other:&Crc<T>) -> bool {
    same_ref_internal::<T>(first.elem.as_ref(), other.elem.as_ref())
}

//Checks if two pointers point to the same memory address
fn same_ref_internal<T>(a: *const T, b: *const T) -> bool {
    a == b
}

impl<T> Crc<T> {
    //To create a new Counted RC the depth and elems have to be given in addition to theelem itself
    pub fn new(elem:T) -> Self {
       Crc{ elem: Rc::new(elem) }
    }
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
impl<T:Eq> Eq for Crc<T> {}
impl<T:PartialEq> PartialEq for Crc<T> {
    fn eq(&self, other: &Self) -> bool {
        //if same pointer look no further
        if same_ref_internal::<T>(self.elem.as_ref(), other.elem.as_ref()) {return true}
        //else do a regular compare
        self.elem == other.elem
    }
}

//A helper enum to allow cycle tracking/prevention in caches
#[derive(Clone)]
pub enum State<T:Clone> {
    Missing,        //not yet processed
    Calculating,    //currently under processing (tying to process again means is in a cycle)
    Present(T),    //already processed does not have to be processed again
}

//A helper enum to make the backing vector of a cache lazy ( allows to generate all caches even if they are not needed) without to much allocation
pub enum LazyCache<T:Clone>{
    Empty(usize),       //An empty cache that knows how big it should be
    Full(Vec<State<T>>) //A cache
}

//A simple memoization cache which works on keys that are ints
// The cache has a specific size and can host keys which ints are smaller than that
// It is specialised to chache deduplicated values
pub struct Cache<K:CacheKey,T:Clone>{
    map:RefCell<LazyCache<T>>,  //the cache
    phantom:PhantomData<K>      //A compiler pleaser
}

//helper function that gets the cache from a lazy cache (&initializes it if necessary)
fn get_cache<T:Clone>(map:&mut LazyCache<T>) -> &mut Vec<State<T>>{
    //checks if it is not initialized and retrieve the size
    fn get_empty<T:Clone>(map:&mut LazyCache<T>) -> Option<usize> {
        match *map {
            LazyCache::Empty(amount) => Some(amount),
            LazyCache::Full(_) => None
        }
    }
    //if it is empty make it full
    if let Some(amount) = get_empty(map) {
        *map = LazyCache::Full(vec![State::Missing;amount])
    }
    //returned the necessarely initialized value
    if let LazyCache::Full(ref mut v) = *map {
        return v
    } else {
        //unreachable as the if always hits
        unreachable!()
    }

}

impl<K:CacheKey + Copy,T:Clone> Cache<K,T>  {

    //Generates a new cache with elements number of Slots
    pub fn new(elements:usize) -> Self {
        Cache{
            //Cache is initialized to None
            map: RefCell::new(LazyCache::Empty(elements)),
            phantom: PhantomData,
        }
    }



    //clones a value out of the cache (if it is there)
    fn get_clone(&self, key:K) -> Result<Option<T>>{
        //just get the cache and then the index
        match get_cache(&mut self.map.borrow_mut())[key.as_key()] {
            //was in their, return it
            State::Present(ref v) => Ok(Some(v.clone())),
            //was not yet computed
            State::Missing => Ok(None),
            //is currently computing we have a cycle (abort)
            State::Calculating => cycle_error(),

        }
    }

    //Gets a value from cache and if it is not there compute it and cache it
    // it is assumed that f is always the same function if the key is the same
    pub fn cached<F>(&self, key:K, f:F) ->  Result<T> where F: FnOnce() -> Result<T>{
        //check if it is their
        match self.get_clone(key)? {
            //it is return it
            Some(v) => Ok(v),
            //it is not comoute it
            None => {
                //mark as computing (preventing cycles)
                get_cache(&mut self.map.borrow_mut())[key.as_key()] = State::Calculating;
                //compute it over the supplied function
                let res = f()?;
                //store the result in the cache
                get_cache(&mut self.map.borrow_mut())[key.as_key()] = State::Present(res.clone());
                //return the result
                Ok(res)
            },
        }
    }
}

//helper to quickly fill a set
pub fn build_set<T:Ord+Clone>(args:&[T]) -> BTreeSet<T> {
    //create the empty set
    let mut set = BTreeSet::new();
    //iterate over the supplied elems
    for cap in args {
        //insert them
        set.insert(cap.clone());
    }
    //return the reslt
    set
}
