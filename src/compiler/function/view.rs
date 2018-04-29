use compiler::common::macros::view::*;
use compiler::common::view::*;
use compiler::common::types::*;
use compiler::errors::parsing::*;
use compiler::function::fields::*;


use lazycell::LazyCell;
use blake2_rfc::blake2b::{blake2b,Blake2bResult};


pub struct HeaderFunctionView<'a> {
    data: &'a [u8],
    pub params: FieldViewPointer<'a>,
    pub generics: PrivilegesViewPointer<'a>,
    pub module_imports: HashViewPointer<'a>,
    pub type_imports: ImportedTypePointer<'a>,
    function_hash_cache:LazyCell<Blake2bResult>,
}

pub struct BodyFunctionView<'a> {
    data: &'a [u8],
    pub module_imports: HashViewPointer<'a>,
    pub type_imports: ImportedTypePointer<'a>,
    pub function_imports: ImportedFunctionPointer<'a>,
    pub constructor_imports: ImportedConstructorsPointer<'a>,
    pub init_imports: ImportedInitPointer<'a>,

}

pub struct FunctionView<'a> {
    data: &'a [u8],
    pub header: HeaderFunctionView<'a>,
    pub body: Option<BodyFunctionView<'a>>,
    pub code: Option<&'a [u8]>,
    integrity_hash_cache:Option<LazyCell<Blake2bResult>>,
}

impl<'a> FunctionView<'a> {

    pub fn parse(data: &'a [u8], with_body:bool) -> Result<Self, ParsingError> {
        if data[main::MAGIC_NUMBER.start] != main::FUNCTION_MAGIC_NUMBER { return Result::Err(ParsingError::WrongMagicNumber { expected: main::FUNCTION_MAGIC_NUMBER, actual: data[main::MAGIC_NUMBER.start] }) }
        if data[main::VERSION.start] != PARSER_VERSION { return Result::Err(ParsingError::WrongInputVersion { expected: PARSER_VERSION, actual: data[main::VERSION.start] }) }

        let generics = PrivilegesViewPointer::create(data, main::DYNAMIC_PART_START, data[main::NUM_GENERICS.start]);
        let params = FieldViewPointer::create(data,generics.after(), data[main::NUM_PARAMS.start]);
        let module_imports = HashViewPointer::create(data,params.after(), data[main::NUM_MODULE_IMPORTS.start]);
        let type_imports = ImportedTypePointer::create(data,module_imports.after(), data[main::NUM_TYPE_IMPORTS.start]);

        if with_body {
            let Length(body_start) = Length::from_bytes(&data[main::HEADER_SIZE.start..])?;
            let Length(body_size) =  Length::from_bytes(&data[((body_start as usize) + body::BODY_SIZE.start) ..])?;

            let body_module_imports =  HashViewPointer::create(data,(body_start as usize) + body::DYNAMIC_PART_START, data[(body_start as usize) + body::NUM_MODULE_IMPORTS.start]);
            let body_type_imports = ImportedTypePointer::create(data,body_module_imports.after(), data[(body_start as usize) + body::NUM_TYPE_IMPORTS.start]);
            let body_function_imports = ImportedFunctionPointer::create(data,body_type_imports.after(), data[(body_start as usize) + body::NUM_FUN_IMPORTS.start]);
            let body_constructor_imports = ImportedConstructorsPointer::create(data,body_function_imports.after(), data[(body_start as usize) + body::NUM_CONSTRUCTOR_IMPORTS.start]);
            let body_init_imports = ImportedInitPointer::create(data,body_constructor_imports.after(), data[(body_start as usize) + body::NUM_INIT_IMPORTS.start]);


            let code_start = (body_start + body_size) as usize;
            let code = &data[code_start..];

            Result::Ok(FunctionView{
                data,
                header: HeaderFunctionView {
                    data,
                    params,
                    generics,
                    module_imports,
                    type_imports,
                    function_hash_cache:LazyCell::new(),
                },
                body: Some(BodyFunctionView {
                    data,
                    module_imports: body_module_imports,
                    type_imports: body_type_imports,
                    function_imports: body_function_imports,
                    constructor_imports: body_constructor_imports,
                    init_imports: body_init_imports,
                }),
                code: Some(code),
                integrity_hash_cache:Some(LazyCell::new()),
            })
        } else {
            Result::Ok(FunctionView{
                data,
                header: HeaderFunctionView {
                    data,
                    params,
                    generics,
                    module_imports,
                    type_imports,
                    function_hash_cache:LazyCell::new(),
                },
                body: None,
                code: None,
                integrity_hash_cache:None,
            })
        }


    }

    pub fn integrity_hash(&self) -> Result<Hash,ParsingError> {
        match self.integrity_hash_cache {
            Some(ref c) => Hash::from_blake(c.borrow_with(|| blake2b(HASH_SIZE, &[], &self.data[main::IDENTITY_HEADER_END..]))),
            None => Err(ParsingError::MissingBody)
        }
    }

    pub fn borrow_data(&self) -> &'a [u8]{
        self.data
    }

    pub fn extract_data(&self) -> Vec<u8>{
        let mut targ:Vec<u8> = vec![0 as u8; self.data.len()];
        targ.copy_from_slice(self.data);
        targ
    }
}

impl<'a> HeaderFunctionView<'a>{

    field_accessor!(main::VERSION,version,Version,Version);
    field_accessor!(main::FUN_INDEX,fun_index,MemberIndex,MemberIndex);
    field_accessor!(main::VISIBILITY,visibility,Visibility,Visibility);
    field_accessor!(main::EXECUTION_MODE,execution_mode,ExecutionMode,ExecutionMode);
    field_accessor!(main::OPTIMISATION_DECLARATION,declared_optimisation,OptimizationDeclaration,OptimizationDeclaration);
    field_accessor!(main::MODULE_HASH,module_hash,Hash,Hash);
    field_accessor!(main::RETURN_CONTROL,return_control,Control,Control);
    field_accessor!(main::RETURN_TYPE,return_type,TypeId,TypeId);
    field_accessor!(main::CODE_HASH,code_hash,Hash,Hash);


    pub fn function_hash(&self) -> Result<Hash,ParsingError>{
        Hash::from_blake(self.function_hash_cache.borrow_with(||blake2b(HASH_SIZE, &[], &self.data[0..main::IDENTITY_HEADER_END])))
    }

}
