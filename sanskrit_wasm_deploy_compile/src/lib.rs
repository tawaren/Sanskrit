extern crate sanskrit_deploy;
extern crate sanskrit_compile;
extern crate sanskrit_common;
extern crate sanskrit_core;
extern crate sanskrit_memory_store;
extern crate sanskrit_interpreter;
extern crate sanskrit_default_externals;
extern crate core;
#[macro_use]
extern crate lazy_static;

use sanskrit_deploy::{deploy_module, deploy_function};
use sanskrit_common::errors::*;

use sanskrit_common::store::{StorageClass, Store};
use sanskrit_compile::compile_function;
use store::ExternalStore;
use sanskrit_common::model::HASH_SIZE;
use sanskrit_default_externals::{SYS_MODS, External, ServerExternals};


mod store;

extern  {
    fn load_input(ptr: *mut u8);
    //if return is positive we assume that it suceeded and that many bytes where llocated
    //if return is negative we assume that we did not had enough space and allocate that many space then reexecute load
    fn load(class:u8, ptr_key_hash: *const u8, target_ptr: *mut u8, reserved_space:usize) -> isize;

    //As a later improvement we do not have to pass input back out as there are known outside
    //fn store_input(ptr_key_hash: *u8);

    fn store(class:u8, ptr_key_hash: *const u8, ptr_data: *const u8, ptr_data_size:usize ) -> bool;

    fn error(ptr: *const u8, len: usize);

}

pub fn emit_error(str:&str){
    unsafe{error(str.as_ptr(), str.len())}
}

#[no_mangle]
pub extern fn register(sys_id:isize) -> bool {
    let mut hash = [0;HASH_SIZE];
    unsafe{load_input(hash.as_mut_ptr())}
    if sys_id as usize >= SYS_MODS.len() {
        emit_error("System module index out of range");
        return false;
    }
    let sys_impl = SYS_MODS[sys_id as usize];
    sys_impl(hash);
    return true;
}

#[no_mangle]
pub extern fn compile(input_size: usize, store_prealloc:usize, is_txt:bool, system_mode:bool, system_id:isize) -> bool {
    let mut data = Vec::with_capacity(input_size);
    data.resize(input_size, 0);
    unsafe{load_input(data.as_mut_ptr())}
    if is_txt {
        match process_txt_deploy(data, store_prealloc) {
            Ok(_) => true,
            Err(val) => {
                emit_error(error_to_string(&val));
                false
            }
        }
    } else {
        match process_module_deploy(data,store_prealloc, system_mode, system_id) {
            Ok(_) => true,
            Err(val) => {
                emit_error(error_to_string(&val));
                false
            }
        }
    }
}

fn process_module_deploy(module:Vec<u8>, pre_alloc:usize, system_mode_on:bool, sys_id:isize) -> Result<()>{
    let store = ExternalStore::new(pre_alloc);
    //Note: We use a special store that commits eagerly (for efficiency and thus does not implement commit - hence disable auto commit)
    //      This is ok as in this case storing is the last thing that would happen and we store only the module
    let hash = deploy_module(&store,module,system_mode_on,false)?;
    if system_mode_on && sys_id >= 0 {
        if sys_id as usize >= SYS_MODS.len() {
            return sanskrit_common::errors::error(||"System module index out of range");
        }
        let sys_impl = SYS_MODS[sys_id as usize];
        sys_impl(hash);
    }
    Ok(())
}

//todo: we need to return result
//      later we can just keep store external forwarding calls to the user
//      but this may not be that easy -- bu needs to be done
fn process_txt_deploy(txt:Vec<u8>, pre_alloc:usize) -> Result<Vec<u8>> {
    let store = ExternalStore::new(pre_alloc);
    //Note: We use a special store that commits eagerly (for efficiency and thus does not implement commit - hence disable auto commit)
    //      Note if we fail after deploy we still can record the deploy and only repeat the compile
    //       Later -- we can even make seperate wasm entry points for these
    let hash = deploy_function(&store,txt,false)?;
    let (t_hash,_) = compile_function::<_,ServerExternals>(&store,hash, false)?;
    let res = store.get(StorageClass::Descriptor, &t_hash, |d|d.to_vec())?;
    Ok(res)
}

//Todo: Have a feature that also includes the interprete
// Maybe a seperate package??
