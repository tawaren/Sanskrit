use crate::model::resolved::{ResolvedType, ResolvedPermission};
use core::cmp::Ordering;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::{Serializable, Serializer, Parsable, Parser, ParserAllocator};
use crate::model::BitSerializedVec;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};

impl Serializable for BitSerializedVec {
    fn serialize(&self, s: &mut Serializer) -> Result<()> {
        (self.0.len() as u16).serialize(s)?;
        let mut cur:u8 = 0;
        for (i,b) in self.0.iter().enumerate() {
            let bit = (i % 8) as u8;
            if *b { cur = cur | 1 << bit; }
            if bit == 7 {
                s.produce_byte(cur);
                cur = 0;
            }
        }
        if self.0.len() % 8 != 0 {
            s.produce_byte(cur);
        }
        Ok(())
    }
}

impl<'a> Parsable<'a> for BitSerializedVec {
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc: &'a A) -> Result<BitSerializedVec> {
        let len = u16::parse(p,alloc)?;
        let mut vec = Vec::with_capacity(len as usize);
        let mut cur = 0;
        for i in 0..len {
            let bit = i % 8;
            if bit == 0 { cur = p.consume_byte()?; }
            vec.push( (cur & 1 << bit) != 0);
        }
        Ok(BitSerializedVec(vec))
    }
}

impl Eq for ResolvedType {}
impl PartialEq for ResolvedType {
    fn eq(&self, other: &Self) -> bool {
        match (self,other) {
            (ResolvedType::Generic { offset:off1, .. }, ResolvedType::Generic { offset:off2, .. })
            => off1 == off2,
            (ResolvedType::Projection { depth:depth1,  un_projected:typ1, .. }, ResolvedType::Projection { depth:depth2, un_projected:typ2, .. })
            => typ1 == typ2 && depth1 == depth2,
            (ResolvedType::Sig { module:mod1, offset:off1, applies:applies1, .. }, ResolvedType::Sig { module:mod2, offset:off2, applies:applies2, .. })
            | (ResolvedType::Data { module:mod1, offset:off1, applies:applies1, .. }, ResolvedType::Data { module:mod2, offset:off2, applies:applies2, .. })
            | (ResolvedType::Lit { module:mod1, offset:off1, applies:applies1, .. }, ResolvedType::Lit { module:mod2, offset:off2, applies:applies2, .. })
            => off1 == off2 && mod1 == mod2 && applies1 == applies2,
            (ResolvedType::Virtual(hash1), ResolvedType::Virtual(hash2))
            => hash1 == hash2,
            _ => false
        }
    }
}

impl PartialOrd for ResolvedType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ResolvedType {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self,other) {
            (ResolvedType::Generic { offset:off1, .. }, ResolvedType::Generic { offset:off2, .. })
            => off1.cmp(off2),
            (ResolvedType::Projection {  depth:depth1, un_projected:typ1, .. }, ResolvedType::Projection {  depth:depth2, un_projected:typ2, .. })
            => typ1.cmp(typ2).then_with(||depth1.cmp(depth2)),
            (ResolvedType::Sig { module:mod1, offset:off1, applies:applies1, .. }, ResolvedType::Sig { module:mod2, offset:off2, applies:applies2, .. })
            | (ResolvedType::Data { module:mod1, offset:off1, applies:applies1, .. }, ResolvedType::Data { module:mod2, offset:off2, applies:applies2, .. })
            | (ResolvedType::Lit { module:mod1, offset:off1, applies:applies1, .. }, ResolvedType::Lit { module:mod2, offset:off2, applies:applies2, .. })
            => off1.cmp(off2).then_with(||mod1.cmp(mod2) ).then_with(||applies1.cmp(applies2)),
            (ResolvedType::Virtual(hash1), ResolvedType::Virtual(hash2))
            => hash1.cmp(hash2),
            (ResolvedType::Generic{ .. }, _) => Ordering::Less,
            (_, ResolvedType::Generic{ .. }) => Ordering::Greater,
            (ResolvedType::Projection{ .. }, _) => Ordering::Less,
            (_, ResolvedType::Projection{ .. }) => Ordering::Greater,
            (ResolvedType::Sig{ .. }, _) => Ordering::Less,
            (_, ResolvedType::Sig{ .. }) => Ordering::Greater,
            (ResolvedType::Data{ .. }, _) => Ordering::Less,
            (_, ResolvedType::Data{ .. }) => Ordering::Greater,
            (ResolvedType::Lit{ .. }, _) => Ordering::Less,
            (_, ResolvedType::Lit{ .. }) => Ordering::Greater,
            //(ResolvedType::Virtual{ .. }, _) => Ordering::Less,
            //(_, ResolvedType::Virtual{ .. }) => Ordering::Greater
        }
    }
}

impl Hash for ResolvedType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ResolvedType::Generic { offset, .. } => {
                state.write_u8(0);
                state.write_u8(*offset);
            },
            ResolvedType::Projection { depth, un_projected, .. } => {
                state.write_u8(1);
                state.write_u8(*depth);
                un_projected.hash(state)
            },
            ResolvedType::Sig { module, offset, applies, .. } => {
                state.write_u8(2);
                state.write_u8(*offset);
                module.hash(state);
                applies.hash(state)
            },
            ResolvedType::Data { module, offset, applies, .. } => {
                state.write_u8(3);
                state.write_u8(*offset);
                module.hash(state);
                applies.hash(state)
            },
            ResolvedType::Lit { module, offset, applies, .. } => {
                state.write_u8(4);
                state.write_u8(*offset);
                module.hash(state);
                applies.hash(state)
            },
            ResolvedType::Virtual(hash) => {
                state.write_u8(5);
                hash.hash(state)
            },
        }
    }
}

impl Eq for ResolvedPermission {}
impl PartialEq for ResolvedPermission {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ResolvedPermission::TypeSig { perm:perm1, typ:typ1, .. }, ResolvedPermission::TypeSig { perm:perm2, typ:typ2, .. })
            | (ResolvedPermission::TypeData { perm:perm1, typ:typ1, .. }, ResolvedPermission::TypeData { perm:perm2, typ:typ2, .. })
            | (ResolvedPermission::TypeLit { perm:perm1, typ:typ1, .. }, ResolvedPermission::TypeLit { perm:perm2, typ:typ2, .. })
            => perm1 == perm2 && typ1 == typ2,
            (ResolvedPermission::FunSig { perm:perm1, fun:fun1, .. }, ResolvedPermission::FunSig { perm:perm2, fun:fun2, .. })
            => perm1 == perm2 && fun1 == fun2,
            _ => false
        }
    }
}

impl PartialOrd for ResolvedPermission {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ResolvedPermission {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (ResolvedPermission::TypeSig { perm:perm1, typ:typ1, .. }, ResolvedPermission::TypeSig { perm:perm2, typ:typ2, .. })
            | (ResolvedPermission::TypeData { perm:perm1, typ:typ1, .. }, ResolvedPermission::TypeData { perm:perm2, typ:typ2, .. })
            | (ResolvedPermission::TypeLit { perm:perm1, typ:typ1, .. }, ResolvedPermission::TypeLit { perm:perm2, typ:typ2, .. })
            => perm1.cmp(perm2).then_with(||typ1.cmp(typ2)),
            (ResolvedPermission::FunSig { perm:perm1, fun:fun1, .. }, ResolvedPermission::FunSig { perm:perm2, fun:fun2, .. })
            => perm1.cmp(perm2).then_with(||fun1.cmp(fun2)),
            (ResolvedPermission::TypeSig { .. }, _) => Ordering::Less,
            (_,ResolvedPermission::TypeSig { .. }) => Ordering::Greater,
            (ResolvedPermission::TypeData { .. }, _) => Ordering::Less,
            (_,ResolvedPermission::TypeData { .. }) => Ordering::Greater,
            (ResolvedPermission::TypeLit { .. }, _) => Ordering::Less,
            (_,ResolvedPermission::TypeLit { .. }) => Ordering::Greater,
            //(ResolvedPermission::FunSig { .. }, _) => Ordering::Less,
            //(_,ResolvedPermission::FunSig { .. }) => Ordering::Greater,
        }
    }
}

impl Hash for ResolvedPermission {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ResolvedPermission::TypeSig { perm, typ, .. } => {
                state.write_u8(0);
                state.write_u8(perm.0);
                typ.hash(state)
            },
            ResolvedPermission::TypeData { perm, typ, .. }=> {
                state.write_u8(1);
                state.write_u8(perm.0);
                typ.hash(state)
            },
            ResolvedPermission::TypeLit { perm, typ, .. }=> {
                state.write_u8(2);
                state.write_u8(perm.0);
                typ.hash(state)
            },
            ResolvedPermission::FunSig { perm, fun, .. }=> {
                state.write_u8(3);
                state.write_u8(perm.0);
                fun.hash(state)
            },
        }
    }
}