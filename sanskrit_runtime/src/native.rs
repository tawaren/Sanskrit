use sanskrit_common::errors::*;
use sanskrit_common::model::*;
use sanskrit_common::capabilities::CapSet;
use model::NativeAdtType;
use model::*;
use sanskrit_common::arena::*;
use sanskrit_common::encoding::ParserAllocator;

//lit types do not have generics
fn resolve_runtime_leaf_type<'a,'h>(typ: NativeType, applies:SlicePtr<'a,Ptr<'a,RuntimeType<'a>>>, alloc:&'a VirtualHeapArena<'h>) ->  Result<Ptr<'a,RuntimeType<'a>>>{
    //build the type (consider generics where required)
    alloc.alloc(RuntimeType::NativeType {
        //get the base caps (does not consider generics, not necessary as literals have none)
        caps: typ.base_caps(),
        typ,
        applies
    })
}

//create a runtime type for a native type
pub fn to_runtime_type<'a,'b,'h>(typ:NativeType, applies:SlicePtr<'a,Ptr<'a,RuntimeType<'a>>>, alloc:&'a VirtualHeapArena<'h>, code_alloc:&'b VirtualHeapArena) -> Result<Ptr<'a,RuntimeType<'a>>> {
    match typ {
        NativeType::Data(_)
        | NativeType::SInt(_)
        | NativeType::UInt(_)
        | NativeType::Context
        | NativeType::Index
        | NativeType::Ref
        | NativeType::Unique => {
            //these types hav eno generics
            if !applies.is_empty() {
                generic_args_mismatch()
            } else {
                //they behave like a leaf type
                resolve_runtime_leaf_type(typ, applies, alloc)
            }
        },
        NativeType::Singleton  => {
            //singleton has one generic but it phantom
            if applies.len() != 1 {
                generic_args_mismatch()
            } else {
                //it behaves like a leaf type (because param is phantom)
                resolve_runtime_leaf_type(typ, applies,alloc)
            }
        },
        NativeType::Bool => {
            //Bools have no generics but an adt descriptor that can be used
            if applies.len() != 0 {
                generic_args_mismatch()
            } else {
                //Construct over descriptor
                NativeAdtType::Bool.get_native_adt_descriptor(code_alloc)?.build_type(applies, alloc)
            }
        },
        NativeType::Tuple(size) => {
            //tuple has one generic per field
            if applies.len() != size as usize {
                generic_args_mismatch()
            } else {
                //Construct over descriptor
                NativeAdtType::Tuple(size).get_native_adt_descriptor(code_alloc)?.build_type(applies, alloc)
            }
        },
        NativeType::Alternative(size) => {
            //alternative has one generic per variant
            if applies.len() != size as usize {
                generic_args_mismatch()
            } else {
                //Construct over descriptor
                NativeAdtType::Alternative(size).get_native_adt_descriptor(code_alloc)?.build_type(applies, alloc)
            }
        },
    }
}


impl NativeAdtType {
    pub fn get_native_adt_descriptor<'b,'h>(&self, alloc:&'b VirtualHeapArena<'h>) -> Result<AdtDescriptor<'b>> {
        Ok(match *self {
            NativeAdtType::Tuple(apply_size) => AdtDescriptor {
                //A Tuple has non-phatnom generics (one per field)
                generics: alloc.repeated_slice(TypeTypeParam(false, CapSet::from_cap(NativeCap::Embed)),apply_size as usize)?,
                //A tuple has 1 ctr with 1 field per generic
                constructors: alloc.copy_alloc_slice(&[alloc.iter_alloc_slice((0..apply_size).map(|i|TypeBuilder::Ref(TypeInputRef(i as u8))))?])?,
                base_caps: NativeType::Tuple(apply_size).base_caps(),
                id: AdtId::Native(NativeType::Tuple(apply_size))
            },

            NativeAdtType::Alternative(apply_size)  => AdtDescriptor {
                //Alternatives have 1 generic pre ctr
                generics: alloc.repeated_slice(TypeTypeParam(false, CapSet::from_cap(NativeCap::Embed)), apply_size as usize)?,
                //Alternatives have 1 ctr per generic
                constructors: alloc.iter_result_alloc_slice((0..apply_size).map(|i|alloc.copy_alloc_slice(&[TypeBuilder::Ref(TypeInputRef(i as u8))])))?,
                base_caps: NativeType::Alternative(apply_size).base_caps(),
                id: AdtId::Native(NativeType::Alternative(apply_size))
            },

            NativeAdtType::Bool => AdtDescriptor {
                //Bools have no generics
                generics: SlicePtr::empty(),
                //bools have 2 ctrs with no fields
                constructors: alloc.copy_alloc_slice(&[SlicePtr::empty();2])?,
                base_caps: NativeType::Bool.base_caps(),
                id: AdtId::Native(NativeType::Bool)
            }
        })
    }
}

impl LitDesc {

    pub fn lit_typ<'a,'h>(&self, size:u16, alloc:&'a VirtualHeapArena<'h>) -> Result<Ptr<'a,RuntimeType<'a>>>{
        match *self {
            LitDesc::Ref => {
                if size != 20 {return literal_data_error()}
                resolve_runtime_leaf_type(NativeType::Ref, SlicePtr::empty(), alloc)
            },
            LitDesc::Data => resolve_runtime_leaf_type(NativeType::Data(size), SlicePtr::empty(), alloc),
            LitDesc::I8 => {
                if size != 1 {return literal_data_error()}
                resolve_runtime_leaf_type(NativeType::SInt(1), SlicePtr::empty(), alloc)
            },
            LitDesc::U8 => {
                if size != 1 {return literal_data_error()}
                resolve_runtime_leaf_type(NativeType::UInt(1), SlicePtr::empty(), alloc)
            },
            LitDesc::I16 => {
                if size != 2 {return literal_data_error()}
                resolve_runtime_leaf_type(NativeType::SInt(2), SlicePtr::empty(), alloc)
            },
            LitDesc::U16 => {
                if size != 2 {return literal_data_error()}
                resolve_runtime_leaf_type(NativeType::UInt(2), SlicePtr::empty(), alloc)
            },
            LitDesc::I32 => {
                if size != 4 {return literal_data_error()}
                resolve_runtime_leaf_type(NativeType::SInt(4), SlicePtr::empty(), alloc)
            },
            LitDesc::U32 => {
                if size != 4 {return literal_data_error()}
                resolve_runtime_leaf_type(NativeType::UInt(4), SlicePtr::empty(), alloc)
            },
            LitDesc::I64 => {
                if size != 8 {return literal_data_error()}
                resolve_runtime_leaf_type(NativeType::SInt(8), SlicePtr::empty(), alloc)
            },
            LitDesc::U64 => {
                if size != 8 {return literal_data_error()}
                resolve_runtime_leaf_type(NativeType::UInt(8), SlicePtr::empty(), alloc)
            },
            LitDesc::I128 => {
                if size != 16 {return literal_data_error()}
                resolve_runtime_leaf_type(NativeType::SInt(16), SlicePtr::empty(), alloc)
            },
            LitDesc::U128 => {
                if size != 16 {return literal_data_error()}
                resolve_runtime_leaf_type(NativeType::UInt(16), SlicePtr::empty(), alloc)
            },
        }
    }
}