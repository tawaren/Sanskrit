use crate::model::{ValueSchema, Entry, Adt};
use sanskrit_common::encoding::{Serializer, Serializable, ParserAllocator, Parser, Parsable, VirtualSize};
use sanskrit_common::errors::*;
use sanskrit_common::model::SlicePtr;
use sanskrit_common::arena::VirtualHeapArena;


pub fn parse_with_schema<'c, 'h, 'b>(schema:ValueSchema<'c>, p:&'c mut Parser<'c>, alloc:&'b VirtualHeapArena<'h>) -> Result<Entry<'b>>{
    schema.parse_value(p,alloc)
}

impl<'a> ValueSchema<'a> {

    pub fn serialize_value(&self, value:Entry, s:&mut Serializer) -> Result<()> {
        match *self {
            ValueSchema::Adt(_, ctrs) => {
                let Adt(tag, fields) = unsafe {value.adt};
                //if their is only 1 tag we omit the tag
                if ctrs.len() != 1 {
                    assert!((tag as usize) < ctrs.len());
                    tag.serialize(s)?;
                }
                //if their are zero fields we omit the fields
                let ctr =  ctrs[tag as usize];
                assert_eq!(fields.len(), ctr.len());
                s.increment_depth()?;
                for (f_value, (_,f_schema)) in fields.iter().zip(ctr.iter()) {
                    f_schema.serialize_value(*f_value, s)?;
                }
                s.decrement_depth();
                Ok(())
            },
            ValueSchema::Data(size) => {
                assert_eq!(size as usize, unsafe {value.data}.len());
                Ok(s.produce_bytes(&unsafe {value.data}))
            },
            ValueSchema::Unsigned(1) => unsafe {value.u8}.serialize(s),
            ValueSchema::Unsigned(2) => unsafe {value.u16}.serialize(s),
            ValueSchema::Unsigned(4) => unsafe {value.u32}.serialize(s),
            ValueSchema::Unsigned(8) => unsafe {value.u64}.serialize(s),
            ValueSchema::Unsigned(16) => unsafe {value.u128}.serialize(s),
            ValueSchema::Signed(1) => unsafe {value.i8}.serialize(s),
            ValueSchema::Signed(2) => unsafe {value.i16}.serialize(s),
            ValueSchema::Signed(4) => unsafe {value.i32}.serialize(s),
            ValueSchema::Signed(8) => unsafe {value.i64}.serialize(s),
            ValueSchema::Signed(16) => unsafe {value.i128}.serialize(s),
            _ => unreachable!()
        }
    }

    //todo: add checks and return Err if input mismatches (needed if literals are parsed)
    //todo: do we need to enforce structural depth here?
    pub fn parse_value<'c, 'd, 'b, A: ParserAllocator>(self, p:&'c mut Parser<'d>, alloc:&'b A) -> Result<Entry<'b>> {
        Ok(match self {
            ValueSchema::Adt(_, ctrs) => {
                //if their is only 1 tag it was omitted
                let tag = if ctrs.len() != 1 {
                    u8::parse(p,alloc)?
                } else {
                    0
                };

                if tag as usize >= ctrs.len() {
                    return error(||"Tag of parsed value is invalid")
                }
                //if their are zero fields we use the EmptySlice shortcut
                let ctr =  ctrs[tag as usize];
                let fields = if !ctr.is_empty() {
                    let mut builder = alloc.poly_slice_builder(ctr.len())?;
                    p.increment_depth()?;
                    for  (_,f_schema) in ctr.iter(){
                        builder.push(f_schema.parse_value(p, alloc)?);
                    }
                    p.decrement_depth();
                    builder.finish()
                } else {
                    SlicePtr::empty()
                };
                Entry{ adt:Adt(tag, fields) }
            },
            ValueSchema::Data(size) => {
                let mut builder = alloc.poly_slice_builder(size as usize)?;
                for _ in 0..size{
                    builder.push(u8::parse(p, alloc)?);
                }
                Entry{ data:builder.finish()}
            },
            ValueSchema::Unsigned(1) => Entry{u8: u8::parse(p, alloc)?},
            ValueSchema::Unsigned(2) => Entry{u16: u16::parse(p, alloc)?},
            ValueSchema::Unsigned(4) => Entry{u32: u32::parse(p, alloc)?},
            ValueSchema::Unsigned(8) => Entry{u64: u64::parse(p, alloc)?},
            ValueSchema::Unsigned(16) => Entry{u128: u128::parse(p, alloc)?},
            ValueSchema::Signed(1) => Entry{i8: i8::parse(p, alloc)?},
            ValueSchema::Signed(2) => Entry{i16: i16::parse(p, alloc)?},
            ValueSchema::Signed(4) => Entry{i32: i32::parse(p, alloc)?},
            ValueSchema::Signed(8) => Entry{i64: i64::parse(p, alloc)?},
            ValueSchema::Signed(16) => Entry{i128: i128::parse(p, alloc)?},
            _ => unreachable!()
        })
    }

    pub fn move_value<'b,'h>(&self, value:Entry, alloc:&'b VirtualHeapArena<'h>) -> Result<Entry<'b>> {
        match *self {
            ValueSchema::Adt(_, ctrs) => {
                let Adt(tag, fields) = unsafe {value.adt};
                let ctr =  ctrs[tag as usize];
                assert_eq!(fields.len(), ctr.len());
                let mut builder =alloc.slice_builder(fields.len())?;
                for (f_value,  (_,f_schema)) in fields.iter().zip(ctr.iter()) {
                    builder.push(f_schema.move_value(*f_value, alloc)?);
                }
                Ok(Entry{adt:Adt(tag, builder.finish())})
            },
            ValueSchema::Data(size) => {
                assert_eq!(size as usize, unsafe {value.data}.len());
                let data = alloc.copy_alloc_slice(&unsafe {value.data})?;
                Ok(Entry{data})
            },
            ValueSchema::Unsigned(1) => Ok(Entry{u8:unsafe {value.u8}}),
            ValueSchema::Unsigned(2) => Ok(Entry{u16:unsafe {value.u16}}),
            ValueSchema::Unsigned(4) => Ok(Entry{u32:unsafe {value.u32}}),
            ValueSchema::Unsigned(8) => Ok(Entry{u64:unsafe {value.u64}}),
            ValueSchema::Unsigned(16) => Ok(Entry{u128:unsafe {value.u128}}),
            ValueSchema::Signed(1) => Ok(Entry{i8:unsafe {value.i8}}),
            ValueSchema::Signed(2) => Ok(Entry{i16:unsafe {value.i16}}),
            ValueSchema::Signed(4) => Ok(Entry{i32:unsafe {value.i32}}),
            ValueSchema::Signed(8) => Ok(Entry{i64:unsafe {value.i64}}),
            ValueSchema::Signed(16) => Ok(Entry{i128:unsafe {value.i128}}),
            _ => unreachable!()
        }
    }

    //note: intended as helper for tools and compilers it is not save to execute during block computation
    pub fn runtime_size(self, data:&[u8]) -> Result<u16> {
        let (res,_) = self.runtime_size_intern(data,0)?;
        Ok(res)
    }

    fn runtime_size_intern(self, data:&[u8], mut pos:usize) -> Result<(u16,usize)> {
        let res = match self {
            ValueSchema::Adt(_, ctrs) => {
                //if their is only 1 tag it was omitted
                let tag = if ctrs.len() != 1 {
                    data[pos]
                } else {
                    0
                };
                pos+=1;
                if tag as usize >= ctrs.len() {
                    return error(||"Tag of parsed value is invalid")
                }
                //if their are zero fields we use the EmptySlice shortcut
                let ctr =  ctrs[tag as usize];
                let mut field_content_sizes = 0;
                if !ctr.is_empty() {
                    for  (_,f_schema) in ctr.iter(){
                        let (size, new_pos) = f_schema.runtime_size_intern(data, pos)?;
                        pos = new_pos;
                        field_content_sizes += size
                    }
                }
                (Entry::SIZE+(field_content_sizes as usize),pos)
            },
            ValueSchema::Data(size) => (Entry::SIZE+(size as usize),pos+(size as usize)),
            ValueSchema::Unsigned(size) | ValueSchema::Signed(size) => (Entry::SIZE, pos+(size as usize))
        };
        if res.0 > (u16::max_value() as usize) {
            error(||"encoding size to big")
        } else {
            Ok((res.0 as u16, res.1))
        }
    }

    pub fn max_runtime_size(self) -> Result<u16> {
        let res = match self {
            ValueSchema::Adt(_, ctrs) => {
                let mut max_field_content_sizes = 0;
                for ctr in ctrs.iter() {
                    let mut field_content_sizes = 0;
                    if !ctr.is_empty() {
                        for  (_,f_schema) in ctr.iter(){
                            field_content_sizes += f_schema.max_runtime_size()? as usize;
                        }
                    }
                    if field_content_sizes > max_field_content_sizes {
                        max_field_content_sizes = field_content_sizes;
                    }
                }
                Entry::SIZE+max_field_content_sizes
            },
            ValueSchema::Data(size) => Entry::SIZE+(size as usize),
            ValueSchema::Unsigned(_) | ValueSchema::Signed(_) => Entry::SIZE
        };
        if res > (u16::max_value() as usize) {
            error(||"encoding size to big")
        } else {
            Ok(res as u16)
        }
    }

    pub fn max_serialized_size(&self) -> u16 {
        match *self {
            ValueSchema::Adt(_, ctrs) => {
                let mut max_ctr_size = 0;
                for ctr in ctrs.iter() {
                    let mut ctr_size = 0;
                    for  (_,f_schema) in ctr.iter() {
                        ctr_size += f_schema.max_serialized_size();
                    }
                    if ctr_size > max_ctr_size {
                        max_ctr_size = ctr_size;
                    }
                }

                //if their is only 1 tag we omit the tag else it uses 1 Byte
                if ctrs.len() != 1 {
                    max_ctr_size+1
                } else {
                    max_ctr_size
                }
            },
            ValueSchema::Data(size) => size,
            ValueSchema::Unsigned(size) | ValueSchema::Signed(size) => size as u16,
        }
    }
}