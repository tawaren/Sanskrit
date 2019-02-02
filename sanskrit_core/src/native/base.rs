use sanskrit_common::errors::*;
use model::resolved::*;
use utils::Crc;
use alloc::prelude::*;
use sanskrit_common::model::*;
use resolver::apply_types;
use core::iter::repeat;


//checks if a native type is a literal
pub fn is_native_literal(typ: NativeType) -> bool {
    match typ {
        NativeType::SInt(_) | NativeType::UInt(_) | NativeType::Data(_) | NativeType::Ref => true,
        _ => false,
    }
}


//gets the constructors for a antive type
pub fn get_native_type_constructors(typ: NativeType, base_applies:&[ResolvedApply]) -> Result<Vec<Vec<Crc<ResolvedType>>>> {
    match typ {
        //Ints & Data do not have Costructors they are constructed over literals
        // bt for size calcs they ger a pseudo Ctr (as else they would be treated as zero byte)
        NativeType::SInt(_)
        | NativeType::UInt(_)
        | NativeType::Data(_)
        | NativeType::Id
        | NativeType::Ref =>  no_ctr_available(),
        //Bools have 2 Ctrs with 0 fields each
        NativeType::Bool => Ok(vec![vec![]; 2]),
        //Tuple has one ctr with a dynamic number of fields (each its own type)
        NativeType::Tuple(_) => Ok(vec![base_applies.iter().map(|t|t.typ.clone()).collect()]),
        //Alternative has dynamic number of ctrs with one field
        NativeType::Alternative(_) => Ok(base_applies.iter().map(|t|vec![t.typ.clone()]).collect()),
    }
}

pub fn resolved_native_type(typ: NativeType, base_applies:&[Crc<ResolvedType>]) ->  Crc<ResolvedType> {
    //get the base caps (does not consider generics)
    let base_caps = typ.base_caps();
    //build the type (consider generics where required)
    Crc::new(match typ {
        //these have no gernerics
        NativeType::Data(_)
        | NativeType::SInt(_)
        | NativeType::UInt(_)
        | NativeType::Bool
        | NativeType::Ref
        | NativeType::Id => ResolvedType::Native {
            //In the absence of  (non-phantom) generics all the caps are the same
            base_caps,
            caps:base_caps,
            extended_caps:base_caps,
            typ,
            applies: vec![]
        },
        // these have real generics (influence caps)
        NativeType::Tuple(_)
        | NativeType::Alternative(_) => {
            let (extended_caps,caps,applies) = apply_types(base_caps,repeat(false),base_applies);
            ResolvedType::Native {
                caps,
                extended_caps,
                base_caps,
                typ,
                applies
            }
        },
    })
}


//Native types have some restrictions
// Ths checks if they hold on the import level
pub fn check_native_type_constraints(typ: NativeType, types:&[Crc<ResolvedType>]) -> Result<()>{
    fn are_real(types:&[Crc<ResolvedType>]) -> Result<()>{
        //Check that the phantom constraints hold
        for appl in types {
            //All primitive generics are real and do not require any caps (So only phantom needs checking)
            if let ResolvedType::Generic { is_phantom:true,  .. }  = **appl {
                return can_not_apply_phantom_to_physical_error()
            }
        }
        Ok(())
    }

    fn have_embed(types:&[Crc<ResolvedType>]) -> Result<()>{
        //Check that the phantom constraints hold
        for appl in types {
            if !appl.get_caps().contains(NativeCap::Embed) {
                return type_apply_constraint_violation()
            }
        }
        Ok(())
    }

    match typ {
        //Ints come only in power of 2 up to 128bit
        NativeType::SInt(arg)| NativeType::UInt(arg) =>{
            //Ints do not have generic types
            if types.is_empty() {
                //Not all sizes are valid
                match arg {
                    1 | 2 | 4 | 8 | 16  => return Ok(()),
                    _ => return native_type_not_exist_error()
                }
            }
            num_applied_generics_error()
        },

        //Data and bool are not allowed to have type params
        NativeType::Data(_)
        | NativeType::Bool => {
            //Bool do not have generic types
            if types.is_empty() {
                Ok(())
            } else {
                num_applied_generics_error()
            }
        }
        //The number of tuple type parameters must much the number of its fields
        NativeType::Tuple(arg)
        //Alternative must have one type param per constructor
        | NativeType::Alternative(arg) => {
            //Tuple andAlternative have arg generic types
            if types.len() == (arg as usize)  {
                have_embed(types)?;
                are_real(types)
            } else {
                num_applied_generics_error()
            }
        },
        //These have 0 params
        NativeType::Id
        | NativeType::Ref => {
            //These have zero generic type
            if types.is_empty() {
                Ok(())
            } else {
                num_applied_generics_error()
            }
        },
    }
}