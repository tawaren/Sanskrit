
use core::cell::RefCell;
use core::{mem, ptr};
use errors::*;
use alloc::vec::Vec;
use core::slice::{from_raw_parts_mut, from_raw_parts};
use encoding::ParserAllocator;
use model::*;
use core::ops::Deref;
use core::ops::DerefMut;
use encoding::VirtualSize;
use core::cell::Cell;
use core::marker::PhantomData;
use core::fmt::{Debug, Formatter};
use core::cmp::Ordering;

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
    pub fn new(real: usize,  convert:f64) -> Self {
        Heap {
            buffer: RefCell::new(Vec::with_capacity(real)),
            pos: Cell::new(0),
            convert,
        }
    }

    pub const fn elems<T:Sized>(num:usize) -> usize {
        num*mem::size_of::<T>()
    }

    pub fn new_arena(&self, size: usize) -> HeapArena {
        let ptr = unsafe { self.buffer.borrow().as_ptr().add(self.pos.get()) };
        let align_offset = align_address(ptr, mem::align_of::<usize>());
        let start = self.pos.get();
        let end = start + size + align_offset;
        if self.buffer.borrow().capacity() < end {
            panic!("Not enough space for allocating arena");
        }
        self.pos.set(end);
        HeapArena {
            buffer: &self.buffer,
            start,
            pos: Cell::new(start),
            end,
            locked:Cell::new(false)
        }
    }

    //todo: make the asserts real checks
    pub fn new_virtual_arena(&self, size: usize) -> VirtualHeapArena {
        //ensures at least size if f > 1
        assert!(self.convert >= 0f64);
        let real_size = size + ((size as f64)*(self.convert-1.0)) as usize;
        assert!(real_size >= size);
        VirtualHeapArena{
            uncounted:self.new_arena(real_size),
            virt_pos: Cell::new(0),
            virt_end_limit:  Cell::new(size),
            virt_end_orig: size
        }
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

    pub fn alloc<T:Sized + Copy>(&self, val: T) -> Ptr<T> {
        if self.locked.get() {panic!()}
        let size = mem::size_of::<T>();
        let pos = self.pos.get();
        unsafe {
            let ptr = self.buffer.borrow_mut().as_mut_ptr().add(pos);
            let align_offset = align_address(ptr, mem::align_of::<T>());
            self.pos.set(pos + align_offset + size);
            if self.end >= self.pos.get() {
                let ptr = ptr.add(align_offset) as *mut T;
                ptr::write(&mut (*ptr), val);
                Ptr(&*ptr)
            } else {
                panic!();
            }
        }
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn alloc_raw_slice<T: Sized + Copy>(&self, len: usize) -> &mut [T] {
        if self.locked.get() {panic!()}
        let size = len * mem::size_of::<T>();
        let pos = self.pos.get();
        let ptr = self.buffer.borrow_mut().as_mut_ptr().add(pos);
        let align_offset = align_address(ptr, mem::align_of::<T>());
        self.pos.set(pos + align_offset + size);
        if self.end >= self.pos.get() {
            from_raw_parts_mut(ptr.add(align_offset) as *mut T, len)
        } else {
            panic!();
        }
    }

    pub fn repeated_mut_slice<T:Copy+Sized>(&self, val:T, len: usize) -> Result<MutSlicePtr<T>> {
        let slice = unsafe {self.alloc_raw_slice(len)};
        for elem in slice.iter_mut() {
            *elem = val;
        }
        MutSlicePtr::new(slice)
    }


    pub fn repeated_slice<T: Sized + Copy>(&self, val: T, len:usize) -> Result<SlicePtr<T>> {
        Ok(self.repeated_mut_slice(val,len)?.freeze())
    }

    pub fn iter_alloc_slice<T: Sized + Copy>(&self, vals: impl ExactSizeIterator<Item = T>) -> Result<SlicePtr<T>> {
        let slice = unsafe {self.alloc_raw_slice(vals.len())};
        for (i,val) in vals.enumerate() {
            slice[i] = val;
        }
        SlicePtr::new(slice)
    }

    pub fn iter_result_alloc_slice<T: Sized + Copy>(&self, vals: impl ExactSizeIterator<Item=Result<T>>) -> Result<SlicePtr<T>> {
        let slice = unsafe {self.alloc_raw_slice(vals.len())};
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


    pub fn temp_arena(&self) -> ArenaLock<Self> {
        if self.locked.get() {panic!();}
        let new = self.unlocked_clone();
        self.locked.set(true);
        ArenaLock{
            old: self,
            new,
        }
    }

    pub fn merge_alloc_slice<T: Sized + Copy + VirtualSize>(&self, vals1: &[T], vals2: &[T]) -> Result<SlicePtr<T>> {
        let slice = unsafe {self.alloc_raw_slice(vals1.len() + vals2.len())};
        slice[..vals1.len()].copy_from_slice(vals1);
        slice[vals1.len()..].copy_from_slice(vals2);
        SlicePtr::new(slice)
    }

    pub fn copy_alloc_slice<T: Sized + Copy>(&self, vals: &[T]) -> Result<SlicePtr<T>> {
        let slice = unsafe {self.alloc_raw_slice(vals.len())};
        slice.copy_from_slice(vals);
        SlicePtr::new(slice)
    }

    pub fn copy_alloc_mut_slice<T: Sized + Copy>(&self, vals: &[T]) -> Result<MutSlicePtr<T>> {
        let slice = unsafe {self.alloc_raw_slice(vals.len())};
        slice.copy_from_slice(vals);
        MutSlicePtr::new(slice)
    }

    pub fn slice_builder<T: Sized + Copy>(&self, len: usize) -> Result<SliceBuilder<T>> {
        if len > u16::max_value() as usize {
            error(||"Size limit exceeded:")
        } else {
            Ok(SliceBuilder::new(unsafe {self.alloc_raw_slice(len)}, 0))
        }
    }

    pub fn alloc_stack<T:Copy+Sized>(&self, size: usize) -> HeapStack<T> {
        let slice = unsafe {self.alloc_raw_slice(size)};
        HeapStack{
            slice,
            pos: 0
        }
    }
}

impl<'o> ParserAllocator for HeapArena<'o>  {
    fn allocated_virtual_bytes(&self) -> usize {
        //just use real size in case we have no virtual
        self.pos.get()
    }


    fn poly_alloc<T: Sized + Copy + VirtualSize>(&self, val: T) -> Result<Ptr<T>> {
        Ok(self.alloc(val))
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
    virt_end_limit:Cell<usize>,
    virt_end_orig:usize,
}

impl<'o> VirtualHeapArena<'o> {

    fn ensure_virt_space(&self, size:usize) -> Result<()>{
        let pos = self.virt_pos.get();
        let new_pos = pos+size;
        if new_pos > self.virt_end_limit.get() {
            return  error(||"Virtual size limit exceeded")
        }
        self.virt_pos.set(new_pos);
        Ok(())
    }

    pub fn remaining_space(&self) -> usize {
        self.virt_end_limit.get() - self.virt_pos.get()
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

    pub fn limit(&self, space:usize) -> Result<()>{
        if self.virt_end_orig - self.virt_pos.get() < space {  return error(||"Not enough virtual space available")}
        Ok(self.virt_end_limit.set(self.virt_pos.get() + space))
    }

    pub fn reuse(self) -> Self {
        VirtualHeapArena{
            uncounted:self.uncounted.reuse(),
            virt_pos:Cell::new(0),
            virt_end_limit:  Cell::new(self.virt_end_orig),
            virt_end_orig: self.virt_end_orig,
        }
    }


    pub fn temp_arena(&self) -> Result<ArenaLock<Self>> {
        if self.uncounted.locked.get() {
            return error(||"Arena already in use by a temporary arena")
        }

        let new = VirtualHeapArena{
            uncounted:self.uncounted.unlocked_clone(),
            virt_pos: Cell::new(self.virt_pos.get()),
            virt_end_limit: Cell::new(self.virt_end_limit.get()),
            virt_end_orig: self.virt_end_orig,

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
        Ok(self.uncounted.alloc(val))
    }

    pub fn copy_alloc_slice<T: Sized + Copy + VirtualSize>(&self, vals: &[T]) -> Result<SlicePtr<T>> {
        self.ensure_virt_space(T::SIZE*vals.len())?;
        self.uncounted.copy_alloc_slice(vals)
    }

    pub fn copy_alloc_mut_slice<T: Sized + Copy + VirtualSize>(&self, vals: &[T]) -> Result<MutSlicePtr<T>> {
        self.ensure_virt_space(T::SIZE*vals.len())?;
        self.uncounted.copy_alloc_mut_slice(vals)
    }

    pub fn slice_builder<T: Sized + Copy + VirtualSize>(&self, len: usize) -> Result<SliceBuilder<T>> {
        self.ensure_virt_space(T::SIZE*len)?;
        self.uncounted.slice_builder(len)
    }
}

#[derive(Debug)]
pub struct HeapStack<'a, T> {
    slice: &'a mut [T],
    pos: usize
}

impl<'a, T:Copy + Sized> HeapStack<'a, T>{

    pub fn push(&mut self, val:T) -> Result<()> {
        if self.pos == self.slice.len() {
            return error(||"Size limit exceeded: max allowed size")
        }
        self.slice[self.pos] = val;
        self.pos += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.pos == 0 { return None; }
        self.pos -= 1;
        Some(self.slice[self.pos])
    }

    pub fn len(&self) -> usize {
        self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, pos:usize) -> Result<&T> {
        if pos > self.pos {
            return error(||"Index out of bounds")
        }
        Ok(&self.slice[pos])
    }

    pub fn get_mut(&mut self, pos:usize) -> Result<&mut T> {
        if pos > self.pos {
            return error(||"Index out of bounds")
        }
        Ok(&mut self.slice[pos])
    }

    pub fn rewind_to(&mut self, pos:usize) -> Result<()> {
        if pos > self.pos {
            return error(||"Index out of bounds")
        }
        self.pos = pos;
        Ok(())
    }

    pub fn transfer_from(&mut self, other:&mut HeapStack<'a, T>) -> Result<()> {
        let end = self.pos+other.pos;
        if end > self.slice.len() {
            return error(||"Size limit exceeded: max allowed size")
        }
        self.slice[self.pos..end].copy_from_slice(other.as_slice());
        self.pos = end;
        other.rewind_to(0)?;
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
    fn allocated_virtual_bytes(&self) -> usize {
        self.virt_pos.get()
    }

    fn poly_alloc<T: Sized + Copy + VirtualSize>(&self, val: T) -> Result<Ptr<T>> {
        self.alloc(val)
    }

    fn poly_slice_builder<T: Sized + Copy + VirtualSize>(&self, len: usize) -> Result<SliceBuilder<T>> {
        self.slice_builder(len)
    }
}

impl<'a,T:Copy> Deref for Ptr<'a, T> {
    type Target = T;

    fn deref(&self) -> &T{
        self.0
    }
}

impl<'a, T:Debug> Debug for Ptr<'a,T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result{
        self.0.fmt(f)
    }
}

impl<'a,T:Copy> SlicePtr<'a,T>{
    fn new(slice:&'a [T]) -> Result<Self>{
        if slice.len() <= u16::max_value() as usize {
            Ok(SlicePtr(slice.len() as u16, slice.as_ptr(), PhantomData))
        } else {
            error(||"Slice Pointer to large")
        }
    }

    pub fn empty() -> Self {
        SlicePtr(0,[].as_ptr(), PhantomData)
    }

    pub fn wrap(val:&'a [T]) -> Self {
        assert!(val.len() < u16::max_value() as usize);
        SlicePtr(val.len() as u16, val.as_ptr(), PhantomData)
    }

    pub fn len(&self) -> usize {
        self.0 as usize
    }
}

impl<'a,T> Deref for SlicePtr<'a, T> {
    type Target = [T];

    fn deref(&self) -> &[T]{
        unsafe {from_raw_parts::<T>(self.1, self.0 as usize)}
    }
}

impl<'a, T:Debug> Debug for SlicePtr<'a,T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        unsafe {from_raw_parts::<T>(self.1, self.0 as usize)}.fmt(f)
    }
}

impl<'a, T:PartialEq> PartialEq for SlicePtr<'a,T> {
    fn eq(&self, other: &Self) -> bool {
       self.deref() == other.deref()
    }
}

impl<'a, T:PartialOrd> PartialOrd for SlicePtr<'a,T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl<'a, T:Ord> Ord for SlicePtr<'a,T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deref().cmp(other.deref())
    }
}

impl<'a,T:Copy> MutSlicePtr<'a,T>{
    fn new(slice:&'a mut [T]) -> Result<Self>{
        if slice.len() <= u16::max_value() as usize {
            Ok(MutSlicePtr(slice.len() as u16, slice.as_mut_ptr(), PhantomData))
        } else {
            error(||"Slice Pointers to large")
        }
    }

    pub fn wrap(val:&'a mut [T]) -> Self {
        assert!(val.len() < u16::max_value() as usize);
        MutSlicePtr(val.len() as u16, val.as_mut_ptr(), PhantomData)
    }

    pub fn freeze(self) -> SlicePtr<'a, T> {
        SlicePtr(self.0, self.1, PhantomData)
    }
}

impl<'a,T:Copy> Deref for MutSlicePtr<'a, T> {
    type Target = [T];

    fn deref(&self) -> &[T]{
        unsafe {from_raw_parts::<T>(self.1, self.0 as usize)}
    }
}

impl<'a,T:Copy> DerefMut for MutSlicePtr<'a, T> {
    fn deref_mut(&mut self) -> &mut [T]{
        unsafe {from_raw_parts_mut::<T>(self.1, self.0 as usize)}
    }
}

//a new type indicating that this is a arena ref
pub struct SliceBuilder<'a, T> {
    slice: &'a mut [T],
    pos: usize
}

impl<'a, T:Copy> SliceBuilder<'a, T>{
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

    pub fn len(&self) -> usize {
        self.pos
    }


    pub fn finish(self) -> SlicePtr<'a,T> {
        SlicePtr::new(&self.slice[..self.pos]).unwrap()
    }

    pub fn finish_mut(self) -> MutSlicePtr<'a,T> {
        MutSlicePtr::new(&mut self.slice[..self.pos]).unwrap()
    }
}