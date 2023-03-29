static WASM: &'static [u8] = include_bytes!("../wasm/sanskrit_wasm_deploy_compile_gas.wasm");

//Todo: see -- https://docs.wasmer.io/integrations/examples/host-functions

use std::borrow::BorrowMut;
use wasmer::{imports, Function, Memory, Instance, Module, MemoryView, WasmPtr, FunctionEnvMut, FunctionEnv, AsStoreMut};
use std::cell::{RefCell, Cell};
use std::fmt;
use sanskrit_common::store::{StorageClass, Store};
use sanskrit_common::model::{HASH_SIZE, hash_from_slice, Hash};
use fluid_let::fluid_let;
use sanskrit_sled_store::SledStore;
use sanskrit_common::errors::{error, error_to_string, Result as SResult};


#[derive(Debug, Clone, Copy)]
pub struct ExitCode(u32);

pub fn gen_code(res:&ExitCode) -> &str {
    match res {
        ExitCode(1) => "abort due to error",
        ExitCode(2) => "Memory access failed",
        ExitCode(3) => "Out of gas",
        ExitCode(4) => "Load failed",
        _ => "Unknown Exit Code"
    }

}

impl fmt::Display for ExitCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", gen_code(self))
    }
}

impl std::error::Error for ExitCode {}

#[cfg(feature = "embedded")]
struct InstanceEnv { }

#[cfg(feature = "wasm")]
struct InstanceEnv {
    memory: Option<Memory>,
    store: Option<wasmer::Store>,
}

fluid_let!(static INPUT: Vec<u8>);
fluid_let!(static STORAGE: SledStore);

impl  InstanceEnv{
    fn get_memory(&self) -> Result<MemoryView, ExitCode> {
        match &self.memory {
            Some(mem) => match &self.store {
                Some(st) => Ok(mem.view(st)),
                None => Err(ExitCode(2))
            },
            None => Err(ExitCode(2))
        }
    }
}

pub struct CompilerInstance<'a>{
    instance: Instance,
    env: &'a mut InstanceEnv,
    register : wasmer::TypedFunction<i32, u32>,
    compile: wasmer::TypedFunction<(u32,u32,u32,u32,i32), u32>,
}


impl<'a> CompilerInstance<'a> {

    thread_local! {
       pub static GAS:Cell<u64> = Cell::new(0);
       pub static CAPTURE:RefCell<Vec<Hash>> = RefCell::new(Vec::with_capacity(2));
    }

    fn get_storage_class(class:i32) -> StorageClass {
        match class {
            0 => StorageClass::Module,
            1 => StorageClass::Transaction,
            2 => StorageClass::Descriptor,
            3 => StorageClass::EntryHash,
            4 => StorageClass::EntryValue,
            _ => unreachable!("Non Existent Storage Class")
        }
    }

    fn vec_from_mem(mem:&MemoryView, ptr:WasmPtr<u8>, len:u32) -> Vec<u8> {
        ptr.slice(&mem, len).unwrap().read_to_vec().unwrap()
    }

    fn load(env:FunctionEnvMut<InstanceEnv>, class:i32, ptr_key_hash:WasmPtr<u8>, target_ptr:WasmPtr<u8>, reserved_space:i32) -> Result<i32, ExitCode> {
        //todo: Find good formula / value
        Self::gas(2000)?;
        let memory = env.data().get_memory()?;
        let hash_vec:Vec<u8> = Self::vec_from_mem(&memory, ptr_key_hash, HASH_SIZE as u32);
        let hash = hash_from_slice(&hash_vec);
        match STORAGE.get(|s|s.unwrap().get(Self::get_storage_class(class), &hash, |data|{
            if data.len() <= reserved_space as usize {
                //todo: instead of Unwraps make Exit Codes
                target_ptr.slice(&memory,data.len() as u32).unwrap().write_slice(data).unwrap();
                data.len() as i32
            } else {
                (0-data.len()) as i32
            }
        })) {
            Ok(inf) => Ok(inf),
            Err(err) => {
                println!("{}",error_to_string(&err));
                Err(ExitCode(4))
            }
        }
    }

    fn store(env:FunctionEnvMut<InstanceEnv>, class:i32, ptr_key_hash:WasmPtr<u8>, ptr_data:WasmPtr<u8>, ptr_data_size:i32 ) -> Result<i32, ExitCode> {
        //todo: Find good formula / value
        Self::gas(2000)?;
        let memory = env.data().get_memory()?;
        let hash_vec = Self::vec_from_mem(&memory, ptr_key_hash, HASH_SIZE as u32);
        let hash = hash_from_slice(&hash_vec);
        Self::CAPTURE.with(|c| c.borrow_mut().push(hash));
        let data = Self::vec_from_mem(&memory, ptr_data, ptr_data_size as u32);
        //todo: instead of Unwraps make Exit Codes
        Ok(STORAGE.get(|s| s.unwrap().set(Self::get_storage_class(class), hash, data).is_ok()) as i32)
    }

    fn load_code(env:FunctionEnvMut<InstanceEnv>, ptr:WasmPtr<u8>) -> Result<(), ExitCode> {
        //todo: Find good formula / value
        Self::gas(2000)?;
        let memory = env.data().get_memory()?;
        INPUT.get(|code| {
            let plain_code = code.unwrap();
            //todo: instead of Unwraps make Exit Codes
            ptr.slice(&memory, plain_code.len() as u32).unwrap().write_slice(plain_code).unwrap();
        });
        Ok(())
    }


    fn gas(gas:i32) ->Result<(), ExitCode> {
        Self::GAS.with(|c|{
            //todo: add self cost? either here ore in instrumentation
            let remaining_gas = c.get();
            if remaining_gas < gas as u64 {
                println!("Out of Gas");
                Err(ExitCode(3))
            } else {
                c.set(remaining_gas-gas as u64);
                Ok(())
            }
        })
    }

    fn error(env:FunctionEnvMut<InstanceEnv>, ptr: u32, len: u32) -> Result<(), ExitCode>  {
        let memory = env.data().get_memory()?;
        // Get a subslice that corresponds to the memory used by the string.
        let str_vec: Vec<_> = Self::vec_from_mem(&memory, WasmPtr::<u8>::new(ptr), len);

        // Convert the subslice to a `&str`.
        match std::str::from_utf8(&str_vec) {
            Ok(string) => println!("{}", string),
            Err(_) => ()
        };

        // Print it!
        Err(ExitCode(1))
    }

    #[cfg(feature = "embedded")]
    pub fn with_compiler_result<R>(f: impl FnOnce(&mut Self) -> SResult<R>) -> SResult<R> {
        let mut inst = InstanceEnv {};
        f(&mut inst)
    }
    #[cfg(feature = "wasm")]
    pub fn with_compiler_result<R>(f: impl FnOnce(&mut Self) -> SResult<R>) -> SResult<R> {

        let mut store = wasmer::Store::default();
        let module = Module::new(&store, WASM).unwrap();


        let mut env = FunctionEnv::new(&mut store, InstanceEnv {
            memory: None,
            store : None,
        });


        let import_object = imports!{
            "env" => {
                "load" => Function::new_typed_with_env(&mut store, &env, Self::load),
                "store" => Function::new_typed_with_env(&mut store, &env, Self::store),
                "load_input" => Function::new_typed_with_env(&mut store, &env, Self::load_code),
                "gas" => Function::new_typed(&mut store, Self::gas),
                "error" => Function::new_typed_with_env(&mut store,  &env, Self::error),
            },
        };

        let instance = Instance::new(&mut store, &module, &import_object).unwrap();



        //Todo: Make better error
        let register : wasmer::TypedFunction<i32, u32> = instance.exports.get_function("register").unwrap().typed(&mut store).unwrap();
        let compile : wasmer::TypedFunction<(u32,u32,u32,u32,i32), u32> = instance.exports.get_function("compile").unwrap().typed(&mut store).unwrap();

        let unsafe_ptr: *mut wasmer::Store = unsafe { std::mem::transmute(&mut store) };
        let mut env_mut = env.as_mut(unsafe {&mut *unsafe_ptr });
        env_mut.memory = Some(instance.exports.get_memory("memory").unwrap().clone());
        env_mut.store = Some(store);

        f(&mut CompilerInstance{instance, env:env_mut, register, compile})
    }

    pub fn register(&mut self, hash:Hash, gas_limit:u64, system_id:isize) -> SResult<(bool, u64)> {
        Self::GAS.with(|c|c.set(gas_limit));
        INPUT.set(&hash.to_vec(), || {
            match self.register.call(&mut self.env.store.as_mut().unwrap(), system_id as i32){
                Ok(res) => Ok((res != 0,Self::GAS.with(|c|gas_limit - c.get()))),
                Err(msg) => {
                    println!("{:?} after {:?} gas", msg, Self::GAS.with(|c|gas_limit - c.get()));
                    error(||"wasm compiler execution resulted in error")
                },
            }
        })
    }

    fn compile(&mut self, input:&Vec<u8>, store:&SledStore, gas_limit:u64, import_pre_alloc:u32, is_txt:bool, system_mode:bool, system_id:isize) -> SResult<(bool, u64)> {
        Self::GAS.with(|c|c.set(gas_limit));
        INPUT.set(input, || STORAGE.set(store,||{
            match self.compile.call(&mut self.env.store.as_mut().unwrap(), input.len() as u32, import_pre_alloc as u32, is_txt as u32, system_mode as u32, system_id as i32) {
                Ok(res) => Ok((res != 0, Self::GAS.with(|c| gas_limit - c.get()))),
                Err(msg) => {
                    println!("{:?} after {:?} gas", msg, Self::GAS.with(|c| gas_limit - c.get()));
                    error(|| "wasm compiler execution resulted in error")
                },
            }
        }))
    }

    //todo: Safety net if CAPTURE is empty (needed as nothing is stored if it is already deployed
    pub fn compile_transaction(&mut self, input:&Vec<u8>, store:&SledStore, gas_limit:u64, import_pre_alloc:u32) -> SResult<(Hash,Hash, u64)> {
        let (res,gas) = self.compile(input,store,gas_limit,import_pre_alloc,true,false,-1)?;
        if res {
            Self::CAPTURE.with(|c|{
                let mut vec = c.borrow_mut();
                if vec.is_empty() {
                    error(||"was already stored")
                } else {
                    let desc_hash = vec.pop().unwrap();
                    let fun_hash = vec.pop().unwrap();
                    store.commit(StorageClass::Transaction);
                    store.commit(StorageClass::Descriptor);
                    Ok((desc_hash,fun_hash, gas))
                }
            })
        } else {
            Self::CAPTURE.with(|c|c.borrow_mut().clear());
            store.rollback(StorageClass::Transaction);
            store.rollback(StorageClass::Descriptor);
            error(||"execution failed")
        }
    }

    pub fn compile_module(&mut self, input:&Vec<u8>, store:&SledStore, gas_limit:u64, import_pre_alloc:u32) -> SResult<(Hash, u64)> {
        let (res,gas) = self.compile(input,store,gas_limit,import_pre_alloc,false,false,-1)?;
        if res {
            Self::CAPTURE.with(|c|{
                let mut vec = c.borrow_mut();
                if vec.is_empty() {
                    error(||"was already stored")
                } else {
                    let hash = vec.pop().unwrap();
                    store.commit(StorageClass::Module);
                    Ok((hash, gas))
                }
            })
        } else {
            Self::CAPTURE.with(|c|c.borrow_mut().clear());
            store.rollback(StorageClass::Module);
            error(||"execution failed")
        }
    }

    pub fn compile_system_module(&mut self, input:&Vec<u8>, store:&SledStore, id:Option<usize>, gas_limit:u64, import_pre_alloc:u32) -> SResult<(Hash, u64)> {
        let (res,gas) = match id {
            None => self.compile(input,store,gas_limit,import_pre_alloc,false,true,-1)?,
            Some(id) => self.compile(input,store,gas_limit,import_pre_alloc,false,true,id as isize)?,
        };
        if res {
            Self::CAPTURE.with(|c|{
                let mut vec = c.borrow_mut();
                if vec.is_empty() {
                    error(||"was already stored")
                } else {
                    let hash = vec.pop().unwrap();
                    store.commit(StorageClass::Module);
                    store.rollback(StorageClass::Module);
                    Ok((hash, gas))
                }
            })
        } else {
            Self::CAPTURE.with(|c|c.borrow_mut().clear());
            error(||"execution failed")
        }
    }
}
