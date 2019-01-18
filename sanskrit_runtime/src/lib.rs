#![no_std]
#![feature(alloc)]
#![feature(nll)]

extern crate blake2_rfc;
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate arrayref;
extern crate byteorder;
extern crate num_traits;
extern crate ed25519_dalek;
extern crate sha2;
extern crate sanskrit_common;
#[macro_use]
extern crate sanskrit_derive;


use sanskrit_common::store::Store;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::Parser;
use model::Transaction;
use sanskrit_common::model::Hash;
use ed25519_dalek::PublicKey;
use ed25519_dalek::Signature;
use blake2_rfc::blake2b::{Blake2b};
use sha2::{Sha512};
use model::RuntimeType;
use elem_store::ElemStore;
use script_stack::LinearScriptStack;
use script_interpreter::Executor;
use sanskrit_common::arena::*;
use alloc::prelude::Vec;
use script_stack::StackEntry;
use sanskrit_common::model::SlicePtr;
use sanskrit_common::linear_stack::Elem;
use sanskrit_common::model::Ptr;
use model::Object;
use interpreter::Frame;
use elem_store::CacheSlotMap;
use elem_store::CacheEntry;


pub mod native;
pub mod descriptors;
pub mod interpreter;
pub mod model;
pub mod script_stack;
pub mod script_interpreter;
pub mod type_builder;
pub mod elem_store;
pub mod encoding;


pub const CONFIG: Configuration = Configuration {
    max_stack_depth:1024,
    max_frame_depth:256,
    max_heap_size:512 * 1024,
    max_script_stack_size:256,
    max_code_size:128 * 1024,
    max_structural_dept:64,
    max_transaction_size:128 * 1024,
    max_store_slots: 32,
    temporary_buffer: 24 * 255 //recalc
};

impl Configuration {
    pub const fn calc_heap_size(&self, virt_factor:usize) -> usize {
        Heap::elems::<Elem<StackEntry,SlicePtr<usize>>>(self.max_script_stack_size)
        + Heap::elems::<Ptr<Object>>(self.max_stack_depth)
        + Heap::elems::<Frame>(self.max_stack_depth)
        + Heap::elems::<Option<(Hash, CacheEntry)>>((self.max_store_slots as usize)*2 )
        + (self.max_code_size* virt_factor)
        + (self.max_heap_size* virt_factor)
        + (self.max_transaction_size * virt_factor)
        + self.temporary_buffer
    }
}

pub struct Configuration {
    max_stack_depth:usize,
    max_frame_depth:usize,
    max_heap_size:usize,
    max_script_stack_size:usize,
    max_code_size:usize,
    max_structural_dept: usize,
    max_transaction_size: usize,
    max_store_slots: u16,
    temporary_buffer: usize
}

//A struct holding context information of the current transaction
#[derive(Copy, Clone, Debug)]
pub struct ContextEnvironment {
    pub block_no:u64,
    pub full_hash:Hash,
    pub txt_hash:Hash,
    pub code_hash:Hash
}

//Executes a transaction
pub fn execute<S:Store>(store:&S, txt_data:&[u8], block_no:u64, heap:&Heap) -> Result<Hash> {
    //Create Allocator
    let size_script_stack = Heap::elems::<Elem<StackEntry,SlicePtr<usize>>>(CONFIG.max_script_stack_size);
    let size_interpreter_stack = Heap::elems::<Ptr<Object>>(CONFIG.max_stack_depth);
    let size_frame_stack = Heap::elems::<Frame>(CONFIG.max_stack_depth);
    let size_slot_map = Heap::elems::<Option<(Hash, CacheEntry)>>((CONFIG.max_store_slots as usize) *2);
    let size_code_alloc = CONFIG.max_code_size;
    let size_alloc = CONFIG.max_heap_size;
    //Static allocations (could be done once)
    // A buffer to parse the transaction
    let txt_alloc = heap.new_virtual_arena(CONFIG.max_transaction_size); //todo: will be static conf later (or block consensus)
    //A Buffer to pass dynamically sized vectors
    //Note this is static: it can hold: a type buffer + a param buffer + a return buffer (each max 255 elems)
    //                  or it can hold a ctr field buffer (max 255 values)
    let temporary_values = heap.new_arena(CONFIG.temporary_buffer); //todo: will be dynamic | static  later
    //Parse the transaction
    let txt:Transaction = Parser::parse_fully(txt_data, CONFIG.max_structural_dept, &txt_alloc)?;

    //create heaps: based on txt input
    let alloc = heap.new_virtual_arena(size_alloc); //todo: will be dynamic later
    let code_alloc = heap.new_virtual_arena(size_code_alloc); //todo: will be dynamic
    let structural_arena = heap.new_arena(size_slot_map+size_script_stack+size_interpreter_stack+size_frame_stack);
    let slot_map = CacheSlotMap::new(&structural_arena, CONFIG.max_store_slots,(0,0,0))?; //todo: will be dynamic & random
    let script_stack = structural_arena.alloc_stack(CONFIG.max_script_stack_size); //todo: will be dynamic

    //check that there are enough signatures on it
    if txt.signers.len() != txt.signatures.len() {
        return signature_error()
    }
    //check that the transaction is not out of date
    if txt.start_block_no > block_no || txt.start_block_no+100 < block_no {
        unimplemented!()
    }

    //find the start of each of the transaction hashes
    //note this is serialize format dependent
    let code_start = 8 + 2 + txt.signers.len()*32;                  //8 is blockNo overhead, 2 is num signers
    let sigs_start = txt_data.len() - txt.signatures.len()*64 - 2;  //1 is num sigs
    //hash over everything
    //todo: alloc  & store ptrs
    let full_hash = hash(&txt_data);
    //hash over everything except signatures
    let txt_hash = hash(&txt_data[..sigs_start]);
    //hash over the code only
    let code_hash = hash(&txt_data[code_start..sigs_start]);

    //todo: check if txt_hash already included in last 100 blocks

    //extract the account types and check the sigantures
    let mut accounts = alloc.slice_builder(txt.signers.len())?;
    for (sig,pk) in txt.signatures.iter().zip(txt.signers.iter()) {
        /*//parse the public key
        let rpk = match PublicKey::from_bytes(pk){
            Ok(r) => r,
            Err(_) => return signature_error(),
        };
        //parse the signature
        let rsig = match Signature::from_bytes(sig){
            Ok(r) => r,
            Err(_) => return signature_error(),
        };
        //Check that the signatur eis from that key
        //sha 256 for simplicity later use blake512 <-- needs digest impl
        match rpk.verify::<Sha512>(&txt_data[..sigs_start], &rsig) {
            Ok(_) => {},
            Err(_) => return signature_error(),
        }
        //hash the pk to get the address
        */
        accounts.push(alloc.alloc(RuntimeType::AccountType { address: hash(pk) })?)
    }

    //create the transaction executor
    let stack = LinearScriptStack::new(&alloc,script_stack)?;
    let mut exec = Executor{
        accounts: accounts.finish(),
        newtypes:Vec::new(),
        stack,
        env:ContextEnvironment {
            block_no,
            full_hash,
            txt_hash,
            code_hash
        },
        store:ElemStore::new(store, slot_map),
        alloc: &alloc,
        code_alloc: &code_alloc,
        stack_alloc: &structural_arena,
    };

    //execute the transaction
    exec.execute(&txt.code, &temporary_values)?;
    Ok(txt_hash)
}


//Helper to calc the input hash
fn hash(data:&[u8]) -> Hash {
    //Make a 20 byte digest hascher
    let mut context = Blake2b::new(20);
    //push the data into it
    context.update(data);
    //calc the Hash
    let hash = context.finalize();
    //generate a array to the hash
    *array_ref!(hash.as_bytes(),0,20)

}