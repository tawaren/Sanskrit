use compiler::common::macros::view::*;
use compiler::common::view::*;
use compiler::common::types::*;
use compiler::errors::parsing::*;

use compiler::function::fields::imported::*;


pub struct ImportedFunctionView<'a> {
    data: &'a [u8],
    start: usize,
    pub generic_params: TypeIdViewPointer<'a>,
    pub params: FieldViewPointer<'a>,
}

impl<'a> ImportedFunctionView<'a> {
    pub fn create(data:&'a [u8],start:usize) -> Self {
        let generic_params = TypeIdViewPointer::create(data, start + DYNAMIC_PART_START, data[start + NUM_GENERIC_PARAMS.start]);
        let params = FieldViewPointer::create(data, generic_params.after(), data[start + NUM_PARAMS.start]);
        ImportedFunctionView {
            data,
            start,
            generic_params,
            params,
        }
    }

    embedded_field_accessor!(MODULE_ID,declaring_module,ModuleId,ModuleId);
    embedded_field_accessor!(VERSION,version,Version,Version);
    embedded_field_accessor!(FUN_INDEX,identifying_index,MemberIndex,MemberIndex);
    embedded_field_accessor!(OPTIMISATION_DECLARATION,declared_optimisation,OptimizationDeclaration,OptimizationDeclaration);
    embedded_field_accessor!(RETURN_CONTROL,return_control,Control,Control);
    embedded_field_accessor!(RETURN_TYPE,return_type,TypeId,TypeId);
    embedded_field_accessor!(CODE_HASH,code_hash,Hash,Hash);

}