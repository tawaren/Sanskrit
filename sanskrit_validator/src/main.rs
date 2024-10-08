
extern crate sanskrit_common;
extern crate sanskrit_preloaded_validation;

extern crate core;

use std::{env};
use sanskrit_validator::execute_with_args;
use sanskrit_common::errors::*;
use sanskrit_preloaded_validation::{process_preloaded_deploy};


pub fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    match execute_with_args(&args[1..],|modules, transactions, dependencies, system_mode_on| process_preloaded_deploy(modules, transactions, dependencies, system_mode_on)) {
        Ok(h) => {
            println!("Validation succeeded for {} modules with {} open dependencies", h.modules.len(), h.open_dependencies.len());
            Ok(())
        }
        Err(_) => error(||"validation failed")
    }
}
