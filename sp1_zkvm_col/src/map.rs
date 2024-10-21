use alloc::vec::Vec;
use core::mem::{replace, zeroed};
use core::ops::{Deref, DerefMut, Index, IndexMut};
use crate::{Seekable, Select};

pub enum Entry<K,V> {
    Occupied(K,V),
    Vacant(K)
}

impl<K,V> Entry<K,V>  {
    pub fn key(&self) -> &K {
        match self {
            Entry::Occupied(ref k,_)
            | Entry::Vacant(ref k) => k
        }
    }

    pub fn value(&self) -> Option<&V> {
        match self {
            Entry::Occupied(_,ref v) => Some(v),
            Entry::Vacant(_) => None
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Entry::Occupied(_,_) => false,
            Entry::Vacant(_) => true
        }
    }
}

pub struct Iter<'a, K,V> {
    index:usize,
    store:&'a Vec<Entry<K,V>>
}

const DEFAULT_INITIAL_CAPACITY:usize = 128;

pub struct WeakHintMap<K,V, const IC:usize = DEFAULT_INITIAL_CAPACITY> {
    vacant:usize,
    store:Vec<Entry<K,V>>
}

/// *Warning:* Only use WeakHintMap directly if you know what you are doing. Otherwise use HintMap
///            It will not validate the hints on drop, nor has it the validate method to do it manually
///            Their are use cases where skipping validate is ok
///            For example an attacker could do the following:
///             claim an entry is absent if it is their
///             have multiple entries for the same key
///                 choose which entry to use if one is requested
impl<K, V, const IC:usize> WeakHintMap<K,V,IC> {
    pub fn new() -> Self {
        WeakHintMap{ vacant:0, store:Vec::with_capacity(IC)}
    }

    pub const fn new_const() -> Self {
        WeakHintMap{ vacant:0, store:Vec::new()}
    }

    pub fn reserve_capacity(&mut self, cap:usize){
        self.store.reserve(cap);
    }

    pub fn reserve_default_capacity(&mut self){
        self.reserve_capacity(DEFAULT_INITIAL_CAPACITY)
    }

    pub fn len(&self) -> usize {
        self.store.len() - self.vacant
    }

    pub fn is_empty(&self) -> bool {
        self.store.len() == self.vacant
    }

    pub fn iter(&self) -> Iter<K,V> {
        Iter {index:0, store:&self.store}
    }
}

struct KeySelect;

impl<K,V> Select<Entry<K,V>,K> for KeySelect {
    fn select(main: &Entry<K,V>) -> &K {
        main.key()
    }
}

enum SeekRes<T> {
    Hit(T),
    Miss
}

impl<K:Eq,V, const IC:usize> WeakHintMap<K,V,IC> {
    pub fn insert(&mut self, key:K, value:V) -> Option<V>{
       let index = self.unconstrained_seek::<K,KeySelect>(&key);
       if index < self.store.len() {
           assert!(*self.store[index].key() == key);
           match replace(&mut self.store[index], Entry::Occupied(key,value)) {
               Entry::Occupied(_, v) => Some(v),
               Entry::Vacant(_) => {
                   self.vacant -=1;
                   None
               }
           }
       } else {
           assert!(index == self.store.len());
           if self.store.capacity() == 0 {self.store.reserve(IC)}
           self.store.push(Entry::Occupied(key,value));
           None
       }
    }

    fn base_remove(&mut self, key:&K) -> SeekRes<Option<V>>{
        let index = self.unconstrained_seek::<K,KeySelect>(key);
        if index >= self.store.len() { return SeekRes::Miss }
        let entry = &self.store[index];
        assert!(entry.key() == key);
        if entry.is_empty() { return SeekRes::Hit(None) }
        if let Entry::Occupied(k,v) = replace(&mut self.store[index], unsafe{zeroed()}){
            self.store[index] = Entry::Vacant(k);
            self.vacant +=1;
            SeekRes::Hit(Some(v))
        } else {
            unreachable!()
        }
    }

    pub fn weak_remove(&mut self, key:&K) -> Option<V>{
       match self.base_remove(key) {
           SeekRes::Hit(v) => v,
           SeekRes::Miss => None
       }
    }

    //Todo: consider a K version for when we have it owned
    pub fn remove(&mut self, key:&K) -> Option<V> where K:Clone{
        match self.base_remove(&key) {
            SeekRes::Hit(v) => v,
            SeekRes::Miss => {
                self.store.push(Entry::Vacant(key.clone()));
                None
            }
        }
    }

    fn base_get(&self, key:&K) -> SeekRes<Option<&V>>{
        let index = self.unconstrained_seek::<K,KeySelect>(key);
        if index >= self.store.len() { return SeekRes::Miss }
        let entry = &self.store[index];
        assert!(entry.key() == key);
        match entry {
            Entry::Occupied(_, ref v) => SeekRes::Hit(Some(v)),
            Entry::Vacant(_) => SeekRes::Hit(None)
        }
    }

    pub fn weak_get(&self, key:&K) -> Option<&V> {
        match self.base_get(key) {
            SeekRes::Hit(v) => v,
            SeekRes::Miss => None
        }
    }

    //Todo: consider a K version for when we have it owned
    pub fn get(&mut self, key:&K) -> Option<&V> where K:Clone{
        //For later, sadly we need to trick borrow checker here
        //    as he does not know that None does not capture a &V
        let self_ptr = self as *mut Self;
        if let SeekRes::Hit(v) = self.base_get(&key) {
            v
        } else {
            // At this point, we know it was a miss and a None will be returned
            // We won't need the result of self.base_get(&key) anymore (all &V's are gone).
            // thus we can safely do a mutation.
            unsafe { (*self_ptr).store.push(Entry::Vacant(key.clone())); }
            None
        }
    }

    fn base_mut_get(&mut self, key:&K) -> SeekRes<Option<&mut V>>{
        let index = self.unconstrained_seek::<K,KeySelect>(key);
        if index >= self.store.len() { return SeekRes::Miss }
        let entry = &mut self.store[index];
        assert!(entry.key() == key);
        match entry {
            Entry::Occupied(_, ref mut v) => SeekRes::Hit(Some(v)),
            Entry::Vacant(_) => SeekRes::Hit(None)
        }
    }

    pub fn weak_get_mut(&mut self, key:&K) -> Option<&mut V> {
        match self.base_mut_get(key) {
            SeekRes::Hit(v) => v,
            SeekRes::Miss => None
        }
    }

    //Todo: consider a K version for when we have it owned
    pub fn get_mut(&mut self, key:&K) -> Option<&mut V> where K:Clone {
        //For later, sadly we need to trick borrow checker here
        //    as he does not know that None does not capture a &V
        let self_ptr = self as *mut Self;
        if let SeekRes::Hit(v) = self.base_mut_get(&key) {
            v
        } else {
            // At this point, we know it was a miss and a None will be returned
            // We won't need the result of self.base_get(&key) anymore (all &V's are gone).
            // thus we can safely do a mutation.
            unsafe { (*self_ptr).store.push(Entry::Vacant(key.clone())); }
            None
        }
    }

    pub fn contains_key_weak(&self, key:&K) -> bool {
        match self.base_get(key) {
            SeekRes::Hit(_) => true,
            SeekRes::Miss => false
        }
    }

    //Todo: consider a K version for when we have it owned
    pub fn contains_key(&mut self, key:&K) -> bool where K:Clone {
        match self.base_get(&key) {
            SeekRes::Hit(_) => true,
            SeekRes::Miss => {
                self.store.push(Entry::Vacant(key.clone()));
                false
            }
        }
    }

    //entry api is too complicated so we have this instead
    pub fn insert_if_missing<F:FnOnce(&K) -> V>(&mut self, key:K, f:F) -> &V {
        let index = self.unconstrained_seek::<K,KeySelect>(&key);
        if index < self.store.len() {
            let entry = &self.store[index];
            assert!(*entry.key() == key);
            if entry.is_empty() {
                if let Entry::Vacant(k) = replace(&mut self.store[index], unsafe{zeroed()}){
                    self.store[index] = Entry::Occupied(k, f(&key));
                    self.vacant -=1;
                }
            }
        } else {
            assert!(index == self.store.len());
            if self.store.capacity() == 0 {self.store.reserve(IC)}
            let res = f(&key);
            self.store.push(Entry::Occupied(key, res))
        }
        &self.store[index].value().unwrap()
    }
}

impl<K:Eq,V, const IC:usize> Seekable<Entry<K,V>> for WeakHintMap<K,V,IC> {
    type I = Entry<K,V>;
    #[inline]
    fn deref(inner: &Self::I) -> &Entry<K,V> { inner }
    #[inline]
    fn with_store<F: FnOnce(&[Self::I]) -> ()>(&self, f: F) {
        f(&self.store)
    }
}

impl<K:Eq,V, const IC:usize> Index<&K> for WeakHintMap<K,V,IC> {
    type Output = V;
    fn index(&self, key: &K) -> &Self::Output {
        self.weak_get(key).expect("Key not found")
    }
}

impl<K:Eq,V, const IC:usize> IndexMut<&K> for WeakHintMap<K,V,IC> {
    fn index_mut(&mut self, key: &K) -> &mut Self::Output {
        self.weak_get_mut(key).expect("Key not found")
    }
}

pub struct HintMap<K,V, const IC:usize = DEFAULT_INITIAL_CAPACITY>(WeakHintMap<K,V,IC>) where K:Eq+Ord;

impl<K:Ord+Eq,V,const IC:usize> HintMap<K,V,IC> {
    /// **Warning**: If indirectly owned by a static value make sure to call `validate()` manually before the program exits.
    ///              If the collection is dropped regularly then 'drop()' will call `validate()`
    ///              An attacker can craft hints that make the collection behave wrongly
    ///              Only by calling `validate()` are those attempts detected
    pub fn new() -> Self {
        HintMap(WeakHintMap::<K,V,IC>::new())
    }
    /// Use this constructor if you need a static instance.
    /// **Warning**: Make sure to call `validate()` manually before the program exits.
    ///              An attacker can craft hints that make the collection behave wrongly
    ///              Only by calling `validate()` are those attempts detected
    pub const fn new_unvalidated() -> Self {
        HintMap(WeakHintMap::<K,V,IC>::new_const())
    }

    /// Panics if wrong hints were supplied
    /// Takes O((n+d)*k) time where
    ///     n = number of entries
    ///     d = number of deleted entries
    ///     k = size of the keys
    pub fn validate(&self) {
        self.validate_unique_entries::<K,KeySelect>()
    }
}

impl<K:Eq+Ord,V,const IC:usize> Drop for HintMap<K,V,IC>{
    fn drop(&mut self) {
        self.validate()
    }
}

impl<K:Eq+Ord,V,const IC:usize> Deref for HintMap<K,V,IC> {
    type Target = WeakHintMap<K,V,IC>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<K:Eq+Ord,V,const IC:usize> DerefMut for HintMap<K,V,IC> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, K,V> Iterator for Iter<'a, K,V> {
    type Item = (&'a K,&'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.store.len() {return None};
        let mut e = &self.store[self.index];
        self.index+=1;
        while e.is_empty() && self.index < self.store.len() {
            e = &self.store[self.index];
            self.index+=1;
        }

        match e {
            Entry::Occupied(k, v) => Some((k,v)),
            Entry::Vacant(_) => None
        }
    }
}
