static WASM: &'static [u8] = include_bytes!("../wasm/sanskrit_wasm_deploy_compile_gas.wasm");

use wasmer_runtime::{Value, imports, func, Memory, CompilerConfig, compile_with_config, Instance};
use std::cell::{RefCell, Cell};
use sanskrit_common::store::{StorageClass, Store};
use sanskrit_common::model::{HASH_SIZE, hash_from_slice, Hash};
use fluid_let::fluid_let;
use sanskrit_sled_store::SledStore;
use sanskrit_common::errors::*;

fluid_let!(static INPUT: Vec<u8>);
fluid_let!(static STORAGE: SledStore);

pub struct CompilerInstance(Instance);

impl CompilerInstance {
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

    fn vec_from_mem(mem:&Memory, ptr:usize, len:usize) -> Vec<u8> {
        mem.view()[ptr..(ptr + len)].iter().map(|c|c.get()).collect()
    }


    fn contains(ctx:&mut wasmer_runtime::Ctx, class:i32, ptr_key_hash:i32) -> Result<i32> {
        //todo: Find good formula / value
        Self::gas(ctx,2000)?;
        let memory = ctx.memory(0);
        let hash_vec:Vec<u8> = Self::vec_from_mem(&memory, ptr_key_hash as usize, HASH_SIZE);
        let hash = hash_from_slice(&hash_vec);
        Ok(STORAGE.get(|s|s.unwrap().contains(Self::get_storage_class(class),&hash)) as i32)
    }

    fn load(ctx:&mut wasmer_runtime::Ctx, class:i32, ptr_key_hash:i32, target_ptr:i32, reserved_space:i32) -> Result<i32> {
        //todo: Find good formula / value
        Self::gas(ctx,2000)?;
        let memory = ctx.memory(0);
        let hash_vec:Vec<u8> = Self::vec_from_mem(&memory, ptr_key_hash as usize, HASH_SIZE);
        let hash = hash_from_slice(&hash_vec);
        match STORAGE.get(|s|s.unwrap().get(Self::get_storage_class(class), &hash, |data|{
            if data.len() <=  reserved_space as usize {
                let memory = ctx.memory(0);
                for (byte, cell) in data.iter().zip(memory.view()[target_ptr as usize..(target_ptr as usize + data.len())].iter()){
                    cell.set(*byte);
                }
                data.len() as i32
            } else {
                (0-data.len()) as i32
            }
        })) {
            Ok(inf) => Ok(inf),
            Err(_) => {
                println!("OoS");
                error(||"Load failed")
            }

        }
    }

    fn store(ctx:&mut wasmer_runtime::Ctx, class:i32, ptr_key_hash:i32, ptr_data:i32, ptr_data_size:i32 ) -> Result<i32> {
        //todo: Find good formula / value
        Self::gas(ctx,2000)?;
        let memory = ctx.memory(0);
        let hash_vec = Self::vec_from_mem(&memory, ptr_key_hash as usize, HASH_SIZE);
        let hash = hash_from_slice(&hash_vec);
        Self::CAPTURE.with(|c|c.borrow_mut().push(hash));
        let data = Self::vec_from_mem(&memory, ptr_data as usize, ptr_data_size as usize);
        Ok(STORAGE.get(|s|s.unwrap().set(Self::get_storage_class(class), hash, data).is_ok()) as i32)
    }

    fn load_code(ctx:&mut wasmer_runtime::Ctx, ptr:i32) -> Result<()> {
        //todo: Find good formula / value
        Self::gas(ctx,2000)?;
        let memory = ctx.memory(0);
        INPUT.get(|code|{
            let plain_code = code.unwrap();
            for (byte, cell) in plain_code.iter().zip(memory.view()[ptr as usize..(ptr as usize + plain_code.len())].iter()){
                cell.set(*byte);
            }
        });
        Ok(())
    }


    fn gas(_ctx:&mut wasmer_runtime::Ctx, gas:i32) -> Result<()> {
        Self::GAS.with(|c|{
            //todo: add self cost? either here ore in instrumentation
            let remaining_gas = c.get();
            if remaining_gas < gas as u64 {
                println!("OoG");
                error(||"Out of gas")
            } else {
                c.set(remaining_gas-gas as u64);
                Ok(())
            }
        })
    }

    fn error(ctx:&mut wasmer_runtime::Ctx,  ptr: u32, len: u32) -> Result<()>  {
        let memory = ctx.memory(0);
        // Get a subslice that corresponds to the memory used by the string.
        let str_vec: Vec<_> = memory.view()[ptr as usize..(ptr + len) as usize]
            .iter()
            .map(|cell| cell.get())
            .collect();

        // Convert the subslice to a `&str`.
        let string = std::str::from_utf8(&str_vec).unwrap();

        // Print it!
        println!("{}", string);
        error(||"abort due to error")
    }

    pub fn new() -> Result<Self> {
        println!("Before import");
        let import_object = imports!{
            "env" => {
                "contains" => func!(Self::contains),
                "load" => func!(Self::load),
                "store" => func!(Self::store),
                "load_input" => func!(Self::load_code),
                "gas" => func!(Self::gas),
                "error" => func!(Self::error),
            },
        };
        println!("Before config");
        let mut config = CompilerConfig::default();
        config.enable_verification = true;
        config.generate_debug_info = false;
        println!("Before compile");
        let module = match compile_with_config(WASM,config){
            Ok(module) => module,
            Err(_) => error(||"Could not compile wasm")?
        };
        println!("Before instantiation");
        let instance = match module.instantiate(&import_object){
            Ok(inst) => inst,
            Err(_) => error(||"Could not instantiate wasm")?
        };
        Ok(CompilerInstance(instance))
    }

    pub fn register(&self, hash:Hash, gas_limit:u64, system_id:isize) -> Result<(bool, u64)> {
        Self::GAS.with(|c|c.set(gas_limit));
        INPUT.set(&hash.to_vec(), ||{
            match self.0.call(
                "register",
                &[Value::I32(system_id as i32)]
            ){
                Ok(res) => match res[0] {
                    Value::I32(res) => Ok((res != 0,Self::GAS.with(|c|gas_limit - c.get()))),
                    _ => panic!(),
                },
                Err(msg) => {
                    println!("{:?} after {:?} gas", msg, Self::GAS.with(|c|gas_limit - c.get()));
                    error(||"wasm compiler execution resulted in error")
                },
            }
        })
    }

    fn compile(&self, input:&Vec<u8>, store:&SledStore, gas_limit:u64, import_pre_alloc:u32, is_txt:bool, system_mode:bool, system_id:isize) -> Result<(bool, u64)> {
        Self::GAS.with(|c|c.set(gas_limit));
        INPUT.set(input, || STORAGE.set(store,||{
            match self.0.call(
                "compile",
                &[Value::I32(input.len() as i32), Value::I32(import_pre_alloc as i32), Value::I32(is_txt as i32), Value::I32(system_mode as i32), Value::I32(system_id as i32)]
            ){
                Ok(res) => match res[0] {
                    Value::I32(res) => Ok((res != 0,Self::GAS.with(|c|gas_limit - c.get()))),
                    _ => panic!(),
                },
                Err(msg) => {
                    println!("{:?} after {:?} gas", msg, Self::GAS.with(|c|gas_limit - c.get()));
                    error(||"wasm compiler execution resulted in error")
                },
            }
        }))
    }

    //todo: Safety net if CAPTURE is empty (needed as nothing is stored if it is already deployed
    pub fn compile_transaction(&self, input:&Vec<u8>, store:&SledStore, gas_limit:u64, import_pre_alloc:u32) -> Result<(Hash,Hash, u64)> {
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

    pub fn compile_module(&self, input:&Vec<u8>, store:&SledStore, gas_limit:u64, import_pre_alloc:u32) -> Result<(Hash, u64)> {
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

    pub fn compile_system_module(&self, input:&Vec<u8>, store:&SledStore, id:Option<usize>, gas_limit:u64, import_pre_alloc:u32) -> Result<(Hash, u64)> {
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