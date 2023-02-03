#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_btree_new)]
extern crate frame_support;
extern crate frame_system;
extern crate sanskrit_common;
extern crate sanskrit_runtime;
extern crate sanskrit_interpreter;
extern crate sanskrit_core;
extern crate sanskrit_deploy;
extern crate sanskrit_compile;

extern crate byteorder;
extern crate ed25519_dalek;
extern crate sha2;
extern crate rand;

pub mod externals;
pub mod chain_store;

use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, StorageMap};
use frame_support::sp_std::vec::Vec;
use frame_support::sp_std::convert::TryInto;

use frame_system::{self as system, ensure_signed, ensure_root};
use sanskrit_common::model::{Hash, SlicePtr};
use sanskrit_common::store::{StorageClass, store_hash};
use sanskrit_runtime::model::{DeployTransaction, DeployType, BundleSection, Transaction, ParamRef, RetType};
use sanskrit_common::encoding::*;
use sanskrit_common::arena::Heap;
use sanskrit_common::errors::*;
use sanskrit_runtime::{CONFIG, deploy, execute, Context, Tracker};
use externals::{ServerExternals, ServerSystem};
use chain_store::{ChainStore, Limit};
use sanskrit_runtime::system::SystemContext;
use sanskrit_interpreter::model::{TxTParam, Entry, TxTReturn};

// 2. Configuration
/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;
}

// 3. Storage
// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    trait Store for Module<T: Trait> as SanskritModule {
        Modules: map hasher(identity) Hash => Vec<u8>;
        Transactions: map hasher(identity) Hash => Vec<u8>;
        Descriptors: map hasher(identity) Hash => Vec<u8>;
        EntryValues: map hasher(identity) Hash => Vec<u8>;
        EntryHashes: map hasher(identity) Hash => Vec<u8>;
    }
}

// 4. Events
// Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event! {
    pub enum Event {
       Placeholder,
    }
}
// 5. Errors
// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
       Placeholder,
       DeploymentFailed,
       ExecutionFailed,
    }
}

// 6. Callable Functions
// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight = 10_000]
        fn deploy_module(origin, /*weight info*/ data: Vec<u8>) {
            // Check that the extrinsic was signed
            ensure_signed(origin)?;
            let res = deploy_module_helper(data, DeployType::Module, false);
            ensure!(res.is_ok(),Error::<T>::DeploymentFailed)
        }

        #[weight = 10_000]
        fn deploy_system_module(origin, /*weight info*/ data: Vec<u8>) {
            // Check that the extrinsic was sent by a root
            ensure_root(origin)?;
            let res = deploy_module_helper(data, DeployType::Module, true);
            ensure!(res.is_ok(),Error::<T>::DeploymentFailed)
        }

        #[weight = 10_000]
        fn deploy_transaction(origin, /*weight info*/ data: Vec<u8>) {
            // Check that the extrinsic was signed
            ensure_signed(origin)?;
            let res = deploy_module_helper(data, DeployType::Transaction, false);
            ensure!(res.is_ok(),Error::<T>::DeploymentFailed)
        }

        #[weight = 10_000]
        fn deploy_system_transaction(origin, /*weight info*/ data: Vec<u8>) {
            // Check that the extrinsic was signed
            ensure_root(origin)?;
            let res = deploy_module_helper(data, DeployType::Transaction, true);
            ensure!(res.is_ok(),Error::<T>::DeploymentFailed)
        }


        #[weight = 10_000]
        fn execute_transaction(origin, /*weight info*/ data: Vec<u8>) {
            // Check that the extrinsic was signed.
            ensure_signed(origin)?;
            //todo: fill from weight Info
            let read = Limit{
                invokes: 0,
                bytes: 0
            };
            let write = Limit{
                invokes: 0,
                bytes: 0
            };

            let block_no = match TryInto::<u64>::try_into(<system::Module<T>>::block_number()){
                Ok(bl) => bl,
                Err(_) => Err(Error::<T>::ExecutionFailed)?
            };
            let res = execute_bundle(&data, read, write, block_no);
            ensure!(res.is_ok(),Error::<T>::ExecutionFailed)
        }

        //	#[transactional] <-- revrts if error returned
    }
}

fn contains_key(class:StorageClass, key:&Hash) -> bool {
    match class {
        StorageClass::Module => !Modules::get(key).is_empty(),
        StorageClass::Transaction => !Transactions::get(key).is_empty(),
        StorageClass::Descriptor => !Descriptors::get(key).is_empty(),
        StorageClass::EntryHash => !EntryHashes::get(key).is_empty(),
        StorageClass::EntryValue => !EntryValues::get(key).is_empty(),
    }
}

/*
Info from Contract module to read code: n = code_size/1024
//Note: this may include code
(195_276_000 as Weight)
    .saturating_add((35_000 as Weight).saturating_mul(n as Weight))
    .saturating_add(T::DbWeight::get().reads(6 as Weight))
    .saturating_add(T::DbWeight::get().writes(3 as Weight))
RES: c.a: 35 / Byte <- neglectable
(processing is 20_000 / Byte -- see worst case bench)
As our CPU 2x slowe: 10_000 /Byte <- go with that
//Meaning: processing does Dominate
 */

fn get(class:StorageClass, key:&Hash) -> Option<Vec<u8>> {
    let res = match class {
        StorageClass::Module => Modules::get(key),
        StorageClass::Transaction => Transactions::get(key),
        StorageClass::Descriptor => Descriptors::get(key),
        StorageClass::EntryHash => EntryHashes::get(key),
        StorageClass::EntryValue => EntryValues::get(key),
    };
    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}

fn remove(class:StorageClass, key:&Hash) {
    match class {
        StorageClass::Module => Modules::remove(key),
        StorageClass::Transaction => Transactions::remove(key),
        StorageClass::Descriptor => Descriptors::remove(key),
        StorageClass::EntryHash => EntryHashes::remove(key),
        StorageClass::EntryValue => EntryValues::remove(key),
    }
}

/*
Info from Contract module to put code: n = code_size/1024
 This includes:
   Validating & Injecting Gas
   Write Original
   Write Injected
   ---------------------------------
   If we build based on read 10x => 350 / byte then process would dominate
   ---------------------------------
(0 as Weight)
.saturating_add((109_242_000 as Weight).saturating_mul(n as Weight))
.saturating_add(T::DbWeight::get().reads(1 as Weight))
.saturating_add(T::DbWeight::get().writes(2 as Weight))
Say: 50% is processing then 1 store = 25% = 25_000 / Byte <-- Educated guess
 (processing is 75_000 / Byte --> ~40_000 as we or CPU is 2x slower as ref)
Res: c.a 65_000 / Byte
 Note: T::DbWeight::get().writes(1) is another 100_000_000 (or 50_000_000 in case patity DB)
 (On a max module with size: 256_000 -> ~400 / Byte) -- neglectable (on per byte basis)
  (And on micro module 256 -> ~400_000 -- tot: ~550_000 / Byte (super dominance of Disk)
 Note: our 2 contains checks are reads: T::DbWeight::get().reads(2)

 Alternative (Worst Case): Say 0% Validation -> 50_000 + 40_000 -> 90_000 / byte
 Average: (90_000 + 65_000)/2 => ~80_000 / byte <-- we will go with this for now (is eq to 20% validation / 80% storage)
 Best case (Storage = 10x Read) => ~40_000 / byte <-- so we may be up to 50% off (in addition to 10x off due to worst case --meaning 20x off)

 But Better Save than sorry

 */

fn insert(class:StorageClass, key:Hash, data:Vec<u8>) {
    match class {
        StorageClass::Module => Modules::insert(key,data),
        StorageClass::Transaction => Transactions::insert(key,data),
        StorageClass::Descriptor => Descriptors::insert(key,data),
        StorageClass::EntryHash => EntryHashes::insert(key,data),
        StorageClass::EntryValue => EntryValues::insert(key,data),
    }
}

fn deploy_module_helper(/*weight info*/ data: Vec<u8>, mode:DeployType, system_mode:bool) -> Result<Hash>{
    //todo: fill from weight Info
    let read = Limit{
        invokes: 0,
        bytes: 0
    };
    let write = Limit{
        invokes: 0,
        bytes: 0
    };
    let txt = DeployTransaction{
        typ: mode,
        data: SlicePtr::wrap(&data)
    };
    execute_deploy(txt, read, write, system_mode)
}


fn execute_deploy(txt:DeployTransaction, read:Limit, write:Limit, system_mode_on:bool) -> Result<Hash> {
    let mut s = Serializer::new(usize::max_value());
    txt.serialize(&mut s)?;
    let heap = Heap::new(CONFIG.calc_heap_size(2),2.0);
    let store = ChainStore::new(read, write);
    deploy::<_, ServerExternals>(&store, &s.extract(), &heap, system_mode_on)
}

//Todo: Make usefull one
struct NoLogger{}
impl Tracker for NoLogger {
    fn section_start(&mut self, _section: &BundleSection) {}
    fn transaction_start(&mut self, _transaction: &Transaction) {}
    fn parameter_load(&mut self, _p_ref: &ParamRef, _p_desc: &TxTParam, _value: &Entry) {}
    fn return_value(&mut self, _r_typ: &RetType, _r_desc: &TxTReturn, _value: &Entry) {}
    fn transaction_finish(&mut self, _transaction: &Transaction, _success: bool) {}
    fn section_finish(&mut self, _section: &BundleSection, _success: bool) {}
}

pub fn execute_bundle(bundle:&[u8], read:Limit, write:Limit, block_no:u64) -> Result<()> {
    let heap = Heap::new(CONFIG.calc_heap_size(2),2.0);
    let txt_bundle_alloc = heap.new_virtual_arena(CONFIG.max_bundle_size);
    let txt_bundle= ServerSystem::parse_bundle(&bundle,&txt_bundle_alloc)?;
    let store = ChainStore::new(read, write);
    let mut tracker = NoLogger{};
    let ctx = Context {
        store: &store,
        txt_bundle: &txt_bundle
    };
    execute::<_, ServerSystem>(ctx,block_no, &heap, &mut tracker)
}