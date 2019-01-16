use alloc::prelude::*;
use alloc::collections::BTreeSet;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use byteorder::{LittleEndian, ByteOrder};
use errors::*;
use core::ops::Deref;
use model::*;
use arena::*;


pub trait ParserAllocator {
    fn poly_alloc<T:Sized + Copy + VirtualSize>(&self, val: T) -> Result<Ptr<T>>;
    fn poly_slice_builder<T: Sized + Copy + VirtualSize>(&self, len: usize) -> Result<SliceBuilder<T>>;
}

#[derive(Copy, Clone, Debug)]
pub struct NoCustomAlloc();
impl ParserAllocator for NoCustomAlloc {
    fn poly_alloc<T: Sized + Copy + VirtualSize>(&self, _val: T) -> Result<Ptr<T>> { unreachable!() }
    fn poly_slice_builder<T: Sized + Copy + VirtualSize>(&self, _len: usize) -> Result<SliceBuilder<T>> {  unreachable!() }
}

//the state needed during parsing including the input
pub struct Parser<'a>{
    index:usize,
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

    //parse the whole content as a specific Parsable<'a>
    pub fn parse_fully<'b,T:Parsable<'b>, A: ParserAllocator>(data:&'a [u8], alloc:&'b A) -> Result<T>{
        let mut parser = Parser::new(data);
        let parsed = T::parse(&mut parser, alloc)?;
        if parser.data.len() != parser.index {return parsing_terminated()}
        Ok(parsed)
    }


    //fetch a single byte
    pub fn consume_byte(&mut self) -> Result<u8>{
        if self.index+1 > self.data.len() {return parsing_terminated()}
        let res = self.data[self.index];
        self.index+=1;
        Ok(res)
    }

    //fetch a fix amount of bytes
    pub fn consume_bytes(&mut self, amount:usize) -> Result<&[u8]>{
        if self.index+amount > self.data.len() {return parsing_terminated()}
        let res = &self.data[self.index..(self.index+amount)];
        self.index+=amount;
        Ok(res)
    }

}

//trait defining Parsable<'a> types
pub trait Parsable<'a> where Self:Sized {
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self>;
}

//the state during serialization
#[derive(Default)]
pub struct Serializer(Vec<u8>);

impl Serializer{

    //Creates a new serializer
    pub fn new() -> Self {
        Serializer(Vec::new())
    }

    //parse the whole content as a specific Parsable<'a>
    pub fn serialize_fully<S:Serializable>(data:&S,) -> Vec<u8>{
        let mut serializer = Serializer::new();
        data.serialize(&mut serializer);
        serializer.extract()
    }

    //Emits a single byte
    pub fn produce_byte(&mut self, byte:u8){
        self.0.push(byte)
    }

    //emit a fixed number of bytes
    pub fn produce_bytes(&mut self, bytes:&[u8]){
        for b in bytes{
            self.0.push(*b)
        }
    }

    //allocates an outbut buffer and returns a pointer to it
    pub fn get_buf(&mut self, amount:usize) -> &mut [u8]{
        let start = self.0.len();
        for _ in 0..amount {
            self.0.push(0)
        }
        &mut self.0[start..]
    }

    //gets the result
    pub fn extract(self) -> Vec<u8>{
        self.0
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
    [a, b][(a > b) as usize]
}

pub trait VirtualSize {
    const SIZE:usize;
}

//Implementations for non- application specific types

impl<'a,T:Parsable<'a>> Parsable<'a> for Vec<T>{
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        let len = p.consume_byte()?;
        let mut elems = Vec::with_capacity(len as usize);
        for _ in 0..len {
            elems.push(Parsable::parse(p,alloc)?);
        }
        Ok(elems)
    }
}

impl<T:Serializable> Serializable for Vec<T>{
    fn serialize(&self, s:&mut Serializer) {
        assert!(self.len() <= 255);
        s.produce_byte(self.len() as u8);
        for elem in self {
            elem.serialize(s);
        }
    }
}

impl<'a> Parsable<'a> for Hash {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(array_ref!(p.consume_bytes(20)?, 0, 20).to_owned())
    }
}

impl Serializable for Hash {
    fn serialize(&self, s:&mut Serializer) {
        s.produce_bytes(&self[..])
    }
}

impl VirtualSize for Hash {
    const SIZE: usize = 20;
}

impl<'a> Parsable<'a> for [u8;32]{
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(array_ref!(p.consume_bytes(32)?, 0, 32).to_owned())
    }
}

impl Serializable for [u8;32]{
    fn serialize(&self, s:&mut Serializer) {
        s.produce_bytes(&self[..])
    }
}

impl VirtualSize for [u8;32] {
    const SIZE: usize = 32;
}

impl<'a> Parsable<'a> for [u8;64]{
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(array_ref!(p.consume_bytes(64)?, 0, 64).to_owned())
    }
}

impl Serializable for [u8;64]{
    fn serialize(&self, s:&mut Serializer) {
        s.produce_bytes(&self[..])
    }
}

impl VirtualSize for [u8;64] {
    const SIZE: usize = 64;
}

impl<'a,T:Parsable<'a>+Ord> Parsable<'a> for BTreeSet<T>{
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        let len = p.consume_byte()?;
        let mut elems = BTreeSet::new();
        for _ in 0..len {
            elems.insert(T::parse(p,alloc)?);
        }
        Ok(elems)
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

impl<'a, K:Parsable<'a>+Ord,V:Parsable<'a>> Parsable<'a> for BTreeMap<K,V>{
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        let len = p.consume_byte()?;
        let mut elems = BTreeMap::new();
        for _ in 0..len {
            elems.insert(K::parse(p,alloc)?,V::parse(p,alloc)?);
        }
        Ok(elems)
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

impl<'a, K:Parsable<'a>,V:Parsable<'a>> Parsable<'a> for (K,V){
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        Ok((K::parse(p,alloc)?,V::parse(p,alloc)?))
    }
}


impl<K:Serializable,V:Serializable> Serializable for (K,V){
    fn serialize(&self, s:&mut Serializer) {
        self.0.serialize(s);
        self.1.serialize(s);
    }
}

impl<K:VirtualSize,V:VirtualSize> VirtualSize for (K,V){
    const SIZE: usize = K::SIZE + V::SIZE;
}

impl<'a, T:Parsable<'a>> Parsable<'a> for Option<T> {
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
       Ok(match p.consume_byte()? {
           0 => None,
           1 => Some(T::parse(p,alloc)?),
           _ => return parsing_case_failure()
       })
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

impl<'a, T:Parsable<'a>> Parsable<'a> for Box<T>{
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        Ok(Box::new(T::parse(p,alloc)?))
    }
}

impl<T:Serializable> Serializable for Box<T>{
    fn serialize(&self, s:&mut Serializer) {
        self.deref().serialize(s);
    }
}

impl<'a, T:Parsable<'a>> Parsable<'a> for Rc<T>{
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        Ok(Rc::new(T::parse(p,alloc)?))
    }
}

impl<T:Serializable> Serializable for Rc<T>{
    fn serialize(&self, s:&mut Serializer) {
        self.deref().serialize(s);
    }
}

impl<'a> Parsable<'a> for u8 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(p.consume_byte()?)
    }
}

impl Serializable for u8 {
    fn serialize(&self, s:&mut Serializer) {
        s.produce_byte(*self)
    }
}

impl VirtualSize for u8{
    const SIZE: usize = 1;
}

impl<'a> Parsable<'a> for u16 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(LittleEndian::read_u16(p.consume_bytes(2)?))
    }
}

impl Serializable for u16 {
    fn serialize(&self, s:&mut Serializer) {
        LittleEndian::write_u16(s.get_buf(2),*self)
    }
}

impl VirtualSize for u16{
    const SIZE: usize = 2;
}

impl<'a> Parsable<'a> for u32 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(LittleEndian::read_u32(p.consume_bytes(4)?))
    }
}

impl Serializable for u32 {
    fn serialize(&self, s:&mut Serializer) {
        LittleEndian::write_u32(s.get_buf(4),*self)
    }
}

impl VirtualSize for u32{
    const SIZE: usize = 4;
}

impl<'a> Parsable<'a> for u64 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(LittleEndian::read_u64(p.consume_bytes(8)?))
    }
}

impl Serializable for u64 {
    fn serialize(&self, s:&mut Serializer) {
        LittleEndian::write_u64(s.get_buf(8),*self)
    }
}

impl VirtualSize for u64{
    const SIZE: usize = 8;
}

impl<'a> Parsable<'a> for u128 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(LittleEndian::read_u128(p.consume_bytes(16)?))
    }
}

impl Serializable for u128 {
    fn serialize(&self, s:&mut Serializer) {
        LittleEndian::write_u128(s.get_buf(16),*self)
    }
}

impl VirtualSize for u128{
    const SIZE: usize = 16;
}

impl VirtualSize for usize{
    const SIZE: usize = 8;
}

impl<'a> Parsable<'a> for i8 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(p.consume_byte()? as i8)
    }
}

impl Serializable for i8 {
    fn serialize(&self, s:&mut Serializer) {
        s.produce_byte(*self as u8)
    }
}

impl VirtualSize for i8{
    const SIZE: usize = 1;
}

impl<'a> Parsable<'a> for i16 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(LittleEndian::read_i16(p.consume_bytes(2)?))
    }
}

impl Serializable for i16 {
    fn serialize(&self, s:&mut Serializer) {
        LittleEndian::write_i16(s.get_buf(2),*self)
    }
}

impl VirtualSize for i16{
    const SIZE: usize = 2;
}

impl<'a> Parsable<'a> for i32 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(LittleEndian::read_i32(p.consume_bytes(4)?))
    }
}

impl Serializable for i32 {
    fn serialize(&self, s:&mut Serializer) {
        LittleEndian::write_i32(s.get_buf(4),*self)
    }
}

impl VirtualSize for i32{
    const SIZE: usize = 4;
}

impl<'a> Parsable<'a> for i64 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(LittleEndian::read_i64(p.consume_bytes(4)?))
    }
}

impl Serializable for i64 {
    fn serialize(&self, s:&mut Serializer) {
        LittleEndian::write_i64(s.get_buf(4),*self)
    }
}

impl VirtualSize for i64{
    const SIZE: usize = 8;
}

impl<'a> Parsable<'a> for i128 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(LittleEndian::read_i128(p.consume_bytes(4)?))
    }
}

impl Serializable for i128 {
    fn serialize(&self, s:&mut Serializer) {
        LittleEndian::write_i128(s.get_buf(4),*self)
    }
}

impl VirtualSize for i128{
    const SIZE: usize = 16;
}

impl VirtualSize for isize{
    const SIZE: usize = 8;
}

impl<'a> Parsable<'a> for bool {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(p.consume_byte()? != 0)
    }
}

impl Serializable for bool {
    fn serialize(&self, s:&mut Serializer) {
        s.produce_byte(if *self {1} else {0})
    }
}

impl VirtualSize for bool{
    const SIZE: usize = 1;
}

impl<'a, T:Parsable<'a>+ Copy + VirtualSize> Parsable<'a> for Ptr<'a, T> {
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        let res = {T::parse(p,alloc)?};
        let ret = alloc.poly_alloc(res);
        ret
    }
}

impl<'a, T:Serializable> Serializable for Ptr<'a, T> {
    fn serialize(&self, s: &mut Serializer) {
        self.0.serialize(s)
    }
}

impl<'a, T> VirtualSize for Ptr<'a, T> {
    const SIZE: usize = 8;
}

impl<'a, T:Parsable<'a> + Copy + VirtualSize> Parsable<'a> for SlicePtr<'a, T>  {
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        let len = u16::parse(p,alloc)?;
        let mut builder = alloc.poly_slice_builder(len as usize)?;
        for _ in 0..len {
            builder.push(T::parse(p,alloc)?);
        }
        Ok(builder.finish())
    }
}

impl<'a, T:Serializable> Serializable for SlicePtr<'a, T> {
    fn serialize(&self, s: &mut Serializer) {
        (self.0.len() as u16).serialize(s);
        for v in self.0 {
            v.serialize(s);
        }
    }
}

impl<'a, T> VirtualSize for SlicePtr<'a, T> {
    const SIZE: usize = 16;
}