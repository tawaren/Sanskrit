
use byteorder::{WriteBytesExt};
use hex::decode;
use model::Type;
use model::Ref;
use sanskrit_interpreter::model::LitDesc;
use sanskrit_common::encoding::EncodingByteOrder;

pub fn parse_lit(input:&str, typ:&Type) -> Vec<u8>{
    fn into_vec<F:FnOnce(&mut Vec<u8>)>(f:F)-> Vec<u8>{
        let mut res = Vec::new();
        f(&mut res);
        res
    }

    match &typ.main {
        Ref::Native(ref id) => match id.0.as_ref() {
            "u8" => into_vec(|res|res.write_u8(input.parse::<u8>().unwrap()).unwrap()),
            "i8" => into_vec(|res|res.write_i8(input.parse::<i8>().unwrap()).unwrap()),
            "u16" => into_vec(|res|res.write_u16::<EncodingByteOrder>(input.parse::<u16>().unwrap()).unwrap()),
            "i16" => into_vec(|res|res.write_i16::<EncodingByteOrder>(input.parse::<i16>().unwrap()).unwrap()),
            "u32" => into_vec(|res|res.write_u32::<EncodingByteOrder>(input.parse::<u32>().unwrap()).unwrap()),
            "i32" => into_vec(|res|res.write_i32::<EncodingByteOrder>(input.parse::<i32>().unwrap()).unwrap()),
            "u64" => into_vec(|res|res.write_u64::<EncodingByteOrder>(input.parse::<u64>().unwrap()).unwrap()),
            "i64" => into_vec(|res|res.write_i64::<EncodingByteOrder>(input.parse::<i64>().unwrap()).unwrap()),
            "u128" => into_vec(|res|res.write_u128::<EncodingByteOrder>(input.parse::<u128>().unwrap()).unwrap()),
            "i128" => into_vec(|res|res.write_i128::<EncodingByteOrder>(input.parse::<i128>().unwrap()).unwrap()),
            "data" => decode(&input[2..]).unwrap(),
            "data4"  => decode(&input[2..10]).unwrap(),
            "data8"  => decode(&input[2..18]).unwrap(),
            "data12"  => decode(&input[2..26]).unwrap(),
            "data16"  => decode(&input[2..34]).unwrap(),
            "data20"  => decode(&input[2..42]).unwrap(),
            "data24"  => decode(&input[2..50]).unwrap(),
            "data28"  => decode(&input[2..58]).unwrap(),
            "data32"  => decode(&input[2..66]).unwrap(),
            "data40"  => decode(&input[2..82]).unwrap(),
            "data48"  => decode(&input[2..98]).unwrap(),
            "data56"  => decode(&input[2..114]).unwrap(),
            "data64"  => decode(&input[2..130]).unwrap(),
            "data80"  => decode(&input[2..162]).unwrap(),
            "data96"  => decode(&input[2..194]).unwrap(),
            "data112"  => decode(&input[2..226]).unwrap(),
            "data128"  => decode(&input[2..258]).unwrap(),
            "data160"  => decode(&input[2..322]).unwrap(),
            "data192"  => decode(&input[2..386]).unwrap(),
            "data224"  => decode(&input[2..450]).unwrap(),
            "publicId" => decode(input).unwrap(),
            _ => panic!(),
        },
        _ => panic!(),
    }
}

pub fn gen_lit_desc(typ:&Type) -> LitDesc {
    match &typ.main {
        Ref::Native(ref id) => match id.0.as_ref() {
            "u8" => LitDesc::U8,
            "i8" => LitDesc::I8,
            "u16" => LitDesc::U16,
            "i16" => LitDesc::I16,
            "u32" => LitDesc::U32,
            "i32" => LitDesc::I32,
            "u64" => LitDesc::U64,
            "i64" => LitDesc::I64,
            "u128" => LitDesc::U128,
            "i128" => LitDesc::I128,
            "data" => LitDesc::Data,
            "publicId" => LitDesc::Id,
            _ => panic!()
        },
        _ => panic!(),
    }
}