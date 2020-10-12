#![feature(nll)]

//extern crate pwasm_std;

extern crate sanskrit_deploy;
extern crate sanskrit_compile;
extern crate sanskrit_common;
extern crate sanskrit_memory_store;
extern crate core;
#[macro_use]
extern crate lazy_static;

use sanskrit_memory_store::BTreeMapStore;
use sanskrit_deploy::deploy_module;
use sanskrit_compile::compile_module;

use std::mem;
use std::sync::Mutex;

lazy_static! {
    static ref STORAGE: Mutex<BTreeMapStore> = Mutex::new(BTreeMapStore::new());
}


#[no_mangle]
pub extern fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut u8;
}

#[no_mangle]
pub extern fn dealloc(ptr: *mut u8, cap: usize) {
    unsafe {
        let _buf = Vec::from_raw_parts(ptr, 0, cap);
    }
}

#[no_mangle]
pub extern fn call(ptr: *mut u8, bytesize: usize) -> usize{
    let data:Vec<u8> = unsafe { Vec::from_raw_parts(ptr, bytesize, bytesize) };
    let s = &*STORAGE.lock().unwrap();
    let h = match deploy_module(s,data){
        Ok(h) => h,
        Err(_) => return 1
    };

    match compile_module(s,h){
        Ok(_) => 0,
        Err(_) => 1
    }

}

#[no_mangle]
pub extern fn reset(){
    *STORAGE.lock().unwrap() = BTreeMapStore::new()
}

/*
struct BlockChainStore;

fn gen_key( class: StorageClass, key: &[u8; 20]) -> H256 {
    let mut s_key = [0u8;32];
    s_key[0] = match class {
        StorageClass::AdtDesc =>  0,
        StorageClass::FunDesc => 1,
        StorageClass::Module => 2,
        StorageClass::Elem => 3,
    };
    s_key[12..].copy_from_slice(key);

    H256::from(s_key)
}

//todo: incorrect dummies
//      in reality we need to store more than 32 bytes

impl Store for BlockChainStore {
    fn contains(&self, class: StorageClass, key: &[u8; 20]) -> bool {
        ext::read(&gen_key(class,key)) != [0u8;32]
    }

    fn delete(&self, class: StorageClass, key: &[u8; 20]) -> Result<()> {
        ext::write(&gen_key(class,key), &[0u8;32]);
        Ok(())
    }

    fn get<P, F: FnOnce(&[u8]) -> P>(&self, class: StorageClass, key: &[u8; 20], f: F) -> Result<P> {
        Ok(f(&ext::read(&gen_key(class,key))))
    }

    fn set(&self, class: StorageClass, key: [u8; 20], data: Vec<u8>) -> Result<()> {
        ext::write(&gen_key(class,&key), data.to());
        Ok(())
    }

    fn replace(&self, class: StorageClass, key: [u8; 20], data: Vec<u8>) -> Result<()> {
        self.set(class,key,data)
    }
}
*/