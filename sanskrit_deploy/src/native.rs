use sanskrit_core::model::resolved::ResolvedType;
use sanskrit_core::utils::Crc;
use sanskrit_common::errors::*;

/*
//Checks if a byte stream is a valid instance of some native type
pub fn check_valid_literal_construction(data:&[u8], lit_type:&Crc<ResolvedType>) -> Result<()> {
    //Next check that the number of supplied bytes is in range
    match **lit_type {
        ResolvedType::Lit {size,..} if size as usize == data.len() => Ok(()) ,
        _ => not_a_literal_error()
    }
}
*/