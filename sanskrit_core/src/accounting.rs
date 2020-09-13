
use sanskrit_common::errors::*;
use core::cell::Cell;


pub struct Accounting {
    pub load_byte_budget:Cell<usize>,
    pub store_byte_budget:Cell<usize>,
    pub process_byte_budget:Cell<usize>,
    pub stack_elem_budget:Cell<usize>,
    //only used for discovering nesting_limit
    pub max_nesting:Cell<usize>,
    pub nesting_limit:usize,
    pub input_limit:usize,
}

impl Accounting {
    pub fn process_bytes(&self, bytes:usize) -> Result<()> {
        checked_subtract(&self.process_byte_budget, bytes)
    }

    pub fn load_bytes(&self, bytes:usize) -> Result<()> {
        checked_subtract(&self.load_byte_budget, bytes)
    }

    pub fn store_bytes(&self, bytes:usize) -> Result<()> {
        checked_subtract(&self.store_byte_budget, bytes)
    }

    pub fn stack_elems(&self, elems:usize) -> Result<()> {
        checked_subtract(&self.stack_elem_budget, elems)
    }
}

pub fn checked_subtract(cell:&Cell<usize>, amount:usize) -> Result<()> {
    if cell.get() < amount {
        error(||"budget exhausted")
    } else {
        cell.set(cell.get()-amount);
        Ok(())
    }
}

