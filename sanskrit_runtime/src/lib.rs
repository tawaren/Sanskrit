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
use script_stack::StackEntry;
use sanskrit_common::model::SlicePtr;
use sanskrit_common::linear_stack::Elem;
use sanskrit_common::model::Ptr;
use model::Object;
use interpreter::Frame;
use elem_store::CacheSlotMap;
use elem_store::CacheEntry;
use script_interpreter::HashingDomain;
use system::*;
use script_interpreter::unique_hash;


pub mod native;
pub mod descriptors;
pub mod interpreter;
pub mod model;
pub mod script_stack;
pub mod script_interpreter;
pub mod type_builder;
pub mod elem_store;
pub mod encoding;
pub mod system;


pub const CONFIG: Configuration = Configuration {
    max_stack_depth:2048,
    max_frame_depth:512,
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
    pub max_stack_depth:usize,
    pub max_frame_depth:usize,
    pub max_heap_size:usize,
    pub max_script_stack_size:usize,
    pub max_code_size:usize,
    pub max_structural_dept: usize,
    pub max_transaction_size: usize,
    pub max_store_slots: u16,
    pub temporary_buffer: usize
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

    //create the transaction executor
    let mut stack = LinearScriptStack::new(&alloc, script_stack)?;
    //find the start of each of the transaction hashes
    //note this is serialize format dependent
    //todo: update when new format
    // code start should include imports & new_types
    let code_start = 8 + 2 + txt.signers.len()*32;                  //8 is blockNo overhead, 2 is num signers
    let witness_size = txt.witness.iter().map(|w|w.len() + 2).sum::<usize>() + 2;  //2 is num wittness / Num Bytes
    let sigs_start = txt_data.len() - witness_size - txt.signatures.len()*64 - 2;  //2 is num sigs
    //hash over everything
    //todo: alloc  & store ptrs
    let full_hash = hash(&[&[HashingDomain::Transaction.get_domain_code()],&txt_data]);
    //hash over everything except signatures
    let txt_hash = hash(&[&[HashingDomain::Transaction.get_domain_code()],&txt_data[..sigs_start]]);
    //hash over the code only
    let code_hash = hash(&[&[HashingDomain::Transaction.get_domain_code()],&txt_data[code_start..sigs_start]]);

    //todo: check if txt_hash already included in last 100 blocks

    //extract the account types and check the sigantures
    let mut accounts = alloc.slice_builder(txt.signers.len())?;
    for (sig,pk) in txt.signatures.iter().zip(txt.signers.iter()) {
        //parse the public key
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
        let address = hash(&[&[HashingDomain::Account.get_domain_code()],pk]);
        let acc_type = alloc.alloc(RuntimeType::AccountType { address })?;
        let val = Object::Data(alloc.copy_alloc_slice(&address)?);
        let singleton = StackEntry::new(
            alloc.alloc(val)?,
            alloc.alloc(account_type(&alloc,acc_type)?)?
        );
        stack.setup_push(singleton)?;
        accounts.push(acc_type)
    }
    let accounts = accounts.finish();

    let mut newtypes = alloc.slice_builder(txt.new_types as usize)?;
    for offset in 0..txt.new_types {
        let val = unique_hash(&txt_hash, HashingDomain::Singleton, u64::from(offset), &alloc)?;
        let n_type = alloc.alloc(RuntimeType::NewType {
            txt: txt_hash,
            offset,
        })?;
        let singleton = StackEntry::new(
            alloc.alloc(val)?,
            alloc.alloc(singleton_type(&alloc, n_type)?)?
        );
        stack.setup_push(singleton)?;
        newtypes.push(n_type);
    }
    let newtypes = newtypes.finish();

    //add the context on top -- As the Context has the Drop Cap the clean up wil snap it
    push_ctx(&mut stack, &alloc, &full_hash, &txt_hash, &code_hash)?;

    let mut exec = Executor{
        accounts,
        witness: txt.witness,
        newtypes,
        imports: txt.imports,
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
fn hash(data:&[&[u8]]) -> Hash {
    //Make a 20 byte digest hascher
    let mut context = Blake2b::new(20);
    //push the data into it
    for d in data {
        context.update(*d);
    }
    //calc the Hash
    let hash = context.finalize();
    //generate a array to the hash
    *array_ref!(hash.as_bytes(),0,20)

}

pub fn push_ctx<'a,'h>(stack:&mut LinearScriptStack<'a,'h>, alloc:&'a VirtualHeapArena<'h>, full_hash:&Hash, txt_hash:&Hash, code_hash:&Hash) -> Result<()> {
    stack.setup_push(StackEntry::new(
        alloc.alloc(Object::Adt(0,
                alloc.copy_alloc_slice(&[
                    alloc.alloc(Object::Data(alloc.copy_alloc_slice(code_hash)?))?,
                    alloc.alloc(Object::Data(alloc.copy_alloc_slice(txt_hash)?))?,
                    alloc.alloc(Object::Data(alloc.copy_alloc_slice(full_hash)?))?,
                    alloc.alloc(Object::U64(0))?,
                    alloc.alloc(Object::U64(0))?
                ])?
        ))?,
        alloc.alloc(context_type())?
    ))
}