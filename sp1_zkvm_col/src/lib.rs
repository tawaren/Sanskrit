#![no_std]

#[macro_use]
extern crate sp1_zkvm;
extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::slice::from_raw_parts;
use sp1_zkvm::io::{hint, hint_slice, read, read_vec};
use sp1_zkvm::lib::unconstrained;

pub mod map;
pub mod arena;
pub mod no_free;
pub mod vec;

pub type DefaultIndexType = u16; // Make overwritabel

trait SourceType {
    const EXP:usize;
}

impl SourceType for u8 {
    const EXP: usize = 1;
}

impl SourceType for u16 {
    const EXP: usize = size_of::<u16>()/size_of::<u8>();
}

impl SourceType for u32 {
    const EXP: usize = size_of::<u32>()/size_of::<u8>();
}

impl SourceType for u64 {
    const EXP: usize = size_of::<u64>()/size_of::<u8>();
}

impl SourceType for usize {
    const EXP: usize = size_of::<usize>()/size_of::<u8>();
}

unsafe fn to_u8<T:SourceType>(arr: &[T]) -> &[u8] {
    let len = T::EXP * arr.len();
    let ptr = arr.as_ptr() as *const u8;
    from_raw_parts(ptr, len)
}

unsafe fn from_u8<T:SourceType>(arr: &[u8]) -> &[T] {
    let len = arr.len()/T::EXP;
    let ptr = arr.as_ptr() as *const T;
    from_raw_parts(ptr, len)
}

/*
fn read_index(len:usize) -> usize {
    if len <= u8::MAX as usize {
        read::<u8>() as usize
    } else if len <= u16::MAX as usize {
        read::<u16>() as usize
    } else {
        assert!(len <= u32::MAX as usize);
        read::<u32>() as usize
    }
}

fn write_index(len:usize, index:usize) {
    if len <= u8::MAX as usize {
        hint::<u8>(&(index as u8));
    } else if len <= u16::MAX as usize {
        hint::<u16>(&(index as u16));
    } else {
        assert!(len <= u32::MAX as usize);
        hint::<u32>(&(index as u32));
    }
}
*/
//Increases flexibility of the UniqueBaseArena to be reused for different things
pub trait Select<V,T> {
    fn select(main:&V) -> &T;
}

pub struct IdSelect;
//No Select
impl<T> Select<T,T> for IdSelect {
    #[inline]
    fn select(main: &T) -> &T { main }
}

pub trait SeekMode<T> {
    fn seek<V:Eq,S:Select<T, V>,>(c:&impl Seekable<T>, value:&V) -> usize;
}

pub struct Unconstrained<T>(PhantomData<T>);
impl<T> SeekMode<T> for Unconstrained<T>  {
    fn seek<V: Eq, S: Select<T, V>>(c: &impl Seekable<T>, value: &V) -> usize {
        c.unconstrained_seek::<V,S>(value)
    }
}

pub struct Regular<T>(PhantomData<T>);
impl<T> SeekMode<T> for Regular<T>  {
    fn seek<V: Eq, S: Select<T, V>>(c: &impl Seekable<T>, value: &V) -> usize {
        c.regular_seek::<V,S>(value)
    }
}

pub trait Seekable<T> {
    type I;
    fn deref(inner:&Self::I) -> &T;
    fn with_store<R,F:FnOnce(&[Self::I])->R>(&self, f:F) -> R;

    fn regular_seek<V:Eq,S:Select<T, V>>(&self, value:&V) -> usize {
        self.with_store(|store|{
            let len = store.len();
            let mut index = len;
            for i in 0..len {
                if S::select(Self::deref(&store[i])) == value {
                    index = i;
                    break;
                }
            }
            assert!(index < DefaultIndexType::MAX as usize);
            index
        })
    }

    fn unconstrained_seek<V:Eq,S:Select<T, V>>(&self, value:&V) -> usize {
        unconstrained!{
            let index = self.regular_seek::<V,S>(value);
            hint::<DefaultIndexType>(&(index as DefaultIndexType));
        }
        read::<DefaultIndexType>() as usize
    }

    fn validate_unique_entries<V:Eq+Ord,S:Select<T,V>>(&self) {
        self.with_store(|store|{
            let len = store.len();
            if store.len() < 2 {return;}
            unconstrained!{
                let mut enumerated_vec:Vec<(usize,&T)> = store.iter().map(|b|Self::deref(b)).enumerate().collect();
                enumerated_vec.sort_by(|a,b| S::select(a.1).cmp(S::select(b.1)));
                let idxs:Vec<DefaultIndexType> = enumerated_vec.iter().map(|e|{
                    assert!(e.0 < DefaultIndexType::MAX as usize);
                    e.0 as DefaultIndexType
                }).collect();
                hint_slice(unsafe{to_u8::<DefaultIndexType>(&idxs)});
            }
            let input = read_vec();
            let idxs = unsafe{from_u8::<DefaultIndexType>(&input)};
            let index = idxs[0] as usize;
            assert!(index < len);
            let mut cur = &store[index];
            for i in 1..idxs.len() {
                let index = idxs[i] as usize;
                assert!(index < len);
                let next = &store[index];
                //asserts each key is only in once -- requires total order
                assert!(S::select(Self::deref(cur)) < S::select(Self::deref(next)));
                cur = next;
            }
        });
    }
}