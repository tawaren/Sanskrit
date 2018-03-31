use compiler::common::types::*;
use compiler::common::parsing::*;
use compiler::typ::fields::*;
use test::inputgen::serializer::*;
use test::inputgen::type_builder::*;
use compiler::common::macros::view::Deserializer;

pub struct ConstructorsImportBuilder {
    data:Vec<u8>,
    ctrs:Vec<(usize,ConstructorCaseBuilder)>,
}

pub struct InitImportBuilder {
    data:Vec<u8>,
}

impl ConstructorsImportBuilder {

    pub fn new()->Self {
        ConstructorsImportBuilder { data: Vec::new(), ctrs: Vec::new() }
    }

    pub fn corresponding_type(&mut self, typ:TypeId){
        add_ser_at(&mut self.data,imported_constructors::TYPE_ID.start, typ);
        add_ser_at(&mut self.data,imported_constructors::NUM_CASES.start, 0 as u8);
    }

    //repeatable
    pub fn add_constructor_case(&mut self, data:ConstructorCaseBuilder){
        self.data[imported_constructors::NUM_CASES.start] += 1;
        let pos = self.data.len();               //Needed later to know placeholder pos
        push_ser(&mut self.data,Ptr(0));                  //Placeholder for index
        self.ctrs.push((pos,data))       //Will be placed later
    }

    pub fn finish(&mut self){
        for &(pos, ref elem)  in &self.ctrs {
            let cur = self.data.len();
            assert!(cur < <u16>::max_value() as usize);
            add_ser_at(&mut self.data,pos,Ptr(cur as u16));
            push_ser(&mut self.data,elem)
        }
    }
}

impl<'a> Serializable for &'a ConstructorsImportBuilder {
    fn to_bytes(&self,data:&mut [u8], start:usize){
        data[start..(start +self.len())].copy_from_slice(&self.data[..]);
        for &(pos, ref elem)  in &self.ctrs {
            let Ptr(val) = Ptr::from_bytes(&data[(start+pos)..]).unwrap();
            Ptr((start+(val as usize)) as u16).to_bytes(&mut data[..(start + pos + elem.len())],start + pos);
        }
    }

    fn len(&self) -> usize{
        self.data.len()
    }
}

impl InitImportBuilder {

    pub fn new()->Self {
        InitImportBuilder { data: Vec::new() }
    }

    pub fn corresponding_type(&mut self, typ:TypeId){
        add_ser_at(&mut self.data,imported_init::TYPE_ID.start, typ);
    }

    pub fn return_type(&mut self, typ:TypeId){
        add_ser_at(&mut self.data,imported_init::INIT_RETURN_TYPE.start, typ);
    }

    pub fn add_init_code(&mut self, data:Hash){
        add_ser_at(&mut self.data,imported_init::INIT_CODE_HASH.start, data);
    }

}

impl<'a> Serializable for &'a InitImportBuilder {
    fn to_bytes(&self,data:&mut [u8], start:usize){
        data[start..(start+self.len())].copy_from_slice(&self.data[..])
    }
    fn len(&self) -> usize{
        self.data.len()
    }
}