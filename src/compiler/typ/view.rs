use compiler::common::macros::view::*;
use compiler::common::view::*;
use compiler::common::types::*;
use compiler::errors::parsing::*;
use compiler::typ::fields::*;


use lazycell::LazyCell;
use blake2_rfc::blake2b::{blake2b,Blake2bResult};

pub struct HeaderTypeView<'a> {
    data: &'a [u8],
    pub generics: BoundViewPointer<'a>,
    //pub claimed_sizes: SizePolyViewPointer<'a>,
    pub module_imports: HashViewPointer<'a>,
    pub type_imports: ImportedTypePointer<'a>,
    pub constructors: CtrViewPointer<'a>,
    type_hash_cache:LazyCell<Blake2bResult>,
}

pub struct InitTypeView<'a> {
    data: &'a [u8],
    start: usize,
    pub module_imports: HashViewPointer<'a>,
    pub type_imports: ImportedTypePointer<'a>,
    pub function_imports: ImportedFunctionPointer<'a>,
    pub constructor_imports: ImportedConstructorsPointer<'a>,
    pub init_imports: ImportedInitPointer<'a>,

}


pub struct TypeView<'a> {
    pub data: &'a [u8],
    pub header: HeaderTypeView<'a>,
    pub body: Option<InitTypeView<'a>>,
    pub code: Option<&'a [u8]>,
    integrity_hash_cache:Option<LazyCell<Blake2bResult>>,
}


impl<'a> TypeView<'a> {

    pub fn parse(data: &'a [u8], with_body:bool) -> Result<Self,ParsingError> {
        if data[main::MAGIC_NUMBER.start] != main::TYPE_MAGIC_NUMBER {return Result::Err(ParsingError::WrongMagicNumber {expected:main::TYPE_MAGIC_NUMBER, actual:data[main::MAGIC_NUMBER.start]})}
        if data[main::VERSION.start] != PARSER_VERSION {return Result::Err(ParsingError::WrongInputVersion {expected:PARSER_VERSION, actual:data[main::VERSION.start]})}

        let has_init =  with_body && TypeKind::from_bytes(&data[main::KIND.start..(main::KIND.start+main::KIND.len)])? == TypeKind::Cell;
        let dynstart = if !has_init  {
            main::DYNAMIC_PART_START
        } else {
            main::DYNAMIC_PART_START_WITH_INIT
        };

        let generics = BoundViewPointer::create(data,dynstart, data[main::NUM_GENERICS.start]);
        //let claimed_sizes = SizePolyViewPointer::create(data,generics.after(), data[main::NUM_GENERICS.start]+1);
        let module_imports = HashViewPointer::create(data,generics.after(),data[main::NUM_MODULE_IMPORTS.start]);
        let type_imports = ImportedTypePointer::create(data,module_imports.after(),data[main::NUM_TYPE_IMPORTS.start]);
        let constructors = CtrViewPointer::create(data,type_imports.after(),data[main::NUM_CONSTRUCTORS.start]);

        if has_init {
            let Length(body_start) = Length::from_bytes(&data[main::HEADER_SIZE.start..])?;
            let Length(body_size) =  Length::from_bytes(&data[((body_start as usize) + body::BODY_SIZE.start)..])?;

            let body_module_imports =  HashViewPointer::create(data,(body_start as usize) + body::DYNAMIC_PART_START, data[(body_start as usize) + body::NUM_MODULE_IMPORTS.start]);
            let body_type_imports = ImportedTypePointer::create(data,body_module_imports.after(), data[(body_start as usize) + body::NUM_TYPE_IMPORTS.start]);
            let body_function_imports = ImportedFunctionPointer::create(data,body_type_imports.after(), data[(body_start as usize) + body::NUM_FUN_IMPORTS.start]);
            let body_constructor_imports = ImportedConstructorsPointer::create(data,body_function_imports.after(), data[(body_start as usize) + body::NUM_CONSTRUCTOR_IMPORTS.start]);
            let body_init_imports = ImportedInitPointer::create(data,body_constructor_imports.after(), data[(body_start as usize) + body::NUM_INIT_IMPORTS.start]);


            let code_start = (body_start + body_size) as usize;
            let code = &data[code_start..];

            Result::Ok(TypeView{
                data,
                header: HeaderTypeView {
                    data,
                    generics,
                    //claimed_sizes,
                    module_imports,
                    type_imports,
                    constructors,
                    type_hash_cache:LazyCell::new(),
                },
                body: Some(InitTypeView {
                    data,
                    start: body_start as usize,
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
            Result::Ok(TypeView{
                data,
                header: HeaderTypeView {
                    data,
                    generics,
                    //claimed_sizes,
                    module_imports,
                    type_imports,
                    constructors,
                    type_hash_cache:LazyCell::new(),
                },
                body: None,
                code: None,
                integrity_hash_cache:if !with_body {
                    None
                } else {
                    Some(LazyCell::new())
                },
            })
        }
    }

    pub fn integrity_hash(&self) -> Result<Hash,ParsingError>{
        match self.integrity_hash_cache {
            Some(ref c) => Hash::from_blake(c.borrow_with(||blake2b(HASH_SIZE, &[], &self.data[..]))),
            None => Err(ParsingError::MissingBody)
        }
    }

    pub fn borrow_data(&self) -> &'a [u8]{
        self.data
    }

    pub fn extract_data(&self) -> Vec<u8>{
        let mut targ:Vec<u8> = Vec::with_capacity(self.data.len());
        targ.copy_from_slice(self.data);
        targ
    }
}

impl<'a> HeaderTypeView<'a> {

    field_accessor!(main::TYPE_INDEX,type_index,MemberIndex,MemberIndex);
    field_accessor!(main::MAX_SUPPORTED_PRIVILEGES,max_supported_privileges,Privileges,Privileges);
    field_accessor!(main::VISIBILITY,visibility,Visibility,Visibility);
    field_accessor!(main::KIND,kind,TypeKind,TypeKind);
    field_accessor!(main::OPTIMISATION_DECLARATION,declared_optimisation,OptimizationDeclaration,OptimizationDeclaration);
    field_accessor!(main::VERSION,version,Version,Version);
    field_accessor!(main::MODULE_HASH,module_hash,Hash,Hash);

    //TODO: Make Optional
    field_accessor!(main::INIT_CODE_HASH,code_hash,Hash,Hash);


    pub fn type_hash(&self) -> Result<Hash,ParsingError>{
        Hash::from_blake(self.type_hash_cache.borrow_with(||blake2b(HASH_SIZE, &[], &self.data[0..main::IDENTITY_HEADER_END])))
    }
}


pub struct CtrView<'a> {
    data: &'a [u8],
    pub params: FieldViewPointer<'a>,
}

impl<'a> CtrView<'a> {
    pub fn create(data:&'a [u8],start:usize) -> Self {
        let len = data[start + constructor::NUM_FIELDS.start];
        CtrView {
            data,
            params: FieldViewPointer::create(data,start + constructor::DYNAMIC_PART_START, len),
        }
    }
}

