
use sanskrit_common::errors::*;
use sanskrit_common::capabilities::CapSet;
use model::*;
use sanskrit_common::model::Ptr;
use sanskrit_common::arena::*;
use sanskrit_interpreter::model::*;

impl<'a> RuntimeType<'a> {
    pub fn get_caps(&self) -> CapSet {
        match *self {
            RuntimeType::Image { .. } => CapSet::open(),
            RuntimeType::Custom { caps, .. } => caps,
            RuntimeType::NativeType { caps, .. } => caps,
            RuntimeType::NewType { .. } => CapSet::empty(),
            RuntimeType::AccountType { .. } =>  CapSet::empty(),
        }
    }
}

pub fn execute_type_builder<'a,'b,'h>(builder:&TypeBuilder<'b>, refs:&[Ptr<'a,RuntimeType<'a>>], alloc:&'a VirtualHeapArena<'h>) -> Result<Ptr<'a,RuntimeType<'a>>> {
    //the bool in the return specifies if it needs to be integrated into the caps
    fn execute_rec<'a,'b,'h>(cur:&TypeBuilder<'b>, refs:&[Ptr<'a,RuntimeType<'a>>], alloc:&'a VirtualHeapArena<'h>) -> Result<Ptr<'a,RuntimeType<'a>>> {
        match &*cur {
            TypeBuilder::Ref(TypeInputRef(idx)) => Ok(refs[*idx as usize]),
            TypeBuilder::Image(inner_builder) => alloc.alloc(RuntimeType::Image{
                typ: execute_rec(inner_builder,refs,alloc)?
            }),
            TypeBuilder::Dynamic(base_caps, kind, ref appls) => {
                let mut caps = *base_caps;
                let mut applies = alloc.slice_builder(appls.len())?;
                for (is_phantom,appl) in appls.iter() {
                    let typ = execute_rec(appl,refs, alloc)?;
                    if !*is_phantom {
                        caps = caps.intersect(typ.get_caps())
                    }
                    applies.push(typ)
                }
                caps = caps.union(base_caps.intersect(CapSet::non_recursive()));


                match *kind {
                    TypeKind::Custom { module, offset } => alloc.alloc(RuntimeType::Custom {
                        caps,
                        module,
                        offset,
                        applies: applies.finish()
                    }),
                    TypeKind::Native { typ } =>  alloc.alloc(RuntimeType::NativeType{
                        caps,
                        typ,
                        applies: applies.finish()
                    }),
                }
            },
        }
    }
    execute_rec(builder,refs, alloc)
}

