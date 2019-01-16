
use core::cell::RefCell;
use core::{mem, ptr};
use errors::*;
use alloc::prelude::Vec;
use core::slice::from_raw_parts_mut;
use encoding::ParserAllocator;
use model::*;
use core::ops::Deref;
use core::ops::DerefMut;
use linear_stack::MicroVec;
use linear_stack::MicroVecBuilder;
use encoding::VirtualSize;
use core::cell::Cell;

fn align_address(ptr: *const u8, align: usize) -> usize {
    let addr = ptr as usize;
    if addr % align != 0 {
        align - addr % align
    } else {
        0
    }
}

pub struct Heap {
    buffer: RefCell<Vec<u8>>,
    pos: Cell<usize>,
    convert:f64,
}

impl Heap {
    pub fn new(real: usize, virt:usize, convert:f64) -> Self {
        let virt = virt + ((virt as f64)*(convert-1.0)) as usize;
        Heap {
            buffer: RefCell::new(Vec::with_capacity(real + virt)),
            pos: Cell::new(0),
            convert,
        }
    }

    pub fn words(size:usize) -> usize {
        size*8
    }

    pub fn elems<T:Sized>(num:usize) -> usize {
        num*mem::size_of::<T>()
    }

    pub fn new_arena(&self, size: usize) -> Result<HeapArena> {
        let ptr = unsafe { self.buffer.borrow().as_ptr().offset(self.pos.get() as isize) };
        let align_offset = align_address(ptr, mem::align_of::<usize>());
        let start = self.pos.get();
        let end = start + size + align_offset;
        if self.buffer.borrow().capacity() < end {
            return size_limit_exceeded_error()
        }
        self.pos.set(end);
        Ok(HeapArena {
            buffer: &self.buffer,
            start,
            pos: Cell::new(start),
            end,
            locked:Cell::new(false)
        })
    }

    pub fn new_virtual_arena(&self, size: usize) -> Result<VirtualHeapArena> {
        //ensures at least size if f > 1
        let real_size = size + ((size as f64)*(self.convert-1.0)) as usize;
        assert!(real_size > 0);
        Ok(VirtualHeapArena{
            uncounted:self.new_arena(real_size)?,
            virt_pos: Cell::new(0),
            virt_end: size
        })
    }

    pub fn reuse(self) -> Self  {
        Heap {
            buffer: self.buffer,
            pos: Cell::new(0),
            convert: self.convert,
        }
    }
}

pub struct HeapArena<'h> {
    buffer: &'h RefCell<Vec<u8>>,
    start: usize,
    pos: Cell<usize>,
    end: usize,
    locked: Cell<bool>,
}

pub trait ArenaUnlock {
    fn set_lock(&self, val:bool);
    fn get_lock(&self) -> bool;
}

pub struct ArenaLock<'a, T:ArenaUnlock> {
    old: &'a T,
    new: T
}

impl<'a, T:ArenaUnlock> Deref for ArenaLock<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
       &self.new
    }
}

impl<'a, T:ArenaUnlock> DerefMut for ArenaLock<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.new
    }
}

impl<'a, T:ArenaUnlock> Drop for ArenaLock<'a, T> {
    fn drop(&mut self) {
        self.old.set_lock(false);
        //just to be sure probably not necessary
        assert!(!self.new.get_lock());
        self.new.set_lock(true);
    }
}

impl<'h> HeapArena<'h> {
    unsafe fn reserve(&self, size: usize, align: usize) -> *mut u8 {
        let align_offset = align_address(self.buffer.borrow().as_ptr().offset(self.pos.get() as isize), align);
        let ptr = self.buffer.borrow_mut().as_mut_ptr().offset((self.pos.get()  + align_offset) as isize);
        self.pos.set(self.pos.get() + size + align_offset);
        ptr
    }

    fn has_room(&self, size: usize, align: usize) -> bool {
        if self.locked.get() {return false}
        let ptr = unsafe { self.buffer.borrow().as_ptr().offset(self.pos.get() as isize) };
        let align_offset = align_address(ptr, align);
        self.end >= self.pos.get() + size + align_offset
    }

    unsafe fn alloc_raw_slice<T: Sized + Copy>(&self, len: usize) -> Result<&mut [T]> {
        let size = len * mem::size_of::<T>();
        if self.has_room(size, mem::align_of::<T>()) {
            let ptr = self.reserve(size, mem::align_of::<T>()) as *mut T;
            Ok(from_raw_parts_mut(ptr, len))
        } else {
            size_limit_exceeded_error()
        }
    }

    pub fn repeated_mut_slice<T:Copy+Sized>(&self, val:T, len: usize) -> Result<MutSlicePtr<T>> {
        let mut slice = unsafe {self.alloc_raw_slice(len)?};
        for i in 0..len {
            slice[i] = val;
        }
        MutSlicePtr::new(slice)
    }


    pub fn repeated_slice<T: Sized + Copy>(&self, val: T, len:usize) -> Result<SlicePtr<T>> {
        Ok(self.repeated_mut_slice(val,len)?.freeze())
    }



    pub fn iter_alloc_slice<T: Sized + Copy>(&self, vals: impl ExactSizeIterator<Item = T>) -> Result<SlicePtr<T>> {
        let mut slice = unsafe {self.alloc_raw_slice(vals.len())?};
        for (i,val) in vals.enumerate() {
            slice[i] = val;
        }
        SlicePtr::new(slice)
    }

    pub fn iter_result_alloc_slice<T: Sized + Copy>(&self, vals: impl ExactSizeIterator<Item=Result<T>>) -> Result<SlicePtr<T>> {
        let mut slice = unsafe {self.alloc_raw_slice(vals.len())?};
        for (i,val) in vals.enumerate() {
            slice[i] = val?;
        }
        SlicePtr::new(slice)
    }

    pub fn reuse(self) -> Self {
        HeapArena {
            buffer: self.buffer,
            start: self.start,
            pos: Cell::new(self.start),
            end: self.end,
            locked: Cell::new(false),
        }
    }

    fn unlocked_clone(&self) -> Self {
        HeapArena {
            buffer: self.buffer,
            start: self.pos.get(),
            pos: Cell::new(self.pos.get()),
            end: self.end,
            locked: Cell::new(false),
        }
    }

    pub fn temp_arena(&self) -> Result<ArenaLock<Self>> {
        if self.locked.get() {return size_limit_exceeded_error();}
        let new = self.unlocked_clone();
        self.locked.set(true);
        Ok(ArenaLock{
            old: self,
            new,
        })
    }

    pub fn alloc<T:Sized + Copy>(&self, val: T) -> Result<Ptr<T>> {
        if self.has_room(mem::size_of::<T>(), mem::align_of::<T>()) {
            unsafe {
                let ptr = self.reserve(mem::size_of::<T>(), mem::align_of::<T>());
                let ptr = ptr as *mut T;
                ptr::write(&mut (*ptr), val);
                Ok(Ptr(&*ptr))
            }
        } else {
            size_limit_exceeded_error()
        }
    }

    pub fn merge_alloc_slice<T: Sized + Copy + VirtualSize>(&self, vals1: &[T], vals2: &[T]) -> Result<SlicePtr<T>> {
        let slice = unsafe {self.alloc_raw_slice(vals1.len() + vals2.len())?};
        slice[..vals1.len()].copy_from_slice(vals1);
        slice[vals1.len()..].copy_from_slice(vals2);
        SlicePtr::new(slice)
    }

    pub fn copy_alloc_slice<T: Sized + Copy>(&self, vals: &[T]) -> Result<SlicePtr<T>> {
        let slice = unsafe {self.alloc_raw_slice(vals.len())?};
        slice.copy_from_slice(vals);
        SlicePtr::new(slice)
    }

    pub fn slice_builder<T: Sized + Copy>(&self, len: usize) -> Result<SliceBuilder<T>> {
        if len > u16::max_value() as usize {
            size_limit_exceeded_error()
        } else {
            Ok(SliceBuilder::new(unsafe {self.alloc_raw_slice(len)?}, 0))
        }
    }

    pub fn alloc_stack<T:Copy+Sized>(&self, size: usize) -> Result<HeapStack<T>> {
        let mut slice = unsafe {self.alloc_raw_slice(size)?};
        Ok(HeapStack{
            slice,
            pos: 0
        })
    }



}

impl<'o> ParserAllocator for HeapArena<'o>  {
    fn poly_alloc<T: Sized + Copy + VirtualSize>(&self, val: T) -> Result<Ptr<T>> {
        self.alloc(val)
    }

    fn poly_slice_builder<T: Sized + Copy + VirtualSize>(&self, len: usize) -> Result<SliceBuilder<T>> {
        self.slice_builder(len)
    }
}

impl<'o> ArenaUnlock for HeapArena<'o> {
    fn set_lock(&self, val: bool) {
        self.locked.set(val)
    }

    fn get_lock(&self) -> bool {
        self.locked.get()
    }
}

pub struct VirtualHeapArena<'o>{
    uncounted:HeapArena<'o>,
    virt_pos:Cell<usize>,
    virt_end:usize,
}

impl<'o> VirtualHeapArena<'o> {

    fn ensure_virt_space(&self, size:usize) -> Result<()>{
            self.virt_pos.set(self.virt_pos.get()+size);
            if self.virt_pos.get() >= self.virt_end {return size_limit_exceeded_error()}
            Ok(())
    }

    pub fn repeated_slice<T: Sized + Copy + VirtualSize>(&self, val: T, len:usize) -> Result<SlicePtr<T>> {
        self.ensure_virt_space(T::SIZE*len)?;
        self.uncounted.repeated_slice(val,len)
    }

    pub fn iter_alloc_slice<T: Sized + Copy + VirtualSize>(&self, vals: impl ExactSizeIterator<Item = T>) -> Result<SlicePtr<T>> {
        self.ensure_virt_space(T::SIZE*vals.len())?;
        self.uncounted.iter_alloc_slice(vals)
    }

    pub fn iter_result_alloc_slice<T: Sized + Copy + VirtualSize>(&self, vals: impl ExactSizeIterator<Item=Result<T>>) -> Result<SlicePtr<T>> {
        self.ensure_virt_space(T::SIZE*vals.len())?;
        self.uncounted.iter_result_alloc_slice(vals)
    }

    pub fn reuse(self) -> Self {
        VirtualHeapArena{
            uncounted:self.uncounted.reuse(),
            virt_pos:Cell::new(0),
            virt_end: self.virt_end
        }
    }

    pub fn temp_arena(&self) -> Result<ArenaLock<Self>> {
        if self.uncounted.locked.get() {return size_limit_exceeded_error();}
        let new = VirtualHeapArena{
            uncounted:self.uncounted.unlocked_clone(),
            virt_pos: Cell::new(self.virt_pos.get()),
            virt_end: self.virt_end
        };
        self.uncounted.locked.set(true);
        Ok(ArenaLock{
            old: self,
            new,
        })
    }

    pub fn merge_alloc_slice<T: Sized + Copy + VirtualSize>(&self, vals1: &[T], vals2: &[T]) -> Result<SlicePtr<T>> {
        self.ensure_virt_space(T::SIZE*(vals1.len()+vals2.len()))?;
        self.uncounted.merge_alloc_slice(vals1, vals2)
    }

    pub fn alloc<T: Sized + Copy + VirtualSize>(&self, val: T) -> Result<Ptr<T>> {
        self.ensure_virt_space(T::SIZE)?;
        self.uncounted.alloc(val)
    }

    pub fn copy_alloc_slice<T: Sized + Copy + VirtualSize>(&self, vals: &[T]) -> Result<SlicePtr<T>> {
        self.ensure_virt_space(T::SIZE*vals.len())?;
        self.uncounted.copy_alloc_slice(vals)
    }

    pub fn slice_builder<T: Sized + Copy + VirtualSize>(&self, len: usize) -> Result<SliceBuilder<T>> {
        self.ensure_virt_space(T::SIZE*len)?;
        self.uncounted.slice_builder(len)
    }
}


pub struct HeapStack<'a, T> {
    slice: &'a mut [T],
    pos: usize
}

impl<'a, T:Copy + Sized> HeapStack<'a, T>{

    pub fn push(&mut self, val:T) -> Result<()> {
        if self.pos == self.slice.len() {
            return size_limit_exceeded_error()
        }
        self.slice[self.pos] = val;
        self.pos += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.pos == 0 { return None; }
        self.pos -= 1;
        Some(unsafe {mem::replace(&mut self.slice[self.pos], mem::uninitialized()) })
    }

    pub fn len(&self) -> usize {
        self.pos
    }

    pub fn get(&self, pos:usize) -> Result<&T> {
        if pos > self.pos {return out_of_range_stack_addressing()}
        Ok(&self.slice[pos])
    }

    pub fn get_mut(&mut self, pos:usize) -> Result<&mut T> {
        if pos > self.pos {return out_of_range_stack_addressing()}
        Ok(&mut self.slice[pos])
    }

    pub fn rewind_to(&mut self, pos:usize) -> Result<()> {
        if pos > self.pos {return out_of_range_stack_addressing()}
        self.pos = pos;
        Ok(())
    }

    pub fn as_slice(&self) -> &[T] {
        &self.slice[..self.pos]
    }
}


impl<'o> ArenaUnlock for VirtualHeapArena<'o> {
    fn set_lock(&self, val: bool) {
        self.uncounted.locked.set(val)
    }

    fn get_lock(&self) -> bool {
        self.uncounted.locked.get()
    }
}

impl<'o> ParserAllocator for VirtualHeapArena<'o> {
    fn poly_alloc<T: Sized + Copy + VirtualSize>(&self, val: T) -> Result<Ptr<T>> {
        self.alloc(val)
    }

    fn poly_slice_builder<T: Sized + Copy + VirtualSize>(&self, len: usize) -> Result<SliceBuilder<T>> {
        self.slice_builder(len)
    }
}

impl<'a,T> Deref for Ptr<'a, T> {
    type Target = T;

    fn deref(&self) -> &T{
        self.0
    }
}

impl<'a,T> SlicePtr<'a,T>{
    fn new(slice:&'a [T]) -> Result<Self>{
        if slice.len() <= u16::max_value() as usize {
            Ok(SlicePtr(slice))
        } else {
            size_limit_exceeded_error()
        }
    }

    pub fn empty() -> Self {
        SlicePtr(&[])
    }

    pub fn wrap(val:&'a [T]) -> Self {
        SlicePtr(val)
    }
}

impl<'a,T> Deref for SlicePtr<'a, T> {
    type Target = [T];

    fn deref(&self) -> &[T]{
        self.0
    }
}

impl<'a,T> MutSlicePtr<'a,T>{
    fn new(slice:&'a mut [T]) -> Result<Self>{
        if slice.len() <= u16::max_value() as usize {
            Ok(MutSlicePtr(slice))
        } else {
            size_limit_exceeded_error()
        }
    }

    pub fn wrap(val:&'a mut [T]) -> Self {
        MutSlicePtr(val)
    }

    pub fn freeze(self) -> SlicePtr<'a, T> {
        SlicePtr(self.0)
    }
}

impl<'a,T> Deref for MutSlicePtr<'a, T> {
    type Target = [T];

    fn deref(&self) -> &[T]{
        self.0
    }
}

impl<'a,T> DerefMut for MutSlicePtr<'a, T> {
    fn deref_mut(&mut self) -> &mut [T]{
        self.0
    }
}

//a new type indicating that this is a arena ref
pub struct SliceBuilder<'a, T> {
    slice: &'a mut [T],
    pos: usize
}

impl<'a, T> SliceBuilder<'a, T>{
    fn new(slice:&'a mut [T], pos:usize) -> Self {
        SliceBuilder{
            slice,
            pos
        }
    }

    pub fn push(&mut self, val:T) {
        self.slice[self.pos] = val;
        self.pos += 1;
    }

    pub fn finish(self) -> SlicePtr<'a,T> {
        SlicePtr::new(&self.slice[..self.pos]).unwrap()
    }

    pub fn finish_mut(self) -> MutSlicePtr<'a,T> {
        MutSlicePtr::new(&mut self.slice[..self.pos]).unwrap()
    }
}


impl<'a,T:Clone> MicroVec<T> for SlicePtr<'a,T>{
    fn zero() -> Self {
        SlicePtr::empty()
    }

    fn is_empty(&self) -> bool {
        self.deref().is_empty()
    }

    fn slice(&self) -> &[T] {
        self.deref()
    }
}

impl<'a, T:Clone> MicroVecBuilder<T> for SliceBuilder<'a,T>{
    type MicroVec = SlicePtr<'a,T>;

    fn push(&mut self, val: T) -> Result<()> {
        (self as &mut SliceBuilder<'a,T>).push(val);
        Ok(())
    }

    fn finish(self) -> Self::MicroVec {
        (self as SliceBuilder<'a,T>).finish()
    }
}


impl<T:Clone> MicroVec<T> for Vec<T>{
    fn zero() -> Self {
        Vec::with_capacity(0)
    }

    fn is_empty(&self) -> bool {
        (self as &Vec<T>).is_empty()
    }

    fn slice(&self) -> &[T] {
        self
    }
}


impl<T:Clone> MicroVecBuilder<T> for Vec<T>{
    type MicroVec = Vec<T>;

    fn push(&mut self, val: T) -> Result<()> {
        (self as &mut Vec<T>).push(val);
        Ok(())
    }

    fn finish(self) -> Self::MicroVec {
        self
    }
}