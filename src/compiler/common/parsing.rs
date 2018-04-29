use compiler::common::macros::view::*;
use compiler::errors::parsing::*;
use compiler::common::types::*;

use constant_time_eq::constant_time_eq;
use byteorder::ReadBytesExt;
use blake2_rfc::blake2b::Blake2bResult;

use byteorder::NetworkEndian;

pub type Endian = NetworkEndian;

impl<'a> Deserializer<'a> for u8 {
    fn from_bytes(d: &[u8]) -> Result<Self, ParsingError> {
        Result::Ok(d[0])
    }
}

impl<'a> Deserializer<'a> for Tag {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(Tag(d[0]))
    }
}

impl<'a> Deserializer<'a> for Flag {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(Flag(d[0] == 1))
    }
}


impl<'a> Deserializer<'a> for Version {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(Version(d[0]))
    }
}

impl<'a> Deserializer<'a> for MemberIndex {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(MemberIndex(d[0]))
    }
}

impl<'a> Deserializer<'a> for Field {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        let typ = TypeId::from_bytes(&d[0..1])?;
        let control = Control::from_bytes(&d[1..2])?;
        Ok(Field(control,typ))
    }
}

impl<'a> Deserializer<'a> for Ptr {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(Ptr((&d[..2]).read_u16::<Endian>().map_err(|e| ParsingError::IOError(e))?))
    }
}

impl<'a> Deserializer<'a> for Length {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(Length((&d[..2]).read_u16::<Endian>().map_err(|e| ParsingError::IOError(e))?))
    }
}

impl<'a> Deserializer<'a> for Coefficient {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(Coefficient((&d[..2]).read_u16::<Endian>().map_err(|e| ParsingError::IOError(e))?))
    }
}

impl<'a> Deserializer<'a> for ValueId {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(ValueId((&d[..2]).read_u16::<Endian>().map_err(|e| ParsingError::IOError(e))?))
    }
}

impl<'a> Deserializer<'a> for TypeId {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(TypeId(d[0]))
    }
}

impl<'a> Deserializer<'a> for ModuleId {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(ModuleId(d[0]))
    }
}

impl<'a> Deserializer<'a> for FunId {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(FunId(d[0]))
    }
}


impl<'a> Deserializer<'a> for InitId {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(InitId(d[0]))
    }
}

impl<'a> Deserializer<'a> for CtrId {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(CtrId(d[0]))
    }
}

impl<'a> Deserializer<'a> for ParamId {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(ParamId(d[0]))
    }
}

impl<'a> Deserializer<'a> for Amount {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        Ok(Amount(d[0]))
    }
}

impl<'a> Deserializer<'a> for Privileges {

    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        let val =  (&d[..2]).read_u16::<Endian>().map_err(|e| ParsingError::IOError(e))?;
        if val > ALL_PRIVILEGES {
            Result::Err(ParsingError::WrongPrivilegesEncoding {provided:val,max:ALL_PRIVILEGES})
        } else {
            Ok(Privileges(val))
        }
    }
}

impl<'a> Deserializer<'a> for Control {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        match d[0] {
            0 => Result::Ok(Control::Ref),
            1 => Result::Ok(Control::Owned),
            2 => Result::Ok(Control::Borrowed),
            3 => Result::Ok(Control::UnusedRef),
            4 => Result::Ok(Control::UnusedOwned),
            5 => Result::Ok(Control::UnusedBorrowed),
            x => Result::Err(ParsingError::WrongEnumEncoding {provided:x,max:5,enum_name:"Control"})
        }
    }

}

impl<'a> Deserializer<'a> for Bound {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        match d[0] {
            0 => Result::Ok(Bound::Phantom),
            1 => Result::Ok(Bound::Dynamic),
            x => Result::Err(ParsingError::WrongEnumEncoding {provided:x,max:1,enum_name:"Bound"})
        }
    }

}

impl<'a> Deserializer<'a> for Visibility {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        match d[0] {
            0 => Result::Ok(Visibility::Public),
            1 => Result::Ok(Visibility::Private),
            x => Result::Err(ParsingError::WrongEnumEncoding {provided:x,max:1,enum_name:"Visibility"})
        }
    }
}

impl<'a> Deserializer<'a> for TypeKind {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        match d[0] {
            0 => Result::Ok(TypeKind::Cell),
            1 => Result::Ok(TypeKind::View),
            2 => Result::Ok(TypeKind::Normal),
            x => Result::Err(ParsingError::WrongEnumEncoding {provided:x,max:2,enum_name:"TypeKind"})
        }
    }
}

impl<'a> Deserializer<'a> for ExecutionMode {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        match d[0] {
            0 => Result::Ok(ExecutionMode::Pure),
            1 => Result::Ok(ExecutionMode::Init),
            2 => Result::Ok(ExecutionMode::Dependent),
            3 => Result::Ok(ExecutionMode::Active),
            x => Result::Err(ParsingError::WrongEnumEncoding {provided:x,max:3,enum_name:"ExecutionMode"})
        }
    }
}

impl<'a> Deserializer<'a> for OptimizationDeclaration {
    fn from_bytes(d:&[u8]) -> Result<Self,ParsingError> {
        match d[0] {
            0 => Result::Ok(OptimizationDeclaration::Empty),
            1 => Result::Ok(OptimizationDeclaration::Wrapper),
            2 => Result::Ok(OptimizationDeclaration::Normal),
            x => Result::Err(ParsingError::WrongEnumEncoding {provided:x,max:2,enum_name:"OptimizationDeclaration"})
        }
    }
}


impl<'a> Deserializer<'a> for Hash<'a>{
    fn from_bytes(d: &'a [u8]) -> Result<Self, ParsingError> {
        if d.len() != HASH_SIZE {return Result::Err(ParsingError::WrongHashSize { expected: HASH_SIZE, actual: d.len() })}
        Result::Ok(Hash{d})
    }
}

impl<'a> Hash<'a> {
    pub fn from_blake(res:&'a Blake2bResult) -> Result<Self, ParsingError>{
        if res.len() != HASH_SIZE {return Result::Err(ParsingError::WrongHashSize { expected: HASH_SIZE, actual: res.len() })}
        Result::Ok(Hash{d:res.as_bytes()})
    }

    pub fn extract_value(&self) -> [u8;HASH_SIZE]{
        let mut res: [u8;HASH_SIZE] = [0;HASH_SIZE];
        res[..].copy_from_slice(&self.d[..]);
        res
    }

    pub fn borrow_value(&self) -> &'a [u8]{
        self.d
    }
}

impl<'a> Eq for Hash<'a> {}
impl<'a> PartialEq for Hash<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq(self.d, other.d)
    }
}