#![feature(nll)]

//extern crate pwasm_std;

extern crate sanskrit_deploy;
extern crate sanskrit_compile;
extern crate sanskrit_common;
extern crate sanskrit_core;
extern crate sanskrit_memory_store;
extern crate sanskrit_interpreter;
extern crate core;
#[macro_use]
extern crate lazy_static;

use sanskrit_memory_store::BTreeMapStore;
use sanskrit_deploy::{deploy_module, deploy_function};
use sanskrit_common::errors::*;

use std::mem;
use std::sync::Mutex;
use sanskrit_common::store::{StorageClass, Store};
use std::cell::Cell;
use sanskrit_compile::limiter::Limiter;
use sanskrit_core::accounting::Accounting;
use sanskrit_compile::compile_function;
use externals::ServerExternals;
use store::ExternalStore;

mod externals;
mod store;

extern  {
    fn load_input(ptr: *mut u8);

    //later we could pre check this and not even call the deployer
    fn contains(class:u8, ptr_key_hash: *const u8) -> bool;

    //if return is positive we assume that it suceeded and that many bytes where llocated
    //if return is negative we assume that we did not had enough space and allocate that many space then reexecute load
    fn load(class:u8, ptr_key_hash: *const u8, target_ptr: *mut u8, reserved_space:usize) -> isize;

    //As a later improvement we do not have to pass input back out as there are known outside
    //fn store_input(ptr_key_hash: *u8);

    fn store(class:u8, ptr_key_hash: *const u8, ptr_data: *const u8, ptr_data_size:usize ) -> bool;

    //Note: currently hashes of externals are in memory and only set when deploying
    //      later we need to persist this info outside of wasm and refill memory on startup
}

#[no_mangle]
pub extern fn call(input_size: usize, store_prealloc:usize, is_txt:bool, system_mode:bool, system_id:isize) -> bool {
    let mut data = Vec::with_capacity(input_size);
    data.resize(input_size, 0);
    unsafe{load_input(data.as_mut_ptr())}
    if is_txt {
        match process_txt_deploy(data, store_prealloc) {
            Ok(_) => true,
            Err(val) => false
        }
    } else {
        match process_module_deploy(data,store_prealloc, system_mode, system_id) {
            Ok(_) => true,
            Err(_) => false
        }
    }
}

fn process_module_deploy(module:Vec<u8>, pre_alloc:usize, system_mode_on:bool, sys_id:isize) -> Result<()>{
    let store = ExternalStore::new(pre_alloc);
    //later we get rid of that when we fall back to gas
    let accounting = max_accounting(module.len());
    //Note: We use a special store that commits eagerly (for efficiency and thus does not implement commit - hence disable auto commit)
    //      This is ok as in this case storing is the last thing that would happen and we store only the module
    let hash = deploy_module(&store,&accounting,module.clone(),system_mode_on,false)?;
    if system_mode_on && sys_id >= 0 {
        let sys_impl = externals::SYS_MODS[sys_id as usize];
        sys_impl(hash.clone());
    }
    Ok(())
}

//todo: we need to return result
//      later we can just keep store external forwarding calls to the user
//      but this may not be that easy -- bu needs to be done
fn process_txt_deploy(txt:Vec<u8>, pre_alloc:usize) -> Result<Vec<u8>> {
    let store = ExternalStore::new(pre_alloc);
    //later we get rid of that when we fall back to gas
    let accounting = max_accounting(txt.len());
    let limiter = max_limiter();
    //Note: We use a special store that commits eagerly (for efficiency and thus does not implement commit - hence disable auto commit)
    //      Note if we fail after deploy we still can record the deploy and only repeat the compile
    //       Later -- we can even make seperate wasm entry points for these
    let hash = deploy_function(&store,&accounting,txt.clone(),false)?;
    let (t_hash,_) = compile_function::<_,ServerExternals>(&store, &accounting,&limiter,hash, false)?;
    let res = store.get(StorageClass::Descriptor, &t_hash, |d|d.to_vec())?;
    Ok(res)
}

fn max_accounting(input_limit:usize) -> Accounting{
    Accounting{
        load_byte_budget: Cell::new(usize::max_value()),
        store_byte_budget: Cell::new(usize::max_value()),
        process_byte_budget: Cell::new(usize::max_value()),
        stack_elem_budget: Cell::new(usize::max_value()),
        //these two are a bit counter intuitive
        max_nesting: Cell::new(0),
        nesting_limit: usize::max_value(),
        input_limit
    }
}


fn max_limiter() -> Limiter {
    Limiter {
        max_functions: usize::max_value(),
        max_nesting: usize::max_value(),
        max_used_nesting: Cell::new(0),
        produced_functions: Cell::new(0)
    }
}
