use compiler::common::macros::view::*;
use compiler::errors::parsing::*;
use compiler::common::view::*;
use compiler::common::types::*;
use compiler::module::fields::main::*;

use lazycell::LazyCell;
use blake2_rfc::blake2b::{blake2b,Blake2bResult};

pub struct ModuleView<'a> {
    data:&'a [u8],
    pub types:HashViewPointer<'a>,
    pub functions:HashViewPointer<'a>,
    pub constants:HashViewPointer<'a>,
    module_hash_cache:LazyCell<Blake2bResult>,
}

impl<'a> ModuleView<'a> {
    pub fn parse(data:&'a [u8]) -> Result<Self,ParsingError>{
        if data[MAGIC_NUMBER.start] != MODULE_MAGIC_NUMBER {return Result::Err(ParsingError::WrongMagicNumber {expected:MODULE_MAGIC_NUMBER, actual:data[0]})}
        if data[VERSION.start] != PARSER_VERSION {return Result::Err(ParsingError::WrongInputVersion {expected:PARSER_VERSION, actual:data[1]})}
        let types = HashViewPointer::create(data,DYNAMIC_PART_START,data[NUM_TYPES.start]);
        let functions = HashViewPointer::create(data,types.after(),data[NUM_FUNCTIONS.start]);
        let constants = HashViewPointer::create(data,functions.after(),data[NUM_CONSTANTS.start]);

        Result::Ok(ModuleView {
            data: &data,
            types,
            functions,
            constants,
            module_hash_cache:LazyCell::new()
        })

    }

    field_accessor!(VERSION,version,Version,Version);

    pub fn module_hash(&self) -> Result<Hash,ParsingError>{
        Hash::from_blake(self.module_hash_cache.borrow_with(||blake2b(HASH_SIZE, &[], &self.data[..])))
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
