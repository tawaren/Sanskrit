use compiler::common::types::*;
use compiler::common::parsing::*;
use compiler::typ::fields::*;
use test::inputgen::serializer::*;
use test::inputgen::import::type_import_builder::*;
use test::inputgen::import::function_import_builder::*;
use test::inputgen::import::type_body_import_builder::*;

#[derive(Clone)]
pub struct ConstructorCaseBuilder {
    data:Vec<u8>,
}


#[derive(Clone)]
pub struct TypeBuilder {
    data: Vec<u8>,
    types: Vec<(usize, TypeImportBuilder)>,
    ctrs: Vec<(usize, ConstructorCaseBuilder)>,
    functions: Vec<(usize, FunctionImportBuilder)>,
    ctr_imports: Vec<(usize, ConstructorsImportBuilder)>,
    init_imports: Vec<(usize, InitImportBuilder)>,
    body_start: u16,
    code_start: u16,
}

#[derive(Clone)]
pub struct HeaderData<'a> {
    pub module_hash:Hash<'a>,
    pub type_index:MemberIndex,
    pub max_privileges: Privileges,
    pub visibility:Visibility,
    pub kind:TypeKind,
    pub optimisation_declaration:OptimizationDeclaration,
}

impl TypeBuilder {

    //NOTE: THESE METHODS ARE DESIGNED TO BE CALLED IN ORDER

    pub fn new()->Self {
        let mut base:Vec<u8> = Vec::new();
        add_ser_at(&mut base,main::MAGIC_NUMBER.start, main::TYPE_MAGIC_NUMBER);
        add_ser_at(&mut base,main::VERSION.start, 0 as u8);
        TypeBuilder{
            data:base,
            types:Vec::new(),
            ctrs:Vec::new(),
            functions:Vec::new(),
            ctr_imports:Vec::new(),
            init_imports:Vec::new(),
            body_start:0,
            code_start:0
        }
    }

    pub fn add_fix_header_data(&mut self, data:HeaderData){
        let HeaderData {
            module_hash,
            type_index,
            max_privileges,
            visibility,
            kind,
            optimisation_declaration
        } = data;
        add_ser_at(&mut self.data,main::MODULE_HASH.start, module_hash);
        add_ser_at(&mut self.data,main::TYPE_INDEX.start, type_index);
        add_ser_at(&mut self.data,main::MAX_SUPPORTED_PRIVILEGES.start, max_privileges);
        add_ser_at(&mut self.data,main::VISIBILITY.start, visibility);
        add_ser_at(&mut self.data,main::KIND.start, kind);
        add_ser_at(&mut self.data,main::OPTIMISATION_DECLARATION.start, optimisation_declaration);
        //this will be incremented when added
        add_ser_at(&mut self.data,main::NUM_GENERICS.start, 0 as u8);
        add_ser_at(&mut self.data,main::NUM_MODULE_IMPORTS.start, 0 as u8);
        add_ser_at(&mut self.data,main::NUM_TYPE_IMPORTS.start, 0 as u8);
        add_ser_at(&mut self.data,main::NUM_CONSTRUCTORS.start, 0 as u8);
    }

    pub fn adapt_module_hash(&mut self, module_hash:Hash){
        add_ser_at(&mut self.data,main::MODULE_HASH.start, module_hash);
    }

    //Optional only with Cell types
    pub fn add_init_code(&mut self, init_hash:Hash){
        add_ser_at(&mut self.data,main::INIT_CODE_HASH.start, init_hash);
    }

    //repeatable
    pub fn add_generic(&mut self, data:Bound){
        self.data[main::NUM_GENERICS.start] += 1;
        push_ser(&mut self.data, data)
    }

    //NOT IN YET: Size Coefficients    (NUM_GENERICS+1     *   COEFFICIENT_SIZE)

    //repeatable
    pub fn add_module_import(&mut self, data:Hash){
        self.data[main::NUM_MODULE_IMPORTS.start] += 1;
        push_ser(&mut self.data, data)
    }

    //repeatable
    pub fn add_type_import(&mut self, data:TypeImportBuilder){
        self.data[main::NUM_TYPE_IMPORTS.start] += 1;
        let pos = self.data.len();    //Needed later to know placeholder pos
        push_ser(&mut self.data, Ptr(0));         //Placeholder for index
        self.types.push((pos,data))      //Will be placed later
    }

    //repeatable
    pub fn add_case(&mut self, data:ConstructorCaseBuilder){
        self.data[main::NUM_CONSTRUCTORS.start] += 1;
        let pos = self.data.len();    //Needed later to know placeholder pos
        push_ser(&mut self.data, Ptr(0));          //Placeholder for index
        self.ctrs.push((pos,data))      //Will be placed later
    }

    pub fn finish_header(&mut self, start_body:bool){

        for &(pos, ref elem)  in &self.types {
            let cur = self.data.len();
            assert!(cur <= <u16>::max_value() as usize);
            add_ser_at(&mut self.data,pos,Ptr(cur as u16));
            push_ser(&mut self.data,elem)
        }

        //will be reused by body
        self.types.clear();

        for &(pos, ref elem)  in &self.ctrs {
            let cur = self.data.len();
            assert!(cur <= <u16>::max_value() as usize);
            add_ser_at(&mut self.data,pos,Ptr(cur as u16));
            push_ser(&mut self.data,elem)
        }

        let h = self.data.len();
        self.body_start = h as u16;
        //Header Size needs to be calced
        add_ser_at(&mut self.data,main::HEADER_SIZE.start, Length(self.body_start));

        if start_body {
            add_ser_at(&mut self.data, (self.body_start as usize)+body::NUM_MODULE_IMPORTS.start, 0 as u8);
            add_ser_at(&mut self.data, (self.body_start as usize)+body::NUM_TYPE_IMPORTS.start, 0 as u8);
            add_ser_at(&mut self.data, (self.body_start as usize)+body::NUM_FUN_IMPORTS.start, 0 as u8);
            add_ser_at(&mut self.data, (self.body_start as usize)+body::NUM_CONSTRUCTOR_IMPORTS.start, 0 as u8);
            add_ser_at(&mut self.data, (self.body_start as usize)+body::NUM_INIT_IMPORTS.start, 0 as u8);
        }
    }

    //repeatable
    pub fn add_body_module_import(&mut self, data:Hash){
        self.data[(self.body_start as usize)+body::NUM_MODULE_IMPORTS.start] += 1;
        push_ser(&mut self.data, data)
    }

    //repeatable
    pub fn add_body_type_import(&mut self, data:TypeImportBuilder){
        self.data[(self.body_start as usize)+body::NUM_TYPE_IMPORTS.start] += 1;
        let pos = self.data.len();    //Needed later to know placeholder pos
        push_ser(&mut self.data,Ptr(0));         //Placeholder for index
        self.types.push((pos,data))      //Will be placed later
    }

    //repeatable
    pub fn add_function_import(&mut self, data:FunctionImportBuilder){
        self.data[(self.body_start as usize)+body::NUM_FUN_IMPORTS.start] += 1;
        let pos = self.data.len();    //Needed later to know placeholder pos
        push_ser(&mut self.data,Ptr(0));         //Placeholder for index
        self.functions.push((pos,data))      //Will be placed later
    }

    //repeatable
    pub fn add_constructors_import(&mut self, data:ConstructorsImportBuilder){
        self.data[(self.body_start as usize)+body::NUM_CONSTRUCTOR_IMPORTS.start] += 1;
        let pos = self.data.len();              //Needed later to know placeholder pos
        push_ser(&mut self.data,Ptr(0));         //Placeholder for index
        self.ctr_imports.push((pos,data))       //Will be placed later
    }

    //repeatable
    pub fn add_init_import(&mut self, data:InitImportBuilder){
        self.data[(self.body_start as usize)+body::NUM_INIT_IMPORTS.start] += 1;
        let pos = self.data.len();               //Needed later to know placeholder pos
        push_ser(&mut self.data,Ptr(0));         //Placeholder for index
        self.init_imports.push((pos,data))       //Will be placed later
    }

    pub fn finish_body(&mut self){
        for &(pos, ref elem)  in &self.types {
            let cur = self.data.len();
            assert!(cur <= <u16>::max_value() as usize);
            add_ser_at(&mut self.data,pos,Ptr(cur as u16));
            push_ser(&mut self.data,elem)
        }

        for &(pos, ref elem)  in &self.functions {
            let cur = self.data.len();
            assert!(cur <= <u16>::max_value() as usize);
            add_ser_at(&mut self.data,pos,Ptr(cur as u16));
            push_ser(&mut self.data,elem)
        }

        for &(pos, ref elem)  in &self.ctr_imports {
            let cur = self.data.len();
            assert!(cur <= <u16>::max_value() as usize);
            add_ser_at(&mut self.data,pos,Ptr(cur as u16));
            push_ser(&mut self.data,elem)
        }

        for &(pos, ref elem)  in &self.init_imports {
            let cur = self.data.len();
            assert!(cur <= <u16>::max_value() as usize);
            add_ser_at(&mut self.data,pos,Ptr(cur as u16));
            push_ser(&mut self.data,elem)
        }

        let s = self.data.len();
        self.code_start = s as u16;
        add_ser_at(&mut self.data, (self.body_start as usize)+body::BODY_SIZE.start, Length((s-(self.body_start as usize)) as u16));
    }

    pub fn add_code(&mut self, code:&[u8]){
        push_ser(&mut self.data,code)
    }

    pub fn extract(&self) -> Vec<u8>{
       self.data.to_owned()
    }
}


impl ConstructorCaseBuilder {

    pub fn new()->Self {
        let mut base:Vec<u8> = Vec::new();
        add_ser_at(&mut base, constructor::NUM_FIELDS.start, 0 as u8);
        ConstructorCaseBuilder{ data:base }
    }

    //repeatable
    pub fn add_field(&mut self, data:Field){
        self.data[constructor::NUM_FIELDS.start] += 1;
        push_ser(&mut self.data,data)
    }

}

impl<'a> Serializable for &'a ConstructorCaseBuilder {
    fn to_bytes(&self,data:&mut [u8], start:usize){
        data[start..(start+self.len())].copy_from_slice(&self.data[..])
    }
    fn len(&self) -> usize{
        self.data.len()
    }
}

