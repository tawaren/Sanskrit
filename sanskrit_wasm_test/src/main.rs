extern crate wasmer_runtime;
extern crate sanskrit_memory_store;
extern crate sanskrit_common;

static WASM: &'static [u8] = include_bytes!("../wasm/sanskrit_wasm_deploy_compile_gas.wasm");

static CODE_MOD: &'static [u8] = include_bytes!("../../sanskrit_test/scripts/out/bool.bin");

use wasmer_runtime::{Value, imports, func, error, compile, Memory};
use std::cell::{RefCell, Cell};
use std::time::Instant;
use sanskrit_memory_store::BTreeMapStore;
use sanskrit_common::store::{StorageClass, Store};
use sanskrit_common::model::{HASH_SIZE, hash_from_slice};

thread_local! {
   pub static STORAGE: RefCell<BTreeMapStore> = RefCell::new(BTreeMapStore::new());
   pub static GAS:Cell<u64> = Cell::new(0);
}

fn main() -> error::Result<()> {

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

    fn contains(ctx:&mut wasmer_runtime::Ctx, class:i32, ptr_key_hash:i32) -> Result<i32,()> {
        //todo: Find good formula / value
        gas(ctx,2000)?;
        let memory = ctx.memory(0);
        let hash_vec:Vec<u8> = vec_from_mem(&memory, ptr_key_hash as usize, HASH_SIZE);
        let hash = hash_from_slice(&hash_vec);
        Ok(STORAGE.with(|s|s.borrow().contains(get_storage_class(class),&hash)) as i32)
    }

    fn load(ctx:&mut wasmer_runtime::Ctx, class:i32, ptr_key_hash:i32, target_ptr:i32, reserved_space:i32) -> Result<i32,()> {
        //todo: Find good formula / value
        gas(ctx,2000)?;
        let memory = ctx.memory(0);
        let hash_vec:Vec<u8> = vec_from_mem(&memory, ptr_key_hash as usize, HASH_SIZE);
        let hash = hash_from_slice(&hash_vec);
        match STORAGE.with(|s|s.borrow().get(get_storage_class(class), &hash, |data|{
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
        gas(ctx,2000)?;
        let memory = ctx.memory(0);
        let hash_vec = vec_from_mem(&memory, ptr_key_hash as usize, HASH_SIZE);
        let hash = hash_from_slice(&hash_vec);
        let data = vec_from_mem(&memory, ptr_data as usize, ptr_data_size as usize);
        Ok(STORAGE.with(|s|s.borrow_mut().set(get_storage_class(class), hash, data).is_ok()) as i32)
    }

    fn load_code(ctx:&mut wasmer_runtime::Ctx, ptr:i32) -> Result<(),()> {
        //todo: Find good formula / value
        gas(ctx,2000)?;
        let memory = ctx.memory(0);
        for (byte, cell) in CODE_MOD.iter().zip(memory.view()[ptr as usize..(ptr as usize + CODE_MOD.len())].iter()){
            cell.set(*byte);
        }
        Ok(())
    }

    const GAS_LIMIT_INTERNAL:u64 = 10000000;

    fn gas(_ctx:&mut wasmer_runtime::Ctx, gas:u32) -> Result<(),()> {
        let res = GAS.with(|c|{
            let gas = c.get()+gas as u64;
            c.set(gas);
            gas
        });
        if res > GAS_LIMIT_INTERNAL {
            Err(())
        } else {
            Ok(())
        }

    }


    let import_object = imports!{
        "env" => {
            "contains" => func!(contains),
            "load" => func!(load),
            "store" => func!(store),
            "load_input" => func!(load_code),
            "gas" => func!(gas),
        },
    };

    let module = compile(WASM)?;
    let instance = module.instantiate(&import_object)?;

    let inst = Instant::now();
    const N:u128 = 100000;
    for _ in 0..N {
        GAS.with(|c|c.set(0));
        STORAGE.with(|s|s.borrow_mut().clear_section(StorageClass::Module));

        let res = match instance.call(
            "call",
            &[Value::I32(CODE_MOD.len() as i32), Value::I32(1000 as i32), Value::I32(0), Value::I32(0), Value::I32(-1)]
        ){
            Ok(res) => match res[0] {
                Value::I32(res) => res != 0,
                _ => panic!(),
            },
            Err(msg) => {
                println!("{:?}", msg);
                false
            },
        };
        assert!(res);
    }

    let time = (inst.elapsed().as_micros())/N;

    println!("gas: {:?} - exec_time: {:?}us",GAS.with(|c|c.get()),time);

    Ok(())
}