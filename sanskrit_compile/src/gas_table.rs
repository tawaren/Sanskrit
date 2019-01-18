

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
                NativeType::Unique
                | NativeType::Singleton
                | NativeType::Index
                | NativeType::Ref => 20,
                _ => unreachable!(),
            },
            _ => unreachable!()
        }
    }

    fn is_data(typ:&Crc<ResolvedType>) -> bool{
        match **typ {
            ResolvedType::Native { typ, ..} => match typ {
                NativeType::Data(_)
                | NativeType::Unique
                | NativeType::Singleton
                | NativeType::Index
                | NativeType::Ref => true,
                _ => false,
            },
            _ => unreachable!()
        }
    }

    fn traversing_statistics<S:Store>(typ:&Crc<ResolvedType>, context:&Context<S>, obj_factor:usize, byte_factor:usize) -> Result<usize> {
        Ok(match **typ {
            ResolvedType::Native { typ, ref applies, ..} => match typ {
                NativeType::Data(s) => obj_factor + (s as usize)*byte_factor,
                NativeType::SInt(s)
                | NativeType::UInt(s) => obj_factor + (s as usize)*byte_factor,
                NativeType::Bool => obj_factor + byte_factor,
                NativeType::Tuple(s) => {
                    let mut sum = obj_factor;
                    for appl in applies {
                        sum += traversing_statistics(&appl.typ,context,obj_factor,byte_factor)?;
                    }
                    sum
                },
                NativeType::Alternative(s) => {
                    let mut max = 0;
                    for appl in applies {
                        max = max.max(traversing_statistics(&appl.typ,context,obj_factor,byte_factor)?);
                    }
                    max+obj_factor
                },
                NativeType::Context => obj_factor + 8*byte_factor,
                NativeType::Unique
                | NativeType::Singleton
                | NativeType::Index
                | NativeType::Ref => obj_factor + 20*byte_factor,
            },
            ResolvedType::Import { ref module, offset, ref applies, ..} => {
                //Get the cache
                let adt_cache = context.store.get_adt(&**module, offset)?;
                //Get the adt
                let adt = adt_cache.retrieve();
                //get its context with the applies as substitutions
                let new_context = adt_cache.substituted_context(applies.iter().map(|apl|apl.typ.clone()).collect(),&context.store)?;
                let mut max = 0;
                for ctr in &adt.constructors {
                    let mut sum = obj_factor;
                    for f in &ctr.fields {
                        let typ = f.fetch(&new_context)?;
                        sum += traversing_statistics(&f.fetch(&new_context)?,&new_context,obj_factor,byte_factor)?;
                    }
                    max = max.max(sum);
                }
                max+1
            },
            _ => unreachable!()
        })
    }

    //these are slightly different but are the base
    const STACK_READ:usize = 1;
    const STACK_PUSH:usize = 1;
    const FRAME_PUSH:usize = 2;
    const STACK_REWIND:usize = 1;
    //this looks at the worst case if we have to process data and do it on a byte level
    const BASIC_OP:usize = 1;
    const BASIC_ALLOC:usize = 1; //just a mem_copy
    const OBJECT_VISIT:usize = 1; //ptr fetch
    const BYTE_HASH_COST:usize = 1; //Just a educated cost guess //is cheap as the hasher will collect blocks and not hash single bytes
    const HASH_FINALISATION_COST:usize = 45; //Just a educated guess
    const DISPATCH:usize = 1;

    pub fn and(typ:&[Crc<ResolvedType>]) -> usize {
        if is_data(&typ[0])  {
            (prim_width(&typ[0]) * (BASIC_OP + BASIC_ALLOC)) + (2*STACK_READ) + STACK_PUSH + BASIC_ALLOC
        } else {
            BASIC_OP + 2*BASIC_ALLOC + (2*STACK_READ) + STACK_PUSH
        }
    }

    pub fn or(typ:&[Crc<ResolvedType>]) -> usize {
        if is_data(&typ[0])  {
            (prim_width(&typ[0]) * (BASIC_OP + BASIC_ALLOC)) + (2*STACK_READ) + STACK_PUSH + BASIC_ALLOC
        } else {
            BASIC_OP + 2*BASIC_ALLOC + (2*STACK_READ) + STACK_PUSH
        }
    }

    pub fn xor(typ:&[Crc<ResolvedType>]) -> usize {
        if is_data(&typ[0])  {
            (prim_width(&typ[0]) * (BASIC_OP + BASIC_ALLOC)) + (2*STACK_READ) + STACK_PUSH + BASIC_ALLOC
        } else {
            BASIC_OP + 2*BASIC_ALLOC + (2*STACK_READ) + STACK_PUSH
        }
    }

    pub fn not(typ:&[Crc<ResolvedType>]) -> usize {
        if is_data(&typ[0])  {
            (prim_width(&typ[0]) * (BASIC_OP + BASIC_ALLOC)) + STACK_READ + STACK_PUSH + BASIC_ALLOC
        } else {
            BASIC_OP + 2*BASIC_ALLOC + (2*STACK_READ) + STACK_PUSH
        }
    }

    pub fn convert() -> usize {
        BASIC_OP + (STACK_READ + STACK_PUSH) + BASIC_ALLOC
    }

    pub fn add() -> usize {
        BASIC_OP + (2*STACK_READ) + STACK_PUSH + BASIC_ALLOC
    }

    pub fn sub() -> usize {
        BASIC_OP + (2*STACK_READ) + STACK_PUSH + BASIC_ALLOC
    }

    pub fn mul() -> usize {
        //the additional +BASIC_OP is for mul as it is slightly more expensive than add/sub
        2*BASIC_OP + (2*STACK_READ) + STACK_PUSH + BASIC_ALLOC
    }

    pub fn div() -> usize {
        //not byte dependent (2* is for i/u128 which may need slightly more)
        //the additional +3*BASIC_OP is for div as it is  more expensive than add/sub/mul
        4*BASIC_OP + (2*STACK_READ) + STACK_PUSH + BASIC_ALLOC
    }

    pub fn eq<S:Store>(typ:&[Crc<ResolvedType>], context:&Context<S>) -> Result<usize> {
        Ok((2*STACK_READ) + STACK_PUSH + BASIC_ALLOC + traversing_statistics(&typ[0], context, OBJECT_VISIT+BASIC_OP, BASIC_OP )?)
    }

    pub fn hash<S:Store>(typ:&[Crc<ResolvedType>], context:&Context<S>) -> Result<usize> {
        Ok(HASH_FINALISATION_COST + STACK_READ + STACK_PUSH + BASIC_ALLOC + traversing_statistics(&typ[0], context, OBJECT_VISIT, BYTE_HASH_COST )?)
    }

    pub fn hash_plain(typ:&[Crc<ResolvedType>]) -> usize {
        HASH_FINALISATION_COST + STACK_READ + STACK_PUSH + BASIC_ALLOC + (prim_width(&typ[0]) * BYTE_HASH_COST)
    }

    pub fn join_hash() -> usize {
        HASH_FINALISATION_COST + STACK_READ + STACK_PUSH + BASIC_ALLOC + (40 * BYTE_HASH_COST)
    }

    pub fn cmp() -> usize {
        BASIC_OP + (2*STACK_READ) + STACK_PUSH + BASIC_ALLOC
    }


    pub fn to_data(typ:&[Crc<ResolvedType>]) -> usize {
        (prim_width(&typ[0]) * BASIC_ALLOC) + STACK_READ + STACK_PUSH + BASIC_ALLOC
    }

    pub fn concat(typ:&[Crc<ResolvedType>]) -> usize {
        (prim_width(&typ[2]) * BASIC_ALLOC) + 2*STACK_READ + STACK_PUSH + BASIC_ALLOC
    }

    pub fn get_bit(typ:&[Crc<ResolvedType>]) -> usize {
        BASIC_OP + 2*STACK_READ + STACK_PUSH + BASIC_ALLOC
    }

    pub fn set_bit(typ:&[Crc<ResolvedType>]) -> usize {
        BASIC_OP + (prim_width(&typ[0]) * BASIC_ALLOC) + 2*STACK_READ + STACK_PUSH + BASIC_ALLOC
    }

    pub fn gen_unique() -> usize {
        HASH_FINALISATION_COST + 2*STACK_READ + 2*STACK_PUSH + 2*BASIC_ALLOC + ((8+20) * BYTE_HASH_COST)
    }

    pub fn fetch_env_hash() -> usize {
        //Env read is like stack read some deref
        2*STACK_READ + STACK_PUSH + (20 * BASIC_ALLOC)
    }

    pub fn fetch_env_val() -> usize {
        //Env read is like stack read some deref
        2*STACK_READ + STACK_PUSH + BASIC_ALLOC
    }

    pub fn call(args:usize) -> usize {
        (args*STACK_READ) + (args*STACK_PUSH)
    }

    pub fn op_process() -> usize {
        DISPATCH
    }

    pub fn frame_process() -> usize {
        FRAME_PUSH + STACK_REWIND
    }

    pub fn lit(typ:Crc<ResolvedType>) -> usize {
        if is_data(&typ) {
            (prim_width(&typ) * BASIC_ALLOC) + STACK_READ + STACK_PUSH + BASIC_ALLOC
        } else {
            BASIC_ALLOC + STACK_READ + STACK_PUSH + BASIC_ALLOC
        }
    }

    pub fn unpack(fields:usize) -> usize {
        STACK_READ + fields*STACK_PUSH
    }

    pub fn field() -> usize {
        STACK_READ + STACK_PUSH
    }

    pub fn pack(fields:usize) -> usize {
        fields*STACK_READ + STACK_PUSH + BASIC_ALLOC
    }

    pub fn switch() -> usize {
        STACK_READ + DISPATCH
    }

    pub fn try(catches:usize) -> usize {
        catches*BASIC_OP + FRAME_PUSH //tries need an extra frame
    }
}
