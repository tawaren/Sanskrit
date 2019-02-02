use model::resolved::*;
use sanskrit_common::errors::*;
use utils::Crc;
use utils::build_set;
use alloc::rc::Rc;
use native::base::resolved_native_type;
use sanskrit_common::model::*;


//Produces a resolved function
pub fn resolved_native_function(fun: NativeFunc, base_applies:&[Crc<ResolvedType>]) -> Result<ResolvedSignature> {
    Ok(match fun {
        //And, Or, Xor have no risks and 2 borrow parameter and one return (all of the same type)
        NativeFunc::And | NativeFunc::Or | NativeFunc::Xor => {
            ResolvedSignature {
                risks: build_set(&[]),
                params: vec![
                    ResolvedParam{ consumes: false, typ: base_applies[0].clone() },
                    ResolvedParam{ consumes: false, typ: base_applies[0].clone() }
                ],
                returns: vec![
                    ResolvedReturn {  borrows:vec![], typ: base_applies[0].clone()  }
                ],
            }
        },
        //Not has no risks and 1 borrow parameter and one return (all of the same type)
        NativeFunc::Not => {
            ResolvedSignature {
                risks: build_set(&[]),
                params: vec![ ResolvedParam{ consumes: false,  typ: base_applies[0].clone()}],
                returns: vec![ ResolvedReturn {  borrows:vec![], typ: base_applies[0].clone()}],
            }
        },

        //Extend has no risks and 1 borrow parameter and one return (different types)
        NativeFunc::Extend => {
            ResolvedSignature {
                risks: build_set(&[]),
                params: vec![ResolvedParam{ consumes: false,  typ: base_applies[0].clone() }],
                returns: vec![ResolvedReturn {  borrows:vec![], typ: base_applies[1].clone()}],
            }
        },

        //Cut has 1 risk (NumericOverflow) and 1 borrow parameter and one return (different types)
        NativeFunc::Cut | NativeFunc::SignCast => {
            ResolvedSignature {
                risks: build_set(&[Rc::new(ResolvedErr::Native{err:NativeError::NumericError })]),
                params: vec![ResolvedParam{ consumes: false, typ: base_applies[0].clone() }],
                returns: vec![ResolvedReturn { borrows:vec![], typ: base_applies[1].clone() }],
            }
        },

        //Add,Sub,Mul,Div have 1 risk (NumericOverflow) and 2 borrow parameter and one return (all of the same type)
        NativeFunc::Add | NativeFunc::Sub | NativeFunc::Mul | NativeFunc::Div => {
            ResolvedSignature {
                risks: build_set(&[Rc::new(ResolvedErr::Native{err:NativeError::NumericError })]),
                params: vec![
                    ResolvedParam{ consumes: false, typ: base_applies[0].clone() },
                    ResolvedParam{ consumes: false, typ: base_applies[0].clone() }
                ],
                returns: vec![
                    ResolvedReturn { borrows:vec![], typ: base_applies[0].clone()  }
                ],
            }
        },

        //Eq,Lt,Gt,Lte,Gte have no risk and 2 borrow parameter of the same type and one bool return
        NativeFunc::Eq | NativeFunc::Lt | NativeFunc::Gt | NativeFunc::Lte  | NativeFunc::Gte=> {
            ResolvedSignature {
                risks: build_set(&[]),
                params: vec![
                    ResolvedParam{ consumes: false, typ: base_applies[0].clone() },
                    ResolvedParam{ consumes: false, typ: base_applies[0].clone() }
                ],
                returns: vec![ ResolvedReturn { borrows:vec![], typ: resolved_native_type(NativeType::Bool, &[]) }],
            }
        },

        //Hash and Plain Hash have no risks and borrow a parameter creating a hash of type data20 from it
        NativeFunc::Hash | NativeFunc::PlainHash => {
            ResolvedSignature {
                risks: build_set(&[]),
                params: vec![ ResolvedParam{ consumes: false, typ: base_applies[0].clone() } ],
                returns: vec![ ResolvedReturn { borrows:vec![], typ: resolved_native_type(NativeType::Data(20), &[]) }],
            }
        },

        //ToData has no risks and makes a data value from its input
        NativeFunc::ToData => {
            ResolvedSignature {
                risks: build_set(&[]),
                params: vec![ ResolvedParam{ consumes: false, typ: base_applies[0].clone() } ],
                returns: vec![ ResolvedReturn { borrows:vec![], typ: base_applies[1].clone() }],
            }
        },

        //Concat has no risks and takes two data input returning the concatenated data output
        NativeFunc::Concat => {
            ResolvedSignature {
                risks: build_set(&[]),
                params: vec![
                    ResolvedParam{ consumes: false, typ: base_applies[0].clone() },
                    ResolvedParam{ consumes: false, typ: base_applies[1].clone() }
                ],
                returns: vec![ ResolvedReturn { borrows:vec![], typ: base_applies[2].clone() }],
            }
        },

        //SetBit has the out of range risk
        NativeFunc::SetBit => {
            ResolvedSignature {
                risks: build_set(&[Rc::new(ResolvedErr::Native{err:NativeError::IndexError })]),
                params: vec![
                    ResolvedParam{ consumes: false, typ: base_applies[0].clone() },
                    ResolvedParam{ consumes: false, typ: base_applies[1].clone() },
                    ResolvedParam{ consumes: false, typ: resolved_native_type(NativeType::Bool, &[]) },
                ],
                returns: vec![
                    ResolvedReturn { borrows:vec![], typ: base_applies[0].clone()  }
                ],
            }
        },

        //Get bit has the out of range risk
        NativeFunc::GetBit  => {
            ResolvedSignature {
                risks: build_set(&[Rc::new(ResolvedErr::Native{err:NativeError::IndexError })]),
                params: vec![
                    ResolvedParam{ consumes: false, typ: base_applies[0].clone() },
                    ResolvedParam{ consumes: false, typ: base_applies[1].clone() }
                ],
                returns: vec![
                    ResolvedReturn { borrows:vec![], typ:  resolved_native_type(NativeType::Bool, &[])  }
                ],
            }
        },

        //GenIndex has no risks and returns a new index generated from its input
        NativeFunc::GenId => {
            ResolvedSignature {
                risks: build_set(&[]),
                params: vec![ ResolvedParam{ consumes: true, typ: base_applies[0].clone() } ],
                returns: vec![ ResolvedReturn { borrows:vec![], typ: resolved_native_type(NativeType::Id, &[]) }],
            }
        },

        //Derive combines its inputs into a output that is deterministically derived
        NativeFunc::Derive => {
            ResolvedSignature {
                risks: build_set(&[]),
                params: vec![
                    ResolvedParam{ consumes: false, typ: base_applies[0].clone() },
                    ResolvedParam{ consumes: false, typ: base_applies[1].clone() }
                ],
                returns: vec![ ResolvedReturn { borrows:vec![], typ: base_applies[0].clone() }],
            }
        },

        //ToRef has no riks and returns a reference generated from its inputs
        NativeFunc::ToRef => {
            ResolvedSignature {
                risks: build_set(&[]),
                params: vec![ ResolvedParam{ consumes: false, typ: base_applies[0].clone() }],
                returns: vec![ ResolvedReturn { borrows:vec![], typ: resolved_native_type(NativeType::Ref, &[])  }],
            }
        },
    })
}


//Checks if an import of a Native function is allowed
pub fn check_native_function_constraints(fun: NativeFunc, types:&[Crc<ResolvedType>]) -> Result<()>{
    //helper that checks if something is of int, bool or data type
    fn is_int_or_bool_or_data(typ:&Crc<ResolvedType>) -> Result<bool>{
        Ok(match **typ {
            ResolvedType::Native{ typ: NativeType::UInt(_), .. } => true,
            ResolvedType::Native{ typ: NativeType::SInt(_), .. }  => true,
            ResolvedType::Native{ typ: NativeType::Bool, .. } => true,
            ResolvedType::Native{ typ: NativeType::Data(_), .. } => true,
            _ => false,
        })
    }

    //helper that checks if something is of data type
    fn is_data(typ:&Crc<ResolvedType>) -> Result<bool>{
        Ok(match **typ {
            ResolvedType::Native{ typ: NativeType::Data(_), .. } => true,
            _ => false,
        })
    }

    //checks if 2 ints have the same size but a different sign
    fn same_sized_different_signed_int(typ1:&Crc<ResolvedType>, typ2:&Crc<ResolvedType>) -> Result<bool>{
        Ok(match (&**typ1,&**typ2) {
            (ResolvedType::Native{ typ: NativeType::UInt(s1), .. }, ResolvedType::Native{ typ: NativeType::SInt(s2), .. }) => s1 == s2,
            (ResolvedType::Native{ typ: NativeType::SInt(s1), .. }, ResolvedType::Native{ typ: NativeType::UInt(s2), .. }) => s1 == s2,
            _ => false,
        })
    }

    //check if two ints have the same sign but the second is at least as bi as the first
    fn same_but_bigger_int(typ1:&Crc<ResolvedType>, typ2:&Crc<ResolvedType>) -> Result<bool>{
        Ok(match (&**typ1,&**typ2) {
            (ResolvedType::Native{ typ: NativeType::UInt(a), .. }, ResolvedType::Native{ typ: NativeType::UInt(b), .. }) => b >= a,
            (ResolvedType::Native{ typ: NativeType::SInt(a), .. }, ResolvedType::Native{ typ: NativeType::SInt(b), .. }) => b >= a,
            _ => false,
        })
    }

    //check if something is an int
    fn is_int(typ:&Crc<ResolvedType>) -> Result<bool>{
        Ok(match **typ {
            ResolvedType::Native{ typ: NativeType::UInt(_), .. } => true,
            ResolvedType::Native{ typ: NativeType::SInt(_), .. } => true,
            _ => false,
        })
    }

    //check if something is an int
    fn positive_small_int(typ:&Crc<ResolvedType>) -> Result<bool>{
        Ok(match **typ {
            ResolvedType::Native{ typ: NativeType::UInt(s), .. } => s <= 2,
            _ => false,
        })
    }

    //checks that the types are data types and that the return type len is the sum of the inputs
    fn summed_up_data(typ1:&Crc<ResolvedType>, typ2:&Crc<ResolvedType>, typ3:&Crc<ResolvedType>) ->  Result<bool> {
        Ok(match (&**typ1,&**typ2,&**typ3) {
            (
                ResolvedType::Native{ typ: NativeType::Data(s1), .. },
                ResolvedType::Native{ typ: NativeType::Data(s2), .. },
                ResolvedType::Native{ typ: NativeType::Data(s3), .. },
            ) => *s1 as usize + *s2 as usize == *s3 as usize, //the as usize ensures a no overflow rule
            _ => false,
        })
    }

    //check if something is an int of a specific width
    fn same_sized_int_key_or_data(typ1:&Crc<ResolvedType>, typ2:&Crc<ResolvedType>) -> Result<bool>{
        let width = match **typ2 {
            ResolvedType::Native { typ: NativeType::Data(width), .. } => width,
            _ => return Ok(false),
        };

        Ok(match **typ1 {
            ResolvedType::Native{ typ: NativeType::UInt(s), .. } => u16::from(s) == width,
            ResolvedType::Native{ typ: NativeType::SInt(s), .. } => u16::from(s) == width,
            ResolvedType::Native{ typ: NativeType::Data(s), .. } => s == width,
            ResolvedType::Native{ typ: NativeType::Ref, .. } => 20 == width,
            ResolvedType::Native{ typ: NativeType::Id, .. } => 20 == width,
            _ => false,
        })
    }

    //check if a type has a statically known upper bound size
    //meaning no generic args (except for phantoms)
    fn is_sized(typ:&Crc<ResolvedType>) -> bool {
        fn are_args_sized(args:&[ResolvedApply]) -> bool {
            for a in args {
                if !a.is_phantom {
                    match *a.typ {
                        ResolvedType::Generic { is_phantom:false, ..} => return false,
                        ResolvedType::Import { ref applies, .. }
                        | ResolvedType::Native { ref applies, .. } => if !are_args_sized(applies) { return false },
                        _ => {}
                    }
                }
            }
            true
        }
        match **typ {
            ResolvedType::Generic { .. } => false,
            ResolvedType::Image { ref typ } => is_sized(typ),
            ResolvedType::Import { ref applies, .. }
            | ResolvedType::Native { ref applies, .. } => are_args_sized(applies),
        }
    }

    //check if something can be used as an Id
    fn is_ref_input(typ:&Crc<ResolvedType>) -> Result<bool>{
        Ok(match **typ {
            ResolvedType::Native{ typ: NativeType::Id, .. } => true,
            ResolvedType::Native{ typ: NativeType::Data(20), .. } => true,
            _ => false,
        })
    }

    //check if something is a key or refs a key
    fn is_derivable(typ:&Crc<ResolvedType>) -> Result<bool>{
        Ok(match **typ {
            ResolvedType::Native{ typ: NativeType::Id, .. } => true,
            ResolvedType::Native{ typ: NativeType::Ref, .. } => true,
            _ => false,
        })
    }

    //check if something is a key or refs a key
    fn is_data_material(typ:&Crc<ResolvedType>) -> Result<bool>{
        Ok(match **typ {
            ResolvedType::Native{ typ: NativeType::Data(_), .. } => true,
            ResolvedType::Native{ typ: NativeType::Id, .. } => true,
            ResolvedType::Native{ typ: NativeType::Ref, .. } => true,
            _ => false,
        })
    }

    //Fast way to produce an error from a bool
    fn true_or_err(check:bool) -> Result<()> {
        if check { Ok(())  } else { generic_args_mismatch() }
    }

    //Note: We omit phantom apply check as there are no native functions with phantom parameters

    match fun {
        //And, Or, Xor, Not are defined only fo Ints, Bools and Data and must have one type parameter
        NativeFunc::And | NativeFunc::Or | NativeFunc::Xor | NativeFunc::Not => true_or_err(types.len() == 1 && is_int_or_bool_or_data(&types[0])?),
        //Extend must have 2 type parameters where both int with the same sign mode where the second is bigger (or equal)
        NativeFunc::Extend => true_or_err( types.len() == 2 && same_but_bigger_int(&types[0], &types[1])?),
        //Extend must have 2 type parameters where both int with the same sign mode where the second is smaller (or equal)
        NativeFunc::Cut => true_or_err( types.len() == 2 && same_but_bigger_int(&types[1], &types[0])?),
        //SignCast must have 2 type parameters where both int with the same size but different sign mode
        NativeFunc::SignCast => true_or_err(types.len() == 2 && same_sized_different_signed_int(&types[0], &types[1])?),
        //Eq & Hash is defined for everithing
        NativeFunc::Eq | NativeFunc::Hash => true_or_err(types.len() == 1 && is_sized(&types[0])) ,
        //PlainHash is defined for data only
        NativeFunc::PlainHash => true_or_err(types.len() == 1 && is_data(&types[0])?),
        //Add, Sub, Mul, Div are defined only fo Ints and must have one type parameter
        NativeFunc::Add | NativeFunc::Sub | NativeFunc::Mul | NativeFunc::Div | NativeFunc::Lt | NativeFunc::Gt | NativeFunc::Lte | NativeFunc::Gte  => {
            true_or_err(types.len() == 1 && is_int(&types[0])?)
        },
        //ToData takes 2 type Params a int and a Data (with same width)
        NativeFunc::ToData =>  true_or_err(types.len() == 2 && same_sized_int_key_or_data(&types[0], &types[1])?),
        //Concat takes 3 type Params allData (where size 1 + size 2 = size 3)
        NativeFunc::Concat =>  true_or_err(types.len() == 3 && summed_up_data(&types[0], &types[1], &types[2])?),
        //Set & Get Bit takes 2 type params 1 for the data and one for the index
        NativeFunc::GetBit | NativeFunc::SetBit =>  true_or_err(types.len() == 2 && is_data(&types[0])? && positive_small_int(&types[1])?),
        //Gen a Key has one type pram that can be a unique/singleton/data
        NativeFunc::GenId => true_or_err(types.len() == 1 && is_data(&types[0])?),
        //Derives a new Key from two existing ones (or a ref from 2 Refs)
        NativeFunc::Derive => true_or_err(types.len() == 2 && is_derivable(&types[0])? && is_data_material(&types[1])?),
        //Generates a Ref From a Key or from raw Data
        NativeFunc::ToRef => true_or_err(types.len() == 1 && is_ref_input(&types[0])?),
    }
}
