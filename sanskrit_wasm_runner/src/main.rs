extern crate wasmer_runtime;
extern crate sanskrit_memory_store;
extern crate sanskrit_common;
extern crate fluid_let;

static WASM: &'static [u8] = include_bytes!("../wasm/sanskrit_wasm_deploy_compile_gas.wasm");
static CODE_MOD: &'static [u8] = include_bytes!("../../sanskrit_test/scripts/out/bool.bin");

use wasmer_runtime::{Value, imports, func, error, Memory, CompilerConfig, compile_with_config, Instance};
use std::cell::{RefCell, Cell};
use std::time::Instant;
use sanskrit_memory_store::BTreeMapStore;
use sanskrit_common::store::{StorageClass, Store};
use sanskrit_common::model::{HASH_SIZE, hash_from_slice};
use fluid_let::fluid_let;


fluid_let!(static INPUT: Vec<u8>);

struct CompilerInstance(Instance);

impl CompilerInstance {
    thread_local! {
       pub static STORAGE: RefCell<BTreeMapStore> = RefCell::new(BTreeMapStore::new());
       pub static GAS:Cell<u64> = Cell::new(0);
    }

    fn get_storage_class(class:i32) -> StorageClass {
        match class {
            0 => StorageClass::Module,
            1 => StorageClass::Transaction,
            2 => StorageClass::Descriptor,
            3 => StorageClass::EntryHash,
            4 => StorageClass::EntryValue,
            _ => unreachable!()
        }
    }

    fn vec_from_mem(mem:&Memory, ptr:usize, len:usize) -> Vec<u8> {
        mem.view()[ptr..(ptr + len)].iter().map(|c|c.get()).collect()
    }

    fn load(ctx:&mut wasmer_runtime::Ctx, class:i32, ptr_key_hash:i32, target_ptr:i32, reserved_space:i32) -> Result<i32,()> {
        //todo: Find good formula / value
        Self::gas(ctx,2000)?;
        let memory = ctx.memory(0);
        let hash_vec:Vec<u8> = Self::vec_from_mem(&memory, ptr_key_hash as usize, HASH_SIZE);
        let hash = hash_from_slice(&hash_vec);
        match Self::STORAGE.with(|s|s.borrow().get(Self::get_storage_class(class), &hash, |data|{
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
            Err(_) => Err(())
        }
    }

    fn store(ctx:&mut wasmer_runtime::Ctx, class:i32, ptr_key_hash:i32, ptr_data:i32, ptr_data_size:i32 ) -> Result<i32,()> {
        //todo: Find good formula / value
        Self::gas(ctx,2000)?;
        let memory = ctx.memory(0);
        let hash_vec = Self::vec_from_mem(&memory, ptr_key_hash as usize, HASH_SIZE);
        let hash = hash_from_slice(&hash_vec);
        let data = Self::vec_from_mem(&memory, ptr_data as usize, ptr_data_size as usize);
        Ok(Self::STORAGE.with(|s|s.borrow_mut().set(Self::get_storage_class(class), hash, data).is_ok()) as i32)
    }

    fn load_code(ctx:&mut wasmer_runtime::Ctx, ptr:i32) -> Result<(),()> {
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


    fn gas(_ctx:&mut wasmer_runtime::Ctx, gas:i32) -> Result<(),()> {
        Self::GAS.with(|c|{
            //todo: add self cost? either here ore in instrumentation
            let remaining_gas = c.get();
            if remaining_gas < gas as u64 {
                Err(())
            } else {
                c.set(remaining_gas-gas as u64);
                Ok(())
            }
        })
    }

    pub fn new() -> error::Result<Self> {
        let import_object = imports!{
            "env" => {
                "load" => func!(Self::load),
                "store" => func!(Self::store),
                "load_input" => func!(Self::load_code),
                "gas" => func!(Self::gas),
            },
        };
        let mut config = CompilerConfig::default();
        config.enable_verification = true;
        config.generate_debug_info = false;
        let module = compile_with_config(WASM,config)?;
        let instance = module.instantiate(&import_object)?;
        Ok(CompilerInstance(instance))
    }

    //todo: take input explicitly & take gas limit explicitly
    //      ev take store explicitly
    pub fn compile(&self, input:&Vec<u8>, gas_limit:u64, import_pre_alloc:u32) -> error::Result<(bool, u64)> {
        Self::GAS.with(|c|c.set(gas_limit));
        let res = INPUT.set(input, ||{
            match self.0.call(
                "call",
                &[Value::I32(input.len() as i32), Value::I32(import_pre_alloc as i32), Value::I32(0), Value::I32(0), Value::I32(-1)]
            ){
                Ok(res) => match res[0] {
                    Value::I32(res) => res != 0,
                    _ => panic!(),
                },
                Err(msg) => {
                    println!("{:?}", msg);
                    false
                },
            }
        });
       Ok((res,Self::GAS.with(|c|gas_limit - c.get())))

    }
}


fn main() -> error::Result<()> {

    let instance = CompilerInstance::new()?;

    const GAS_LIMIT:u64 = 10000000;
    const N:u128 = 100000;
    let input_vec = CODE_MOD.to_vec();
    let mut gas = 0;
    let inst = Instant::now();
    for _ in 0..N {
        CompilerInstance::STORAGE.with(|s|s.borrow_mut().clear_section(StorageClass::Module));
        let (res,gas_used) = instance.compile(&input_vec, GAS_LIMIT, 1000)?;
        assert!(res);
        gas = gas_used
    }

    let time = (inst.elapsed().as_micros())/N;

    println!("gas: {:?} - exec_time: {:?}us",gas,time);

    Ok(())
}