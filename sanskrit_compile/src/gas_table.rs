
//NOTE: THESE ARE NOT BASED ON MEASUREMENTS BUT ON INTUITION
pub mod gas {
    use sanskrit_core::model::resolved::ResolvedType;
    use sanskrit_common::model::NativeType;
    use sanskrit_core::utils::Crc;
    use sanskrit_common::store::Store;
    use sanskrit_core::resolver::Context;
    use sanskrit_common::errors::*;
    use sanskrit_core::model::linking::Ref;

    fn prim_width(typ:&Crc<ResolvedType>) -> usize{
        match **typ {
            ResolvedType::Native { typ, ..} => match typ {
                NativeType::Data(s) => s as usize,
                NativeType::SInt(s)
                | NativeType::UInt(s) => s as usize,
                NativeType::Bool => 1,
                NativeType::PrivateId
                | NativeType::PublicId => 20,
                _ => unreachable!(),
            },
            _ => unreachable!()
        }
    }

    fn is_data(typ:&Crc<ResolvedType>) -> bool{
        match **typ {
            ResolvedType::Native { typ, ..} => match typ {
                NativeType::Data(_)
                | NativeType::PrivateId
                | NativeType::PublicId => true,
                _ => false,
            },
            _ => unreachable!()
        }
    }

    fn traversing_statistics<S:Store>(typ:&Crc<ResolvedType>, context:&Context<S>, node_fix:u64,  leaf_static:u64, leaf_per_byte:u64, prim_fact:u64) -> Result<u64> {
        Ok(match **typ {
            ResolvedType::Native { typ, ref applies, ..} => match typ {
                NativeType::Data(s) => leaf_static + (s as u64)*leaf_per_byte,
                NativeType::SInt(s)
                | NativeType::UInt(s) => leaf_static + (s as u64)*leaf_per_byte*prim_fact,
                NativeType::Bool => leaf_static + leaf_per_byte*prim_fact,
                NativeType::Nothing => leaf_static,
                NativeType::Tuple(_) => {
                    let mut sum = node_fix;
                    for appl in applies {
                        sum += traversing_statistics(&appl.typ,context,node_fix,leaf_static, leaf_per_byte,prim_fact)?;
                    }
                    sum
                },
                NativeType::Alternative(_) => {
                    let mut max = 0u64;
                    for appl in applies {
                        max = max.max(traversing_statistics(&appl.typ,context,node_fix,leaf_static, leaf_per_byte,prim_fact)?);
                    }
                    max+node_fix
                },
                NativeType::PrivateId
                | NativeType::PublicId => leaf_static + 20*leaf_per_byte,
            },
            ResolvedType::Import { ref module, offset, ref applies, ..} => {
                //Get the cache
                let adt_cache = context.store.get_adt(&**module, offset)?;
                //Get the adt
                let adt = adt_cache.retrieve();
                //get its context with the applies as substitutions
                let new_context = adt_cache.substituted_context(applies.iter().map(|apl|apl.typ.clone()).collect(),&context.store)?;
                let mut max = 0u64;
                for ctr in &adt.constructors {
                    let mut sum = leaf_static;
                    for f in &ctr.fields {
                        let typ = f.fetch(&new_context)?;
                        sum += traversing_statistics(&typ,&new_context,node_fix,leaf_static, leaf_per_byte,prim_fact)?;
                    }
                    max = max.max(sum);
                }
                max
            },
            _ => unreachable!()
        })
    }


    //The following values are from a benchmark but with a constant mu to giv niver numbers, old ar in comment
        //the factor is 0.2
    pub fn and(typ:&[Crc<ResolvedType>]) -> usize {
        if is_data(&typ[0])  {
            let width = prim_width(&typ[0]);
            width/8 + 19//(width*2) + (4*width)/64 + 95
        } else {
            13
        }
    }

    pub fn or(typ:&[Crc<ResolvedType>]) -> usize {
        if is_data(&typ[0])  {
            let width = prim_width(&typ[0]);
            width/8 + 19//(width*2) + (4*width)/64 + 95
        } else {
            13
        }
    }

    pub fn xor(typ:&[Crc<ResolvedType>]) -> usize {
        if is_data(&typ[0])  {
            let width = prim_width(&typ[0]);
            width/8 + 19//(width*2) + (4*width)/64 + 95
        } else {
            13
        }
    }

    pub fn not(typ:&[Crc<ResolvedType>]) -> usize {
        if is_data(&typ[0])  {
            //the width * 11/80 is rounded to width/8
            let width = prim_width(&typ[0]);
            width/8 + 17//(width/2) + (12*width)/64 + 85
        } else {
            13
        }
    }

    pub fn convert() -> usize {
        11
    }

    pub fn add() -> usize {
        12
    }

    pub fn sub() -> usize {
        12
    }

    pub fn mul() -> usize {
       13
    }

    pub fn div() -> usize {
        17 /*reality: 65 for u/i8, 70 for u/i16, 75 for u/i32, 80 for u/i64, 85 for u/i128*/
    }

    pub fn eq<S:Store>(typ:&[Crc<ResolvedType>], context:&Context<S>) -> Result<usize> {
        Ok(12 + (traversing_statistics(&typ[0], context, 34, 240, 5, 0)?/200) as usize)
    }

    pub fn hash<S:Store>(typ:&[Crc<ResolvedType>], context:&Context<S>) -> Result<usize> {
        Ok(60 + (traversing_statistics(&typ[0], context, 11, 16, 1, 1)?/4) as usize)
    }

    pub fn hash_plain(typ:&[Crc<ResolvedType>]) -> usize {
        60 + prim_width(&typ[0])/5
    }

    pub fn join_hash() -> usize {
       70
    }

    pub fn cmp() -> usize {
        13
    }


    pub fn to_data(_typ:&[Crc<ResolvedType>]) -> usize {
        18
    }

    pub fn concat(typ:&[Crc<ResolvedType>]) -> usize {
        20 + prim_width(&typ[2])/50
    }

    pub fn get_bit() -> usize {
        13
    }

    pub fn set_bit(typ:&[Crc<ResolvedType>]) -> usize {
        22 + prim_width(&typ[0])/50
    }

    pub fn call(args:usize) -> usize {
        //1.4 was rounded to 1.5
        14 + (3*args)/2//70 + 7*args
    }

    pub fn _let() -> usize { 70 }

    pub fn lit(typ:Crc<ResolvedType>) -> usize {
        if is_data(&typ) {
            13 + prim_width(&typ)/50
        } else {
            7
        }
    }

    pub fn module_index() -> usize {
        13
    }

    pub fn unpack(fields:usize) -> usize {
        3 + fields/2
    }

    pub fn field() -> usize {
        4
    }

    pub fn pack(fields:usize) -> usize {
        13 + fields
    }

    pub fn switch() -> usize {
        16
    }

    pub fn ret(rets:usize) -> usize {
        5 + 5*rets
    }

    pub fn throw() -> usize {
        5
    }

    pub fn try(catches:usize) -> usize {
        16 + catches //catches is a educated guess as the measurements where flawed*/
    }
}
