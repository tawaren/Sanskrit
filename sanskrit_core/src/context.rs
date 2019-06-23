use model::*;
use sanskrit_common::model::*;

//A context that makes available the Elements defined in the input
#[derive(Copy, Clone, Debug)]
pub struct InputContext<'a> {
    module:Option<ModuleLink>,              //The module the context comes from (None for top level funcs)
    import:&'a PublicImport,                //The plain import section
    functions:Option<&'a [FunctionImport]>, //the function imports if avaiable
}

impl<'a>  InputContext<'a> {

    //Generates the Context from an ADT
    pub fn from_data<'b:'a>(tpc:&'b DataComponent, module:ModuleLink) -> Self {
        InputContext {
            module:Some(module),                 //Provides the this module
            import: &tpc.import,                 //Provides the Imports
            functions: None,
        }
    }

    //Generates the Context from an Function
    pub fn from_function<'b:'a>(tpc:&'b FunctionComponent, module:Option<ModuleLink>) -> Self {
        InputContext {
            module,                                 //Provides the this module
            import: &tpc.shared.import,             //Provides the Imports
            functions: match tpc.body {             //Provides the Functions
                FunctionImpl::External(_) => None,
                FunctionImpl::Internal { ref functions, .. } => Some(&functions),
            }
        }
    }

    //Generates the Context from an imported Function
    pub fn from_sig<'b:'a>(imp:&'b SigComponent, module:ModuleLink) -> Self{
        InputContext {
            module:Some(module),                    //Provides the this module
            import: &imp.shared.import,             //Provides the Imports
            functions: None,
        }
    }

    //Generates the Context from an imported Function
    pub fn from_fun_import<'b:'a>(imp:&'b FunSigShared, module:ModuleLink) -> Self{
        InputContext {
            module:Some(module),                    //Provides the this module
            import: &imp.import,                    //Provides the Imports
            functions: None,                        //we do not care about the functions when we do an import
        }
    }

    //Generates the Context from an imported Adt
    pub fn from_data_import<'b:'a>(imp:&'b DataComponent, module:ModuleLink) -> Self{
        Self::from_data(imp, module)  //Same as top level
    }

    //Generates the Context from an imported Adt
    pub fn from_sig_import<'b:'a>(imp:&'b SigComponent, module:ModuleLink) -> Self{
        Self::from_fun_import(&imp.shared,module)  //Same as top level
    }

    //Functions to get the number of modules and the module Links if they exist
    pub fn num_modules(&'a self) -> usize { self.import.modules.len()+1 }
    pub fn get_module(&'a self, offset: u8) -> Option<&'a ModuleLink> {
        match self.module {
            None => self.import.modules.get((offset) as usize),
            Some(ref m) => {
                if offset == 0 {return Some(m)}
                self.import.modules.get((offset -1) as usize)
            },
        }
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
