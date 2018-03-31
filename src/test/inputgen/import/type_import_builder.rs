use compiler::common::types::*;
use compiler::common::parsing::*;
use compiler::typ::fields::imported::*;
use test::inputgen::serializer::*;

pub struct TypeImportBuilder {
    data:Vec<u8>,
}

pub struct TypeImportData {
    pub module_id:ModuleId,
    pub type_index:MemberIndex,
    pub optimisation_declaration:OptimizationDeclaration,
    pub privileges_declaration: Privileges,
    pub kind_declaration:TypeKind,
}


impl TypeImportBuilder {

    pub fn new()->Self {
        let mut base:Vec<u8> = Vec::new();
        add_ser_at(&mut base,VERSION.start, 0 as u8);
        TypeImportBuilder{ data:base }
    }

    pub fn add_fix_header_data(&mut self, data: TypeImportData){
        let TypeImportData {
            module_id,
            type_index,
            optimisation_declaration,
            privileges_declaration,
            kind_declaration
        } = data;
        add_ser_at(&mut self.data,MODULE_ID.start, module_id);
        add_ser_at(&mut self.data,TYPE_INDEX.start, type_index);
        add_ser_at(&mut self.data,OPTIMISATION_DECLARATION.start, optimisation_declaration);
        add_ser_at(&mut self.data,PRIVILEGES_DECLARATION.start, privileges_declaration);
        add_ser_at(&mut self.data,KIND_DECLARATION.start, kind_declaration);
        //this will be incremented when added
        add_ser_at(&mut self.data,NUM_TYPE_PARAMS.start, 0 as u8);
        //add_ser_at(&mut self.data,NUM_COEFFICIENTS.start, 0 as u8); //Will stay 0 not yet in
    }

    //repeatable
    pub fn add_type_apply(&mut self, data:TypeId){
        self.data[NUM_TYPE_PARAMS.start] += 1;
        push_ser(&mut self.data,data)
    }


}

impl<'a> Serializable for &'a TypeImportBuilder {
    fn to_bytes(&self,data:&mut [u8], start:usize){
        data[start..(start + self.len())].copy_from_slice(&self.data[..])
    }
    fn len(&self) -> usize{
        self.data.len()
    }
}