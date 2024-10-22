use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::collections::BTreeSet;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use byteorder::{BigEndian, ByteOrder};
use core::ops::Deref;
use crate::model::*;

pub type EncodingByteOrder = BigEndian;


//the state needed during parsing including the input
pub struct Parser<'a>{
    pub index:usize,
    data:&'a [u8]
}

impl<'a> Parser<'a> {
    //create a new parser for an input
    pub fn new(data:&'a [u8]) -> Self {
        Parser {
            index: 0,
            data,
        }
    }

    //parse the whole content as a specific Parsable
    pub fn parse_fully<T:Parsable>(data:&'a [u8]) -> T{
        let mut parser = Parser::new(data);
        let parsed = T::parse(&mut parser);
        assert!(parser.data.len() == parser.index);
        parsed
    }

    //fetch a single byte
    pub fn consume_byte(&mut self) -> u8{
        assert!(self.index < self.data.len());
        let res = self.data[self.index];
        self.index+=1;
        res
    }

    //fetch a fix amount of bytes
    pub fn consume_bytes(&mut self, amount:usize) -> &[u8]{
        assert!(self.index+amount <= self.data.len());
        let res = &self.data[self.index..(self.index+amount)];
        self.index+=amount;
        res
    }

}

pub trait Parsable where Self:Sized {
    fn parse(p: &mut Parser) -> Self;
}

//the state during serialization
//todo: can we directly serialize to store??
#[derive(Default)]
pub struct Serializer{
    out:Vec<u8>
}

impl Serializer{

    //Creates a new serializer
    pub fn new() -> Self {
        Serializer{ out:Vec::new() }
    }

    //parse the whole content as a specific Parsable
    pub fn serialize_fully<S:Serializable>(data:&S) -> Vec<u8>{
        let mut serializer = Serializer::new();
        data.serialize(&mut serializer);
        serializer.extract()
    }

    //Emits a single byte
    pub fn produce_byte(&mut self, byte:u8){
        self.out.push(byte)
    }

    //emit a fixed number of bytes
    pub fn produce_bytes(&mut self, bytes:&[u8]){
        for b in bytes{
            self.out.push(*b)
        }
    }

    //allocates an outbut buffer and returns a pointer to it
    pub fn get_buf(&mut self, amount:usize) -> &mut [u8]{
        let start = self.out.len();
        for _ in 0..amount {
            self.out.push(0)
        }
        &mut self.out[start..]
    }

    //gets the result
    pub fn extract(self) -> Vec<u8>{
        self.out
    }
}

//A trait for Serializable types
pub trait Serializable {
    fn serialize(&self, s:&mut Serializer);
}

//helps with turo fish representation ambiguity allows TypeId::<orignalType>::SIZE
// where Original type is in X<A> Notation instead of X::<A>
pub type TypeId<T> = T;
pub const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}

//Implementations for non- application specific types
impl<T:Parsable> Parsable for Vec<T>{
    fn parse(p: &mut Parser) -> Self {
        let len = p.consume_byte();
        let mut elems = Vec::with_capacity(len as usize);
        for _ in 0..len {
            elems.push(Parsable::parse(p));
        }
        elems
    }
}

impl<T:Serializable> Serializable for Vec<T>{
    fn serialize(&self, s:&mut Serializer){
        assert!(self.len() <= u8::MAX as usize);
        s.produce_byte(self.len() as u8);
        for elem in self {
            elem.serialize(s);
        }
    }
}

impl<T> Deref for LargeVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T:Parsable> Parsable for LargeVec<T>{
    fn parse(p: &mut Parser) -> Self {
        let len = u16::parse(p);
        let mut elems = Vec::with_capacity(len as usize);
        for _ in 0..len {
            elems.push(Parsable::parse(p));
        }
        LargeVec(elems)
    }
}

impl<T:Serializable> Serializable for LargeVec<T>{
    fn serialize(&self, s:&mut Serializer) {
        assert!(self.0.len() <= u16::MAX as usize);
        (self.0.len() as u16).serialize(s);
        for elem in &self.0 {
            elem.serialize(s);
        }
    }
}

impl Parsable for Hash {
    fn parse(p: &mut Parser) -> Self {
        hash_from_slice(p.consume_bytes(20))
    }
}

impl Serializable for Hash {
    fn serialize(&self, s:&mut Serializer) {
        s.produce_bytes(&self[..]);
    }
}

impl Parsable for [u8;32]{
    fn parse(p: &mut Parser) -> Self {
        array_ref!(p.consume_bytes(32), 0, 32).to_owned()
    }
}

impl Serializable for [u8;32]{
    fn serialize(&self, s:&mut Serializer) {
        s.produce_bytes(&self[..]);
    }
}

impl Parsable for [u8;64]{
    fn parse(p: &mut Parser) -> Self {
        array_ref!(p.consume_bytes(64), 0, 64).to_owned()
    }
}

impl Serializable for [u8;64]{
    fn serialize(&self, s:&mut Serializer) {
        s.produce_bytes(&self[..]);
    }
}

impl<T:Parsable+Ord> Parsable for BTreeSet<T>{
    fn parse(p: &mut Parser) -> Self {
        let len = p.consume_byte();
        let mut elems = BTreeSet::new();
        for _ in 0..len {
            elems.insert(T::parse(p));
        }
        elems
    }
}

impl<T:Serializable+Ord> Serializable for BTreeSet<T>{
    fn serialize(&self, s:&mut Serializer) {
        assert!(self.len() <= 255);
        s.produce_byte(self.len() as u8);
        for elem in self {
            elem.serialize(s);
        }
    }
}

impl<K:Parsable+Ord,V:Parsable> Parsable for BTreeMap<K,V>{
    fn parse(p: &mut Parser) -> Self {
        let len = p.consume_byte();
        let mut elems = BTreeMap::new();
        for _ in 0..len {
            elems.insert(K::parse(p),V::parse(p));
        }
        elems
    }
}


impl<K:Serializable+Ord,V:Serializable> Serializable for BTreeMap<K,V>{

    fn serialize(&self, s:&mut Serializer) {
        assert!(self.len() <= 255);
        s.produce_byte(self.len() as u8);
        for (k,v) in self {
            k.serialize(s);
            v.serialize(s);
        }
    }

}

impl<K:Parsable,V:Parsable> Parsable for (K,V){
    fn parse(p: &mut Parser) -> Self {
        (K::parse(p),V::parse(p))
    }
}


impl<K:Serializable,V:Serializable> Serializable for (K,V){
    fn serialize(&self, s:&mut Serializer) {
        self.0.serialize(s);
        self.1.serialize(s);
    }
}

impl<T:Parsable> Parsable for Option<T> {
    fn parse(p: &mut Parser) -> Self {
       match p.consume_byte() {
           0 => None,
           1 => Some(T::parse(p)),
           _ => panic!("Decoding error: an Options tag must be 0 or 1")
       }
    }
}

impl<T:Serializable> Serializable for Option<T>{
    fn serialize(&self, s:&mut Serializer) {
        match *self {
            None => s.produce_byte(0),
            Some(ref val) => {
                s.produce_byte(1);
                val.serialize(s);
            },
        }
    }
}

impl<T:Parsable> Parsable for Box<T>{
    fn parse(p: &mut Parser) -> Self {
        Box::new(T::parse(p))
    }
}

impl<T:Serializable> Serializable for Box<T>{
    fn serialize(&self, s:&mut Serializer) {
        self.deref().serialize(s);
    }
}

impl<T:Parsable> Parsable for Rc<T>{
    fn parse(p: &mut Parser) -> Self {
        Rc::new(T::parse(p))
    }
}

impl<T:Serializable> Serializable for Rc<T>{
    fn serialize(&self, s:&mut Serializer) {
        self.deref().serialize(s);
    }
}

impl Parsable for u8 {
    fn parse(p: &mut Parser) -> Self {
        p.consume_byte()
    }
}

impl Serializable for u8 {
    fn serialize(&self, s:&mut Serializer) {
        s.produce_byte(*self);
    }
}

impl Parsable for u16 {
    fn parse(p: &mut Parser) -> Self {
        EncodingByteOrder::read_u16(p.consume_bytes(2))
    }
}

impl Serializable for u16 {
    fn serialize(&self, s:&mut Serializer) {
        EncodingByteOrder::write_u16(s.get_buf(2),*self);
    }
}

impl Parsable for u32 {
    fn parse(p: &mut Parser) -> Self {
        EncodingByteOrder::read_u32(p.consume_bytes(4))
    }
}

impl Serializable for u32 {
    fn serialize(&self, s:&mut Serializer) {
        EncodingByteOrder::write_u32(s.get_buf(4),*self);
    }
}

impl Parsable for u64 {
    fn parse(p: &mut Parser) -> Self {
        EncodingByteOrder::read_u64(p.consume_bytes(8))
    }
}

impl Serializable for u64 {
    fn serialize(&self, s:&mut Serializer) {
        EncodingByteOrder::write_u64(s.get_buf(8),*self);
    }
}

impl Parsable for u128 {
    fn parse(p: &mut Parser) -> Self {
        EncodingByteOrder::read_u128(p.consume_bytes(16))
    }
}

impl Serializable for u128 {
    fn serialize(&self, s:&mut Serializer) {
        EncodingByteOrder::write_u128(s.get_buf(16),*self);
    }
}

impl Parsable for i8 {
    fn parse(p: &mut Parser) -> Self {
        p.consume_byte() as i8
    }
}

impl Serializable for i8 {
    fn serialize(&self, s:&mut Serializer) {
        s.produce_byte(*self as u8);
    }
}

impl Parsable for i16 {
    fn parse(p: &mut Parser) -> Self {
        EncodingByteOrder::read_i16(p.consume_bytes(2))
    }
}

impl Serializable for i16 {
    fn serialize(&self, s:&mut Serializer) {
        EncodingByteOrder::write_i16(s.get_buf(2),*self);
    }
}

impl Parsable for i32 {
    fn parse(p: &mut Parser) -> Self {
        EncodingByteOrder::read_i32(p.consume_bytes(4))
    }
}

impl Serializable for i32 {
    fn serialize(&self, s:&mut Serializer) {
        EncodingByteOrder::write_i32(s.get_buf(4),*self);
    }
}

impl Parsable for i64 {
    fn parse(p: &mut Parser) -> Self {
        EncodingByteOrder::read_i64(p.consume_bytes(8))
    }
}

impl Serializable for i64 {
    fn serialize(&self, s:&mut Serializer) {
        EncodingByteOrder::write_i64(s.get_buf(8),*self);
    }
}

impl Parsable for i128 {
    fn parse(p: &mut Parser) -> Self {
        EncodingByteOrder::read_i128(p.consume_bytes(16))
    }
}

impl Serializable for i128 {
    fn serialize(&self, s:&mut Serializer) {
        EncodingByteOrder::write_i128(s.get_buf(16),*self);
    }
}


impl Parsable for bool {
    fn parse(p: &mut Parser) -> Self {
        p.consume_byte() != 0
    }
}

impl Serializable for bool {
    fn serialize(&self, s:&mut Serializer) {
        s.produce_byte(if *self {1} else {0});
    }
}