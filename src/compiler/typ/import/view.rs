use compiler::common::macros::view::*;
use compiler::common::view::*;
use compiler::common::types::*;
use compiler::errors::parsing::*;
use compiler::typ::fields::*;

pub struct ImportedTypeView<'a> {
    data: &'a [u8],
    start: usize,
    pub params: TypeIdViewPointer<'a>,
    //pub sizes: SizePolyViewPointer<'a>,

}


impl<'a> ImportedTypeView<'a> {

    pub fn create(data:&'a [u8],start:usize) -> Self {
        let params = TypeIdViewPointer::create(data, start + imported::DYNAMIC_PART_START, data[start + imported::NUM_TYPE_PARAMS.start]);
        //let sizes = SizePolyViewPointer::create(data, params.after(), data[start + imported::NUM_COEFFICIENTS.start]);
        ImportedTypeView {
            data,
            start,
            params,
            //sizes,
        }
    }

    embedded_field_accessor!(imported::MODULE_ID,declaring_module,ModuleId,ModuleId);
    embedded_field_accessor!(imported::VERSION,module_version,Version,Version);
    embedded_field_accessor!(imported::TYPE_INDEX,identifying_index,MemberIndex,MemberIndex);
    embedded_field_accessor!(imported::PRIVILEGES_DECLARATION,declared_privileges,Privileges,Privileges);
    embedded_field_accessor!(imported::KIND_DECLARATION,declared_kind,TypeKind,TypeKind);
    embedded_field_accessor!(imported::OPTIMISATION_DECLARATION,declared_optimisation,OptimizationDeclaration,OptimizationDeclaration);

}

pub struct ImportedConstructorsView<'a> {
    data: &'a [u8],
    start: usize,
    pub constructors: CtrViewPointer<'a>
}

impl<'a> ImportedConstructorsView<'a> {

    pub fn create(data:&'a [u8],start:usize) -> Self {
        let len = data[start + imported_constructors::NUM_CASES.start];
        ImportedConstructorsView {
            data,
            start,
            constructors: CtrViewPointer::create(data, start + imported_constructors::DYNAMIC_PART_START, len),
        }
    }

    embedded_field_accessor!(imported_constructors::TYPE_ID,coresponding_type,TypeId,TypeId);

}


pub struct ImportedInitView<'a> {
    data: &'a [u8],
    start: usize,
}

impl<'a> ImportedInitView<'a> {

    pub fn create(data:&'a [u8],start:usize) -> Self {
        ImportedInitView {
            data,
            start,
        }
    }

    embedded_field_accessor!(imported_init::TYPE_ID,coresponding_type,TypeId,TypeId);
    embedded_field_accessor!(imported_init::INIT_CODE_HASH,code_hash,Hash,Hash);
    embedded_field_accessor!(imported_init::INIT_RETURN_TYPE,init_return_type,TypeId,TypeId);

}

