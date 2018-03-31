use byteorder::NetworkEndian ;
use compiler::errors::parsing::*;

pub type Endian = NetworkEndian;

pub trait Deserializer<'a> where Self:Sized {
    fn from_bytes(d:&'a [u8]) -> Result<Self,ParsingError>;
}

macro_rules! field_accessor {
    ($field:expr, $name:ident, $typ:ty, $ctr:ident) => {
        pub fn $name(&self) -> Result<$typ,ParsingError> {
            $ctr::from_bytes(&self.data[$field.start..($field.start + $field.len)])
        }
    }
}

macro_rules! embedded_field_accessor {
    ($field:expr, $name:ident, $typ:ty, $ctr:ident) => {
        pub fn $name(&self) -> Result<$typ,ParsingError> {
            let start = self.start + $field.start;
            $ctr::from_bytes(&self.data[start..(start + $field.len)])
        }
    }
}


macro_rules! view_pointer_impl {
    ($size:expr, $name:ident, $typ:ty, $ctr:ident) => {
        #[derive(Copy, Clone)]
        pub struct $name<'a> {
            data: &'a [u8],
            start:usize,
            len:u8
        }

        impl<'a> $name<'a> {
            pub fn start(&self) -> usize {
                self.start
            }

            pub fn after(&self) -> usize {
                self.start + ($size * (self.len as usize))
            }

            pub fn create(data: &'a [u8], start:usize, len:u8) -> Self {
                $name { data, start, len}
            }

            pub fn get(&self, elem:usize) -> Result<$typ,ParsingError>{
                if elem >= self.len as usize {return Err(ParsingError::ViewIndexAccessError{requested:elem, len:self.len as usize})}
                let index = self.start + ($size  * elem);
                $ctr::from_bytes(&self.data[index..(index+$size)])
            }

            pub fn len(&self) -> usize {
                self.len as usize
            }
        }
    }
}

macro_rules! view_index_pointer_impl {
    ($name:ident, $typ:ty, $ctr:ident) => {
        #[derive(Copy, Clone)]
        pub struct $name<'a> {
            data: &'a [u8],
            start:usize,
            len:u8
        }

        impl<'a> $name<'a> {

            pub fn create(data: &'a [u8], start:usize, len:u8) -> Self {
                $name { data, start, len}
            }

            pub fn start(&self) -> usize {
                self.start
            }

            pub fn after(&self) -> usize {
                self.start + (PTR_SIZE * (self.len as usize))
            }

            pub fn get(&self, elem:usize) -> Result<$typ,ParsingError>{
                if elem >= self.len() {return Result::Err(ParsingError::ViewIndexAccessError{requested:elem as usize, len:self.len as usize})}
                let index = self.start + (PTR_SIZE * (elem as usize));
                let Ptr(start) = Ptr::from_bytes(&self.data[index..(index+PTR_SIZE)])?;
                Result::Ok($ctr::create(self.data,start as usize))
            }

            pub fn len(&self) -> usize {
                self.len as usize
            }
        }
    }
}