#![feature(nll)]

extern crate rustyline;
extern crate sanskrit_runtime;
extern crate sanskrit_compile;
extern crate sanskrit_deploy;
extern crate sanskrit_common;
extern crate sanskrit_core;
extern crate sanskrit_test_script_compiler;
extern crate sanskrit_sled_store; //for now later use an ethereum or substrate based one
extern crate sled;
extern crate ed25519_dalek;
extern crate sha2;
extern crate rand;
extern crate hex;
#[macro_use]
extern crate arrayref;
extern crate blake2_rfc;

use std::io::{stdin,stdout,Write};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::path::Path;
use sanskrit_sled_store::SledStore;
use std::env;
use std::path::PathBuf;
use sanskrit_test_script_compiler::model::Id;
use sanskrit_test_script_compiler::transaction::Compiler;
use std::collections::HashSet;
use sanskrit_deploy::deploy_module;
use sanskrit_compile::compile_module;
use sanskrit_test_script_compiler::transaction::CompiledTransactions;
use sanskrit_runtime::execute;
use sanskrit_common::store::store_hash;
use sanskrit_common::store::Store;
use sanskrit_common::store::StorageClass;
use sanskrit_common::encoding::*;
use sanskrit_core::model::Module;
use sanskrit_common::model::Hash;
use sanskrit_runtime::model::StoreElem;
use std::collections::HashMap;
use sled::Tree;
use rand::rngs::OsRng;
use sha2::Sha512;
use ed25519_dalek::Keypair;
use hex::encode;
use blake2_rfc::blake2b::{Blake2b};
use sanskrit_common::arena::Heap;

struct State<'a> {
    transaction: Compiler<'a>,
    store:SledStore,
    deployed:HashSet<Id>,
    accounts:Tree,
}

impl<'a> State<'a> {

    fn deploy_modules(&mut self, modules:impl IntoIterator<Item=(Id,Vec<u8>)>) {
        for (id,data) in modules{
            if !self.deployed.contains(&id) {
                let hash1 = store_hash(&[&data]);
                if !self.store.contains(StorageClass::Module, &hash1){
                    let hash = deploy_module(&self.store,data).expect(&format!("error in {:?}", id));
                    assert_eq!(hash,hash1);
                    compile_module(&self.store, hash).unwrap();
                    println!("deployed {:?} with hash 0x{:?}", &id, encode(&hash));
                }
                self.deployed.insert(id);
            }
        }
    }

    fn deploy(&mut self, name:Id) {
        let m_comp = self.transaction.get_module_compiler();
        m_comp.parse_module_tree(name);
        m_comp.compile_module_tree().unwrap();
        let res = m_comp.get_results().into_iter();
        self.deploy_modules(res);
    }

    fn execute(&mut self, name:Id, accounts:HashMap<Id,Keypair>) {
        self.transaction.parse_transactions(name);
        self.transaction.compile_transactions_with_accounts(accounts).unwrap();
        let CompiledTransactions{modules,txts} = self.transaction.extract_results();
        self.deploy_modules(modules.into_iter());
        let mut heap = Heap::new(100000,2.0);
        for txt in txts {
            let hash = execute(&self.store,&txt,0, &heap).unwrap();
            println!("executed txt with hash 0x{:?}", encode(&hash));
            heap = heap.reuse()
        }
    }

    fn get_account(&mut self, ident:String) -> Keypair {
        let key = ident.into_bytes();
        if self.accounts.contains_key(&key).unwrap() {
           return Keypair::from_bytes(&self.accounts.get(&key).unwrap().unwrap()).unwrap()
        }
        let mut csprng: OsRng = OsRng::new().unwrap();
        let kp = Keypair::generate::<Sha512, _>(&mut csprng);
        self.accounts.set(key,kp.to_bytes().to_vec()).unwrap();
        return kp;
    }
}

pub fn main() {

    //Commands:
    // Deploy ... <-- Path to file                      | will automatically start the whole build process
    // Execute ... <-- Path to file + Accounts (signer) | will automatically start the whole build process
    //   Can subs: alla: execute test A=... B=... //where A b the Account used to sign
    //   Create account implicitly
    // Inspect ... <-- Displays a stored value
    // transaction { .. } <-- Execute a txt
    // transaction[..]{..} <-- Gives list of signing accounts
    // List Modules ...
    // List Types ...
    // List Functions ...
    // List Elements
    // Clear ... <-- Clears DB
    //

    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    let args: Vec<String> = env::args().collect();
    let work_dir =  env::current_dir().unwrap();
    let db = if args.len() >= 2 {
        Path::new(&args[1]).to_owned()
    } else {
        work_dir.join("db")
    };

    let account_p = db.join("account").with_extension("db");
    let mut state = State {
        transaction: Compiler::new(&work_dir),
        store: SledStore::new(&db),
        deployed: HashSet::new(),
        accounts: Tree::start_default(account_p).unwrap(),
    };

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                let elems:Vec<String> = line.split(|c| c == ' ' || c == '=' || c == ':').map(|e|e.trim().to_owned()).filter(|s|!s.is_empty()).collect();
                match elems[0].to_lowercase().as_ref() {
                    "deploy" => state.deploy(Id(elems[1].to_owned())),
                    "execute" => {
                        let mut count = 2;
                        let mut accounts = HashMap::new();
                        while count+1 < elems.len() {
                            let id = Id(elems[count].clone());
                            let subs = state.get_account(elems[count+1].clone());
                            accounts.insert(id,subs);
                            count+=2;
                        }
                        state.execute(Id(elems[1].to_owned()),accounts)
                    },
                    "account" => {
                        let subs = state.get_account(elems[1].clone());
                        println!("name:{:?} address:0x{:?} pk:0x{:?}", elems[1].clone(), encode(hash(&subs.public.to_bytes())) , encode(subs.public.to_bytes()));
                    }
                    "list" =>  match elems[1].to_lowercase().as_ref() {
                        "module" => {
                            let mods = state.store.list(StorageClass::Module);
                            let res:Vec<(Hash,String)> = mods.iter().map(|(hash,data)|(
                                hash.clone(),
                                String::from_utf8(ParserState::parse_fully::<Module>(&*data, &mut NoCustomAlloc()).unwrap().meta.0).unwrap()
                            )).collect();

                            for (hash,id) in res {
                                println!("hash:0x{:?} => id:{:?}", encode(hash),id);
                            };
                        },
                        "elems" =>  {
                            let mods = state.store.list(StorageClass::Elem);
                            let res:Vec<StoreElem> = mods.iter().map(|(_,data)| ParserState::parse_fully::<StoreElem>(&*data, &mut NoCustomAlloc()).unwrap()).collect();

                            for elem in res {
                                println!("elem:{:?}", elem);
                            };
                        },
                        _ =>  println!("?:Line: {:?}", elems),
                    }
                    _ =>  println!("?:Line: {:?}", elems),
                }
            },
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
    }
    rl.save_history("history.txt").unwrap();
}

//Helper to calc the a hash
fn hash(data:&[u8]) -> Hash {
    //Make a 20 byte digest hascher
    let mut context = Blake2b::new(20);
    //push the data into it
    context.update(data);
    //calc the Hash
    let hash = context.finalize();
    //generate a array to the hash
    let hash_data_ref = array_ref!(hash.as_bytes(),0,20);
    //get ownership and return
    hash_data_ref.to_owned()
}