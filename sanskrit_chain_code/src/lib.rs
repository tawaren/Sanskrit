#![no_std]

extern crate sanskrit_common;

#[macro_use]
extern crate sanskrit_derive;

extern crate alloc;

use alloc::vec::Vec;
use sanskrit_common::model::ValueRef;
use crate::model::{Kind};

pub mod model;

pub trait Value<I:ExecutionInterface>:Sized {
    //todo: shall i mark does unsafe?
    //Note these assume static type checking.
    //  if the entry is not the right type it results in undefined behaviour
    fn as_data(&self) -> &[u8];
    fn get_tag(&self) -> u8;
    fn get_field(&self, index:u8) -> &I::Entry;
    //fn call_fun(&self, params: &[I::Entry]) -> &[I::Entry];
    fn as_bool(&self) -> bool;
    fn as_u8(&self) -> u8;
    fn as_i8(&self) -> i8;
    fn as_u16(&self) -> u16;
    fn as_i16(&self) -> i16;
    fn as_u32(&self) -> u32;
    fn as_i32(&self) -> i32;
    fn as_u64(&self) -> u64;
    fn as_i64(&self) -> i64;
    fn as_u128(&self) -> u128;
    fn as_i128(&self) -> i128;
}


pub trait ExecutionInterface:Sized {
    type Entry:Value<Self>;

    fn get(&self, index: ValueRef) -> Self::Entry;
    fn push_entry(&self, tail:bool, entry:Self::Entry);
    fn data_entry(&self, data:Vec<u8>) -> Self::Entry;
    fn adt_entry(&self, tag:u8, fields:Vec<Self::Entry>) -> Self::Entry;
    fn fun_entry(&self, idx:u16, captures:Vec<Self::Entry>) -> Self::Entry;
    //Adt with tag 0 or 1 (and 0 fields) does the same
    fn bool_entry(&self, val:bool)  -> Self::Entry;
    fn u8_entry(&self, val:u8) -> Self::Entry;
    fn i8_entry(&self, val:i8) -> Self::Entry;
    fn u16_entry(&self, val:u16) -> Self::Entry;
    fn i16_entry(&self, val:i16) -> Self::Entry;
    fn u32_entry(&self, val:u32) -> Self::Entry;
    fn i32_entry(&self, val:i32) -> Self::Entry;
    fn u64_entry(&self, val:u64) -> Self::Entry;
    fn i64_entry(&self, val:i64) -> Self::Entry;
    fn u128_entry(&self, val:u128) -> Self::Entry;
    fn i128_entry(&self, val:i128) -> Self::Entry;

    fn process_entry_slice<R: Sized, F: FnOnce(&[u8]) -> R>(kind: Kind, op1: Self::Entry, proc: F) -> R;
}

pub trait RuntimeExternals {
    fn typed_system_call<I:ExecutionInterface>(interface:&mut I, id:u8, kind:Kind, values: &[ValueRef], tail:bool);
    fn system_call<I:ExecutionInterface>(interface:&mut I, id:u8, values: &[ValueRef], tail:bool);
}