use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::collections::BTreeSet;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use byteorder::{BigEndian, ByteOrder};
use errors::*;
use core::ops::Deref;
use model::*;
use arena::*;

pub type EncodingByteOrder = BigEndian;

pub trait ParserAllocator {
    fn allocated_virtual_bytes(&self) -> usize;
    fn poly_alloc<T:Sized + Copy + VirtualSize>(&self, val: T) -> Result<Ptr<T>>;
    fn poly_slice_builder<T: Sized + Copy + VirtualSize>(&self, len: usize) -> Result<SliceBuilder<T>>;
}

#[derive(Copy, Clone, Debug)]
pub struct NoCustomAlloc();
impl ParserAllocator for NoCustomAlloc {
    fn allocated_virtual_bytes(&self) -> usize {
        0
    }

    fn poly_alloc<T: Sized + Copy + VirtualSize>(&self, _val: T) -> Result<Ptr<T>> { unreachable!() }
    fn poly_slice_builder<T: Sized + Copy + VirtualSize>(&self, _len: usize) -> Result<SliceBuilder<T>> {  unreachable!() }
}

//the state needed during parsing including the input
pub struct Parser<'a>{
    pub index:usize,
    max_depth:usize,
    data:&'a [u8]
}

impl<'a> Parser<'a> {
    //create a new parser for an input
    pub fn new(data:&'a [u8], max_depth:usize) -> Self {
        Parser {
            index: 0,
            max_depth,
            data,
        }
    }

    //parse the whole content as a specific Parsable<'a>
    pub fn parse_fully<'b,T:Parsable<'b>, A: ParserAllocator>(data:&'a [u8], max_depth:usize, alloc:&'b A) -> Result<T>{
        let mut parser = Parser::new(data,max_depth);
        let parsed = T::parse(&mut parser, alloc)?;
        if parser.data.len() != parser.index {
            let res = format!("Decoding error: input data has wrong size. it has {} - consumed {}", parser.data.len(), parser.index );
            return error(||&res)
        }
        Ok(parsed)
    }

    pub fn increment_depth(&mut self) -> Result<()> {
        self.max_depth -=1;
        if self.max_depth == 0 {
            return error(||"Allowed structural dept exceeded during decoding")
        }
        Ok(())
    }

    pub fn decrement_depth(&mut self) {
        self.max_depth +=1
    }

    //fetch a single byte
    pub fn consume_byte(&mut self) -> Result<u8>{
        if self.index+1 > self.data.len() {
            return error(||"Decoding error: input data has not enough bytes")
        }
        let res = self.data[self.index];
        self.index+=1;
        Ok(res)
    }

    //fetch a fix amount of bytes
    pub fn consume_bytes(&mut self, amount:usize) -> Result<&[u8]>{
        if self.index+amount > self.data.len() {
            return error(||"Decoding error: input data has not enough bytes")
        }
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
//todo: can we directly serialize to store??
#[derive(Default)]
pub struct Serializer{
    out:Vec<u8>,
    max_depth:usize,
}

impl Serializer{

    //Creates a new serializer
    pub fn new(max_depth:usize) -> Self {
        Serializer{
            out:Vec::new(),
            max_depth
        }
    }

    //parse the whole content as a specific Parsable<'a>
    pub fn serialize_fully<S:Serializable>(data:&S,max_depth:usize) -> Result<Vec<u8>>{
        let mut serializer = Serializer::new(max_depth);
        data.serialize(&mut serializer)?;
        Ok(serializer.extract())
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

    pub fn increment_depth(&mut self) -> Result<()> {
        self.max_depth -=1;
        if self.max_depth == 0 {
            return error(||"Allowed structural dept exceeded during encoding")
        }
        Ok(())
    }

    pub fn decrement_depth(&mut self) {
        self.max_depth +=1
    }

    //gets the result
    pub fn extract(self) -> Vec<u8>{
        self.out
    }
}

//A trait for Serializable types
pub trait Serializable {
    fn serialize(&self, s:&mut Serializer) -> Result<()>;
}

//helps with turo fish representation ambiguity allows TypeId::<orignalType>::SIZE
// where Original type is in X<A> Notation instead of X::<A>
pub type TypeId<T> = T;
pub const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}

pub trait VirtualSize {
    const SIZE:usize;
}

//Implementations for non- application specific types

impl<'a,T:Parsable<'a>> Parsable<'a> for Vec<T>{
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        let len = p.consume_byte()?;
        let mut elems = Vec::with_capacity(len as usize);
        p.increment_depth()?;
        for _ in 0..len {
            elems.push(Parsable::parse(p,alloc)?);
        }
        p.decrement_depth();
        Ok(elems)
    }
}

impl<T:Serializable> Serializable for Vec<T>{
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        assert!(self.len() <= u8::max_value() as usize);
        s.produce_byte(self.len() as u8);
        s.increment_depth()?;
        for elem in self {
            elem.serialize(s)?;
        }
        s.decrement_depth();
        Ok(())
    }
}

impl<T> Deref for LargeVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a,T:Parsable<'a>> Parsable<'a> for LargeVec<T>{
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        let len = u16::parse(p,alloc)?;
        let mut elems = Vec::with_capacity(len as usize);
        p.increment_depth()?;
        for _ in 0..len {
            elems.push(Parsable::parse(p,alloc)?);
        }
        p.decrement_depth();
        Ok(LargeVec(elems))
    }
}

impl<T:Serializable> Serializable for LargeVec<T>{
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        assert!(self.0.len() <= u16::max_value() as usize);
        (self.0.len() as u16).serialize(s)?;
        s.increment_depth()?;
        for elem in &self.0 {
            elem.serialize(s)?;
        }
        s.decrement_depth();
        Ok(())
    }
}

impl<'a> Parsable<'a> for Hash {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(hash_from_slice(p.consume_bytes(20)?))
    }
}

impl Serializable for Hash {
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        s.produce_bytes(&self[..]);
        Ok(())
    }
}

impl VirtualSize for Hash {
    const SIZE: usize = HASH_SIZE;
}

impl<'a> Parsable<'a> for [u8;32]{
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(array_ref!(p.consume_bytes(32)?, 0, 32).to_owned())
    }
}

impl Serializable for [u8;32]{
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        s.produce_bytes(&self[..]);
        Ok(())
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
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        s.produce_bytes(&self[..]);
        Ok(())
    }
}

impl VirtualSize for [u8;64] {
    const SIZE: usize = 64;
}

impl<'a,T:Parsable<'a>+Ord> Parsable<'a> for BTreeSet<T>{
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        let len = p.consume_byte()?;
        let mut elems = BTreeSet::new();
        p.increment_depth()?;
        for _ in 0..len {
            elems.insert(T::parse(p,alloc)?);
        }
        p.decrement_depth();
        Ok(elems)
    }
}

impl<T:Serializable+Ord> Serializable for BTreeSet<T>{
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        assert!(self.len() <= 255);
        s.produce_byte(self.len() as u8);
        s.increment_depth()?;
        for elem in self {
            elem.serialize(s)?;
        }
        s.decrement_depth();
        Ok(())
    }
}

impl<'a, K:Parsable<'a>+Ord,V:Parsable<'a>> Parsable<'a> for BTreeMap<K,V>{
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        let len = p.consume_byte()?;
        let mut elems = BTreeMap::new();
        p.increment_depth()?;
        for _ in 0..len {
            elems.insert(K::parse(p,alloc)?,V::parse(p,alloc)?);
        }
        p.decrement_depth();
        Ok(elems)
    }
}


impl<K:Serializable+Ord,V:Serializable> Serializable for BTreeMap<K,V>{

    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        assert!(self.len() <= 255);
        s.produce_byte(self.len() as u8);
        s.increment_depth()?;
        for (k,v) in self {
            k.serialize(s)?;
            v.serialize(s)?;
        }
        s.decrement_depth();
        Ok(())
    }

}

impl<'a, K:Parsable<'a>,V:Parsable<'a>> Parsable<'a> for (K,V){
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        p.increment_depth()?;
        let res = (K::parse(p,alloc)?,V::parse(p,alloc)?);
        p.decrement_depth();
        Ok(res)
    }
}


impl<K:Serializable,V:Serializable> Serializable for (K,V){
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        s.increment_depth()?;
        self.0.serialize(s)?;
        self.1.serialize(s)?;
        s.decrement_depth();
        Ok(())
    }
}

impl<K:VirtualSize,V:VirtualSize> VirtualSize for (K,V){
    const SIZE: usize = K::SIZE + V::SIZE;
}

impl<K:VirtualSize> VirtualSize for Option<K>{
    const SIZE: usize = 1 + K::SIZE;
}

impl<'a, T:Parsable<'a>> Parsable<'a> for Option<T> {
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
       Ok(match p.consume_byte()? {
           0 => None,
           1 => {
               p.increment_depth()?;
               let res = Some(T::parse(p,alloc)?);
               p.decrement_depth();
               res
           },
           _ => return error(||"Decoding error: an Options tag must be 0 or 1")

       })
    }
}

impl<T:Serializable> Serializable for Option<T>{
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        match *self {
            None => s.produce_byte(0),
            Some(ref val) => {
                s.produce_byte(1);
                s.increment_depth()?;
                val.serialize(s)?;
                s.decrement_depth()
            },
        }
        Ok(())
    }
}

impl<'a, T:Parsable<'a>> Parsable<'a> for Box<T>{
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        p.increment_depth()?;
        let res = Box::new(T::parse(p,alloc)?);
        p.decrement_depth();
        Ok(res)
    }
}

impl<T:Serializable> Serializable for Box<T>{
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        s.increment_depth()?;
        self.deref().serialize(s)?;
        s.decrement_depth();
        Ok(())
    }
}

impl<'a, T:Parsable<'a>> Parsable<'a> for Rc<T>{
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        p.increment_depth()?;
        let res = Rc::new(T::parse(p,alloc)?);
        p.decrement_depth();
        Ok(res)
    }
}

impl<T:Serializable> Serializable for Rc<T>{
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        s.increment_depth()?;
        self.deref().serialize(s)?;
        s.decrement_depth();
        Ok(())
    }
}

impl<'a> Parsable<'a> for u8 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(p.consume_byte()?)
    }
}

impl Serializable for u8 {
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        s.produce_byte(*self);
        Ok(())
    }
}

impl VirtualSize for u8{
    const SIZE: usize = 1;
}

impl<'a> Parsable<'a> for u16 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(EncodingByteOrder::read_u16(p.consume_bytes(2)?))
    }
}

impl Serializable for u16 {
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        EncodingByteOrder::write_u16(s.get_buf(2),*self);
        Ok(())
    }
}

impl VirtualSize for u16{
    const SIZE: usize = 2;
}

impl<'a> Parsable<'a> for u32 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(EncodingByteOrder::read_u32(p.consume_bytes(4)?))
    }
}

impl Serializable for u32 {
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        EncodingByteOrder::write_u32(s.get_buf(4),*self);
        Ok(())
    }
}

impl VirtualSize for u32{
    const SIZE: usize = 4;
}

impl<'a> Parsable<'a> for u64 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(EncodingByteOrder::read_u64(p.consume_bytes(8)?))
    }
}

impl Serializable for u64 {
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        EncodingByteOrder::write_u64(s.get_buf(8),*self);
        Ok(())
    }
}

impl VirtualSize for u64{
    const SIZE: usize = 8;
}

impl<'a> Parsable<'a> for u128 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(EncodingByteOrder::read_u128(p.consume_bytes(16)?))
    }
}

impl Serializable for u128 {
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        EncodingByteOrder::write_u128(s.get_buf(16),*self);
        Ok(())
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
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        s.produce_byte(*self as u8);
        Ok(())
    }
}

impl VirtualSize for i8{
    const SIZE: usize = 1;
}

impl<'a> Parsable<'a> for i16 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(EncodingByteOrder::read_i16(p.consume_bytes(2)?))
    }
}

impl Serializable for i16 {
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        EncodingByteOrder::write_i16(s.get_buf(2),*self);
        Ok(())
    }
}

impl VirtualSize for i16{
    const SIZE: usize = 2;
}

impl<'a> Parsable<'a> for i32 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(EncodingByteOrder::read_i32(p.consume_bytes(4)?))
    }
}

impl Serializable for i32 {
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        EncodingByteOrder::write_i32(s.get_buf(4),*self);
        Ok(())
    }
}

impl VirtualSize for i32{
    const SIZE: usize = 4;
}

impl<'a> Parsable<'a> for i64 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(EncodingByteOrder::read_i64(p.consume_bytes(8)?))
    }
}

impl Serializable for i64 {
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        EncodingByteOrder::write_i64(s.get_buf(8),*self);
        Ok(())
    }
}

impl VirtualSize for i64{
    const SIZE: usize = 8;
}

impl<'a> Parsable<'a> for i128 {
    fn parse<A: ParserAllocator>(p: &mut Parser, _alloc:&'a A) -> Result<Self> {
        Ok(EncodingByteOrder::read_i128(p.consume_bytes(16)?))
    }
}

impl Serializable for i128 {
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        EncodingByteOrder::write_i128(s.get_buf(16),*self);
        Ok(())
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
    fn serialize(&self, s:&mut Serializer) -> Result<()> {
        s.produce_byte(if *self {1} else {0});
        Ok(())
    }
}

impl VirtualSize for bool{
    const SIZE: usize = 1;
}

impl<'a, T:Parsable<'a>+ Copy + VirtualSize> Parsable<'a> for Ptr<'a, T> {
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        p.increment_depth()?;
        let res = {T::parse(p,alloc)?};
        p.decrement_depth();
        alloc.poly_alloc(res)
    }
}

impl<'a, T:Serializable> Serializable for Ptr<'a, T> {
    fn serialize(&self, s: &mut Serializer) -> Result<()> {
        s.increment_depth()?;
        self.0.serialize(s)?;
        s.decrement_depth();
        Ok(())
    }
}

impl<'a, T> VirtualSize for Ptr<'a, T> {
    const SIZE: usize = 8;
}

impl<'a, T:Parsable<'a> + Copy + VirtualSize> Parsable<'a> for SlicePtr<'a, T>  {
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc:&'a A) -> Result<Self> {
        let len = u16::parse(p,alloc)?;
        let mut builder = alloc.poly_slice_builder(len as usize)?;
        p.increment_depth()?;
        for _ in 0..len {
            builder.push(T::parse(p,alloc)?);
        }
        p.decrement_depth();
        Ok(builder.finish())
    }
}

impl<'a, T:Serializable+Copy> Serializable for SlicePtr<'a, T> {
    fn serialize(&self, s: &mut Serializer) -> Result<()>{
        self.0.serialize(s)?;
        s.increment_depth()?;
        for v in self.iter() {
            v.serialize(s)?;
        }
        s.decrement_depth();
        Ok(())
    }
}

impl<'a, T> VirtualSize for SlicePtr<'a, T> {
    const SIZE: usize = 10;
}
