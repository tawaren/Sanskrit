use compiler::common::types::*;
use compiler::common::parsing::*;
use compiler::module::fields::*;
use test::inputgen::serializer::*;

pub struct ModuleBuilder{
    data:Vec<u8>
}


impl ModuleBuilder {

    //NOTE: THESE METHODS ARE DESIGNED TO BE CALLED IN ORDER

    pub fn new()->Self {
        let mut base:Vec<u8> = Vec::new();
        add_ser_at(&mut base,main::MAGIC_NUMBER.start, main::MODULE_MAGIC_NUMBER);
        add_ser_at(&mut base,main::VERSION.start, 0 as u8);
        add_ser_at(&mut base,main::NUM_TYPES.start, 0 as u8);
        add_ser_at(&mut base,main::NUM_FUNCTIONS.start, 0 as u8);
        add_ser_at(&mut base,main::NUM_CONSTANTS.start, 0 as u8);
        ModuleBuilder{ data:base }
    }


    //repeatable
    pub fn add_type_import(&mut self, data:Hash){
        self.data[main::NUM_TYPES.start] += 1;
        push_ser(&mut self.data, data)
    }

    //repeatable
    pub fn add_function_import(&mut self, data:Hash){
        self.data[main::NUM_FUNCTIONS.start] += 1;
        push_ser(&mut self.data, data)
    }

    //repeatable
    pub fn add_constant_import(&mut self, data:Hash){
        self.data[main::NUM_CONSTANTS.start] += 1;
        push_ser(&mut self.data, data)
    }

    pub fn extract(&self) -> Vec<u8>{
        self.data.to_owned()
    }
}
