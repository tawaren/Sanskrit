use compiler::common::types::*;
use compiler::common::parsing::*;
use compiler::function::fields::imported::*;
use test::inputgen::serializer::*;


pub struct FunctionImportBuilder {
    data:Vec<u8>,
}

pub struct FunctionImportData<'a> {
    pub module_id:ModuleId,
    pub fun_index:MemberIndex,
    pub optimisation_declaration:OptimizationDeclaration,
    pub return_type:TypeId,
    pub return_control:Control,
    pub code_hash:Hash<'a>
}

impl FunctionImportBuilder {

    pub fn new()->Self {
        FunctionImportBuilder{ data:Vec::new() }
    }

    pub fn add_fix_header_data(&mut self, data: FunctionImportData){
        let FunctionImportData {
            module_id,
            fun_index,
            optimisation_declaration,
            return_type,
            return_control,
            code_hash
        } = data;
        //todo: place sep file
        add_ser_at(&mut self.data,MODULE_ID.start, module_id);
        add_ser_at(&mut self.data,FUN_INDEX.start, fun_index);
        add_ser_at(&mut self.data,OPTIMISATION_DECLARATION.start, optimisation_declaration);
        add_ser_at(&mut self.data,RETURN_TYPE.start, return_type);
        add_ser_at(&mut self.data,RETURN_CONTROL.start,  return_control);
        add_ser_at(&mut self.data,CODE_HASH.start, code_hash);
        //this will be incremented when added
        add_ser_at(&mut self.data,NUM_GENERIC_PARAMS.start, 0 as u8);
        add_ser_at(&mut self.data,NUM_PARAMS.start, 0 as u8); //Will stay 0 not yet in
    }

    //repeatable
    pub fn add_type_apply(&mut self, data:TypeId){
        self.data[NUM_GENERIC_PARAMS.start] += 1;
        push_ser(&mut self.data,data)
    }

    //repeatable
    pub fn add_value_apply(&mut self, data:Field){
        self.data[NUM_PARAMS.start] += 1;
        push_ser(&mut self.data,data)
    }
}

impl<'a> Serializable for &'a FunctionImportBuilder {
    fn to_bytes(&self,data:&mut [u8], start:usize){
        data[start..(start+self.len())].copy_from_slice(&self.data[..])
    }
    fn len(&self) -> usize{
        self.data.len()
    }
}