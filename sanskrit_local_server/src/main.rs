extern crate rustyline;
extern crate sanskrit_runtime;
extern crate sanskrit_common;
extern crate sanskrit_core;
extern crate sanskrit_interpreter;
extern crate sanskrit_sled_store; //for now later use an ethereum or substrate based one
extern crate sled;
extern crate ed25519_dalek;
extern crate sha2;
extern crate rand;
extern crate hex;
extern crate arrayref;
extern crate byteorder;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate sanskrit_derive;

#[macro_use]
extern crate lalrpop_util;
extern crate fluid_let;
#[cfg(feature = "wasm")]
#[cfg(feature = "embedded")]
extern crate sanskrit_compile;
#[cfg(feature = "embedded")]
extern crate sanskrit_deploy;
extern crate wasmer;

mod manager;
mod parser_model;
mod externals;
#[cfg(feature = "wasm")]
mod compiler;

lalrpop_mod!(pub parser);

//extern crate blake2_rfc;

use std::path::Path;
use sanskrit_sled_store::SledStore;
use std::{env, thread};

use sanskrit_common::errors::*;

use std::net::TcpListener;
use byteorder::{ NetworkEndian, ReadBytesExt, WriteBytesExt};
use manager::{State, TrackingState, ExecutionState, ModuleNames, Tx};
use std::io::{Read, Write};
use std::error::Error;
use sanskrit_common::model::{Hash, hash_from_slice};
use hex::encode;
use std::sync::{Mutex, Arc};
use rustyline::Editor;
use rustyline::error::ReadlineError;
use parser_model::Execute;
use sanskrit_common::arena::{Heap, VirtualHeapArena};
use sanskrit_interpreter::model::Entry;
use sanskrit_common::encoding::{VirtualSize, Parser, NoCustomAlloc};
use std::collections::BTreeSet;
use std::cell::RefCell;
use std::rc::Rc;
use sanskrit_common::store::StorageClass;
#[cfg(feature = "wasm")]
use compiler::CompilerInstance;

pub const MODULE_COMMAND:u8 = 0;
pub const TRANSACTION_COMMAND:u8 = 1;
pub const SYS_MODULE_COMMAND:u8 = 2;

pub const SUCCESS_RETURN:u8 = 0;
pub const ERROR_RETURN:u8 = 1;


fn convert_error<T, E:Error>(err:core::result::Result<T,E>) -> Result<T> {
    match err {
        Err(e) => error(||e.description()),
        Ok(t) => Ok(t)
    }
}

fn read_length_prefixed_array<R:Read>(reader:&mut R) -> Result<Vec<u8>> {
    let length = convert_error(reader.read_u32::<NetworkEndian>())?;
    let mut data = Vec::with_capacity(length as usize);
    for _ in 0..length {
        data.push(  convert_error(reader.read_u8())? )
    }
    Ok(data)
}


fn handle_data<R:Read>(state: &mut State, compiler:&mut CompilerInstance, reader:&mut R) -> Result<Vec<Hash>>{
    match convert_error(reader.read_u8())? {
        MODULE_COMMAND => {
            let meta_data_bytes = read_length_prefixed_array(reader)?;
            let data:ModuleNames = Parser::parse_fully(&meta_data_bytes,6,&NoCustomAlloc())?;
            let name = (data.0).0;
            let bytes = read_length_prefixed_array(reader)?;
            let hash = state.deploy_module(compiler, bytes, false, None)?;
            println!("Mapping Module with hash {:?} to name {}", encode(&hash),&name);
            convert_error(state.tracking.data_names.insert(hash.clone(),meta_data_bytes))?;
            convert_error(state.module_name_mapping.insert(name,&hash.clone()))?;
            convert_error(state.tracking.data_names.flush())?;
            convert_error(state.module_name_mapping.flush())?;
            Ok(vec![hash])
        },
        //Todo: allow to disable this per argument
        SYS_MODULE_COMMAND => {
            let meta_data_bytes = read_length_prefixed_array(reader)?;
            let data:ModuleNames = Parser::parse_fully(&meta_data_bytes,6,&NoCustomAlloc())?;
            let name = (data.0).0;
            let (hash, e_hash) = if convert_error(reader.read_u8())? != 0 {
                let sys_id = convert_error(reader.read_u8())?;
                if sys_id as usize >= externals::SYS_MODS.len() {
                    return error(||"unknown system module identifier")
                }
                let bytes = read_length_prefixed_array(reader)?;
                let hash = state.deploy_module(compiler,bytes, true, Some(sys_id))?;
                let sys_impl = externals::SYS_MODS[sys_id as usize];
                sys_impl(hash.clone());
                convert_error(state.system_entries.insert(&[sys_id], &hash))?;
                convert_error(state.system_entries.flush())?;
                let e_hash = encode(&hash);
                println!("Registered System Module {} with Hash {:?} with System Number {:?}",name, e_hash,sys_id);
                (hash, e_hash)
            } else {
                let bytes = read_length_prefixed_array(reader)?;
                let hash = state.deploy_module(compiler,bytes, true, None)?;
                let e_hash = encode(&hash);
                println!("Registered System Module {} with Hash {:?}",name, e_hash);
                (hash, e_hash)
            };
            println!("Mapping Module with hash {:?} to name {}", e_hash,&name);
            convert_error(state.tracking.data_names.insert(hash.clone(),meta_data_bytes))?;
            convert_error(state.module_name_mapping.insert(name,&hash.clone()))?;
            convert_error(state.tracking.data_names.flush())?;
            convert_error(state.module_name_mapping.flush())?;
            Ok(vec![hash])
        },
        TRANSACTION_COMMAND => {
            let name_bytes = read_length_prefixed_array(reader)?;
            let name = convert_error(String::from_utf8(name_bytes))?;
            let bytes = read_length_prefixed_array(reader)?;
            let (f_hash,d_hash) = state.deploy_transaction(compiler, bytes)?;
            println!("Mapping Transaction with descriptor hash {:?} to name {}", encode(&d_hash) ,&name);
            convert_error(state.transaction_name_mapping.insert(name,&d_hash.clone()))?;
            convert_error(state.transaction_name_mapping.flush())?;
            Ok(vec![f_hash, d_hash])
        },
        _ => error(||"Unknown Command"),
    }
}

enum ProcRes {
    End,
    Continue,
    Switch(Box<LineProcessor>)
}


fn extract_command(line:String) -> (String,String) {
    match line.trim().find(|c:char| c.is_whitespace()) {
        None => (line.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_lowercase(), "".to_owned()),
        Some(pos) => match line.split_at(pos){
            (c,d) => (c.trim().to_lowercase(), d.trim().to_owned())
        },
    }
}

fn process_txt_line(input:String, shared_state:Arc<Mutex<State>>, mut bindings: &mut BTreeSet<String>) -> Result<Tx> {
    let txt:Execute = convert_error(parser::ExecuteParser::new().parse(&input))?;
    let mut local_state =   convert_error(shared_state.lock())?;
    txt.build_param_names(&mut local_state);
    txt.build_return_names(&mut local_state);
    let desc = txt.txt_hash(&mut local_state)?;
    let returns = txt.build_returns(&mut bindings);
    let params = txt.build_params(&mut local_state, &bindings)?;
    Ok(Tx { desc, params, returns })
}

type LineProcessor = dyn Fn(String, &VirtualHeapArena) -> Result<ProcRes>;

fn process_line(line:String, shared_state:Arc<Mutex<State>>, full_heap:&VirtualHeapArena) -> Result<ProcRes>{
    let (command, input) = extract_command(line);

    //Todo: Make multi transaction version
    match command.to_lowercase().as_ref() {
        "bundle" => {
            let state = Rc::new(RefCell::new((BTreeSet::new(),Vec::new())));
            {
                let mut local_state =  convert_error(shared_state.lock())?;
                local_state.tracking.exec_state = ExecutionState::new();
            }
            return Ok(ProcRes::Switch(Box::new(move |line:String, _|process_bundle_line(line,state.clone(), shared_state.clone()))))
        },

        //executes a transaction
        "execute" | "exec" =>  {
            let mut bindings:BTreeSet<String> = BTreeSet::new();
            {
                let mut local_state =  convert_error(shared_state.lock())?;
                local_state.tracking.exec_state = ExecutionState::new();
            }
            match process_txt_line(input, shared_state.clone(), &mut bindings) {
                Ok(tx) => {
                    let mut local_state =  convert_error(shared_state.lock())?;
                    local_state.execute_transaction(&[tx])?;
                    if local_state.tracking.exec_state.success{
                        println!("transaction execution successful")
                    } else {
                        println!("transaction execution was rolled back")
                    }
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        },
        //prints account infos (creates it if it does not exist)
        "account" => {
            let kp =   convert_error(shared_state.lock())?.get_account(&input)?;
            let pk = kp.public.to_bytes();
            println!("Pk: 0x{}",encode(pk));
            println!("Subject: 0x{}",encode(State::calc_subject(&pk, full_heap)?.to_vec()))

        },
        "accounts" =>  for (name, kp) in convert_error(shared_state.lock())?.get_accounts()? {
            println!("{} -> 0x{}",name,encode(kp.public.to_bytes()))
        },

        "transactions" =>  for name in convert_error(shared_state.lock())?.get_transactions()? {
            println!("{}",name)
        },

        "transaction" =>   {
            let mut local_state = convert_error(shared_state.lock())?;
            let txt = local_state.get_transaction(&input, full_heap)?;
            println!("size: {} bytes",txt.byte_size.unwrap());
            println!("virtual size: {} bytes",txt.virt_size.unwrap());
            println!("max consumed memory: {} bytes",txt.max_mem);
            println!("max stack slots: {} ({} bytes)",txt.max_stack, (txt.max_stack as usize) * Entry::SIZE);
            println!("max frame slots: {} (~{} bytes)",txt.max_frames, (txt.max_frames as usize) * (5*8));
            println!("gas cost: {}",txt.gas_cost);
            println!("num params: {}",txt.params.len());
            println!("num returns: {}",txt.returns.len());
            println!("num nested functions: {}",txt.functions.len());
        },

        "modules" => for name in convert_error(shared_state.lock())?.get_modules()? {
            println!("{}",name)
        },

        "module" =>   {
            let mut local_state = convert_error(shared_state.lock())?;
            let txt = local_state.get_module(&input, full_heap)?;
            println!("size: {} bytes",txt.byte_size.unwrap());
            println!("num data types: {}",txt.data.len());
            println!("num signature types: {}",txt.sigs.len());
            println!("num functions: {}",txt.functions.len());
            println!("num implementations: {}",txt.implements.len());
        },

        "elems" =>  for (name, data) in convert_error(shared_state.lock())?.get_elems()? {
            println!("{} -> 0x{}",name,data)
        }

        "elem" => {
            let mut local_state = convert_error(shared_state.lock())?;
            let elem = local_state.get_elem(&input)?;
            println!("{}",elem)
        }

        "exit" => return Ok(ProcRes::End),

        x if x.len() != 0 =>  println!("Unknown Command"),
        _ => { }
    }
    Ok(ProcRes::Continue)
}


fn process_bundle_line(line:String, bundle_state:Rc<RefCell<(BTreeSet<String>,Vec<Tx>)>>, shared_state:Arc<Mutex<State>>) -> Result<ProcRes> {
    let (command, input) = extract_command(line);
    match command.to_lowercase().as_ref() {
        "abort" => return Ok(ProcRes::End),
        "transaction" | "txt" => {
            let (ref mut bindings, ref mut txts) = *bundle_state.borrow_mut();
            match process_txt_line(input, shared_state.clone(), bindings) {
                Ok(tx) => txts.push(tx),
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        },
        "execute" | "exec" | "end" => {
            let mut local_state =  convert_error(shared_state.lock())?;
            let (_, ref txts) = *bundle_state.borrow_mut();
            match local_state.execute_transaction(&txts) {
                Ok(_) => {}
                Err(err) => println!("transaction bundle execution produced error: {:?}",err)
            }
            if local_state.tracking.exec_state.success{
                println!("transaction bundle execution successful")
            } else {
                println!("transaction bundle execution was rolled back")
            }
            return Ok(ProcRes::End)
        }
        "bench_code" => {
            let mut local_state = convert_error(shared_state.lock())?;
            let (_, ref txts) = *bundle_state.borrow_mut();
            match local_state.bench_transaction(&txts) {
                Ok(_) => {}
                Err(err) => println!("transaction bundle execution produced error: {:?}",err)
            }
            if local_state.tracking.exec_state.success{
                println!("transaction bundle execution successful")
            } else {
                println!("transaction bundle execution was rolled back")
            }
            return Ok(ProcRes::End)
        }
        x if x.len() != 0 =>  println!("Unknown Command"),
        _ => { }
    }
    Ok(ProcRes::Continue)
}

#[cfg(feature = "wasm")]
fn register_system_modules(state:&State, compiler:&mut CompilerInstance) -> std::io::Result<()> {
    for entry in state.system_entries.iter() {
        let (k,e) = entry?;
        let sys_id = k[0];
        let hash = hash_from_slice(&e);
        let sys_impl = externals::SYS_MODS[sys_id as usize];
        sys_impl(hash.clone());
        match compiler.register(hash.clone(), 100000, sys_id as isize) {
            Ok(_) => {}
            Err(_) => {}
        };
        let e_hash = encode(&hash);
        println!("Re-Registered Module with Hash {:?} as System Module with Number {:?}",e_hash,sys_id);
    }
    Ok(())
}

#[cfg(feature = "embedded")]
fn register_system_modules(state:&State, _compiler:&CompilerData) -> std::io::Result<()> {
    for entry in state.system_entries.iter() {
        let (k,e) = entry?;
        let sys_id = k[0];
        let hash = hash_from_slice(&e);
        let sys_impl = externals::SYS_MODS[sys_id as usize];
        sys_impl(hash.clone());
        let e_hash = encode(&hash);
        println!("Re-Registered Module with Hash {:?} as System Module with Number {:?}",e_hash,sys_id);
    }
    Ok(())
}

pub fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let work_dir =  env::current_dir()?;
    let db_folder = if args.len() >= 2 {
        Path::new(&args[1]).to_owned()
    } else {
        work_dir.join("db")
    };

    let account_db = db_folder.join("accounts").with_extension("db");
    let module_name_db = db_folder.join("module_names").with_extension("db");
    let transaction_name_db = db_folder.join("transaction_names").with_extension("db");
    let elem_tracker_db = db_folder.join("elem_tracker").with_extension("db");
    let data_tracker_db = db_folder.join("data_tracker").with_extension("db");
    let data_names_tracker_db = db_folder.join("data_name_tracker").with_extension("db");
    let sys_entry_db = db_folder.join("system_modules").with_extension("db");
    let meta_db = db_folder.join("meta_data").with_extension("db");

    let history = work_dir.join("history").with_extension("txt");

    let mut auto_flushes = BTreeSet::new();
    auto_flushes.insert(StorageClass::Transaction);
    auto_flushes.insert(StorageClass::Module);
    auto_flushes.insert(StorageClass::Descriptor);

    let mut state = State {
        store: SledStore::new(&db_folder, auto_flushes),
        accounts:sled::open(account_db)?,
        system_entries:sled::open(sys_entry_db)?,
        module_name_mapping:sled::open(module_name_db)?,
        transaction_name_mapping:sled::open(transaction_name_db)?,
        tracking: TrackingState {
            exec_state: ExecutionState::new(),
            active_elems:sled::open(elem_tracker_db)?,
            element_data:sled::open(data_tracker_db)?,
            data_names:sled::open(data_names_tracker_db)?,
        },
        meta_data:sled::open(meta_db)?,
    };

    let shared_state = Arc::new(Mutex::new(state));
    let listener_state = Arc::clone(&shared_state);
    // accept connections and process them serially
    println!("Started Local VM");
    thread::spawn(move || {
        CompilerInstance::with_compiler_result(|mut compiler |{
            register_system_modules(&mut listener_state.lock().unwrap(), &mut compiler).unwrap();
            let listener = TcpListener::bind("127.0.0.1:6000").unwrap();
            for stream_res in listener.incoming() {
                let mut stream = stream_res.unwrap();
                let mut state = listener_state.lock().unwrap();
                match handle_data(&mut state, &mut compiler, &mut stream) {
                    Err(error) => {
                        println!("{}",error_to_string(&error));
                        stream.write_u8(ERROR_RETURN).unwrap();
                    },
                    Ok(hashes) => {
                        stream.write_u8(SUCCESS_RETURN).unwrap();
                        for hash in hashes {
                            stream.write_all(&hash).unwrap();
                        }
                    }
                }
            }
            Ok(())
        })

    });

    let mut rl = Editor::<()>::new();
    if rl.load_history(&history).is_err() {
        println!("No previous history.");
    }


    let heap = Heap::new(100000000,2.0);
    let mut full_heap = heap.new_virtual_arena(10000000 as usize);

    let mut processor:Box<LineProcessor> = Box::new(move |line:String, heap:&VirtualHeapArena|process_line(line, shared_state.clone(), &heap));
    let mut stack:Vec<Box<LineProcessor>> = Vec::new();

    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                rl.add_history_entry(&line);
                match processor(line, &full_heap) {
                    Err(err) => {
                        println!("Error: {:?}", err);
                        continue
                    }
                    Ok(ProcRes::End) if stack.is_empty() => break,
                    Ok(ProcRes::End) => {
                        processor = stack.pop().unwrap()
                    },
                    Ok(ProcRes::Continue) => {},
                    Ok(ProcRes::Switch(proc)) => {
                        stack.push(processor);
                        processor = proc
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
        full_heap = full_heap.reuse();
        rl.save_history(&history).unwrap();
    }
    Ok(())
}
