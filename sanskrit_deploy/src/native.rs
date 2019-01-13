use sanskrit_core::model::resolved::ResolvedType;
use sanskrit_core::utils::Crc;
use sanskrit_common::errors::*;
use sanskrit_common::model::NativeType;

//Checks if a byte stream is a valid instance of some native type
pub fn check_valid_literal_construction(data:&[u8], lit_type:&Crc<ResolvedType>) -> Result<()> {
    //Next check that the number of supplied bytes is in range
    match **lit_type {
        ResolvedType::Native {typ,..} => {
            match typ {
                //If to many bytes are supplied the int can not hold the corresponding value
                NativeType::SInt(arg) | NativeType::UInt(arg) => {
                    if data.len() > arg as usize {
                        literal_data_error()
                    } else {Ok(())}
                },
                //the supplied amount of data bytes must match exactly
                NativeType::Data(arg) => {
                    if data.len() != arg as usize {
                        literal_data_error()
                    } else {Ok(())}
                },
                //refs need to have 20 Bytes like keys
                NativeType::Ref => {
                    if data.len() != 20 as usize {
                        literal_data_error()
                    } else {Ok(())}
                },
                //Remaining Natives can not be constructed over a literal
                _ => not_a_literal_error(),
            }
        },
        _ => not_a_literal_error()
    }
}
