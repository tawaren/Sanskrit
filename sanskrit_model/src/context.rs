use model::*;
use sanskrit_common::model::*;

//A context that makes available the Elements defined in the input
#[derive(Copy, Clone, Debug)]
pub struct InputContext<'a> {
    module:ModuleLink,                      //The module the context comes from
    import:&'a PublicImport,                //The plain import section
    functions:Option<&'a [FunctionImport]>, //the function imports if avaiable
}

impl<'a>  InputContext<'a> {

    //Generates the Context from an ADT
    pub fn from_adt<'b:'a>(tpc:&'b AdtComponent, module:ModuleLink) -> Self {
        InputContext {
            module,                              //Provides the this module
            import: &tpc.import,                 //Provides the Imports
            functions: None,
        }
    }

    //Generates the Context from an Function
    pub fn from_function<'b:'a>(tpc:&'b FunctionComponent, module:ModuleLink) -> Self {
        InputContext {
            module,                                 //Provides the this module
            import: &tpc.import.base,               //Provides the Imports
            functions: Some(&tpc.import.functions), //Provides the Functions
        }
    }

    //Generates the Context from an imported Function
    pub fn from_fun_import<'b:'a>(imp:&'b FunctionComponent, module:ModuleLink) -> Self{
        InputContext {
            module,                                 //Provides the this module
            import: &imp.import.base,               //Provides the Imports
            functions: None,                        //we do not care about the functions when we do an import
        }
    }

    //Generates the Context from an imported Adt
    pub fn from_adt_import<'b:'a>(imp:&'b AdtComponent, module:ModuleLink) -> Self{
        Self::from_adt(imp,module)  //Same as top level
    }

    //Functions to get the number of modules and the module Links if they exist
    pub fn num_modules(&'a self) -> usize { self.import.modules.len()+1 }
    pub fn get_module(&'a self, offset: u8) -> Option<&'a ModuleLink> {
        if offset == 0 {return Some(&self.module)}
        self.import.modules.get((offset -1) as usize)
    }

    //Functions to get the number of types and the type if they exist
    pub fn num_types(&'a self) -> usize { self.import.types.len() }
    pub fn get_type(&'a self, offset: u8) -> Option<&'a Type> { self.import.types.get(offset as usize) }


    //Functions to get the number of functions and the function if they exist
    pub fn num_function_imports(&'a self) -> usize { self.functions.map_or(0,|f|f.len()) }
    pub fn get_function_import(&'a self, offset: u8) -> Option<&'a FunctionImport> { self.functions.and_then(|c|c.get(offset as usize)) }

    //Functions to get the number of errors and the error if they exist
    pub fn num_error_imports(&'a self) -> usize{ self.import.errors.len() }
    pub fn get_error_import(&'a self, offset: u8) -> Option<&'a ErrorImport> { self.import.errors.get(offset as usize) }

}
