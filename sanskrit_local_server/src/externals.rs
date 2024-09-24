use sanskrit_common::errors::*;
use sanskrit_common::model::{Hash, Ptr};

use sanskrit_common::arena::VirtualHeapArena;
use sanskrit_interpreter::model::{Entry, Adt, RuntimeType};
use sanskrit_runtime::system::SystemContext;

use sanskrit_common::hashing::HashingDomain;
use sanskrit_sled_store::SledStore;
use sanskrit_runtime::direct_stored::{StatefulEntryStoreVerifier, SystemDataManager, StatefulEntryStoreExecutor};
use sanskrit_common::encoding::{VirtualSize, ParserAllocator, Parser};
use sanskrit_runtime::model::{BundleWithHash, BaseTransactionBundle};
use sanskrit_runtime::CONFIG;
use sanskrit_interpreter::externals::{ExecutionInterface};
use sanskrit_default_externals::{SYS_HASH, EDDSA_HASH, ServerExternals};

pub fn get_ed_dsa_module() -> Hash {EDDSA_HASH.lock().unwrap().get()}

pub struct ServerSystemDataManager;
impl<'c> SystemDataManager<BundleWithHash<'c>> for ServerSystemDataManager {

    fn providable_size(typ: Ptr<RuntimeType>) -> Result<u32> {
        match *typ {
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 1 => {
                Ok((Hash::SIZE + 4*Entry::SIZE) as u32)
            }
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 2 => {
                Ok((Hash::SIZE + 2*Entry::SIZE) as u32)
            }
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 3 => {
                error(||"Not supported by this runtime yet")
            }

            _ => return error(||"Provided value parameter must be of a supported type")
        }
    }

    fn providable_gas(typ: Ptr<RuntimeType>) -> Result<u64> {
        match *typ {
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 1 => {
                let hash_alloc = (13 + 20/50) as u64;
                let pack = 13 + (6 as u64);
                Ok(hash_alloc + pack)
            }
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 2 => {
                let hash_alloc = (13 + 20/50) as u64;
                let pack = 13 + (6 as u64);
                let hash_cost = 65;
                Ok(hash_alloc + pack + hash_cost)
            }
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 3 => {
                error(||"Not supported by this runtime yet")
            }
            _ => return error(||"Provided value parameter must be of a supported type")
        }
    }

    fn is_chain_value(typ: Ptr<RuntimeType>) -> bool {
        match *typ {
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 0 => true,
            _ => false
        }
    }

    //This means we can only provide 1 value per Txt
    fn provided_value_key(typ: Ptr<RuntimeType>, section_no:u8,  txt_no:u8, p_num:u8) -> Option<Vec<u8>> {
        match *typ {
            //This means we can only provide 1 value per Txt
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 2 => Some(vec![section_no,txt_no]),
            //For the rest (TxData we can provide as many copies as we want)
            _ => None
        }
    }

    fn create_provided_value<'a, 'h>(bundle: &BundleWithHash, typ: Ptr<RuntimeType>, alloc: &'a VirtualHeapArena<'h>, block_no: u64, section_no:u8,  txt_no:u8, p_num:u8) -> Result<Entry<'a>> {
        match *typ {
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 1 => {
                Ok(Entry{adt: Adt(0,alloc.copy_alloc_slice(&[
                    Entry {data: alloc.copy_alloc_slice(&bundle.bundle_hash)?},
                    Entry {u64: block_no},
                    Entry {u8: section_no},
                    Entry {u8: txt_no},
                ])?)})
            },
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 2 => {
                let mut context = HashingDomain::Derive.get_domain_hasher();
                //fill the hash with bunlde hash value
                context.update(&bundle.bundle_hash);
                //fill the hash with section & txt indexes
                context.update(&[section_no,txt_no]);
                Ok(Entry{adt: Adt(0,alloc.copy_alloc_slice(&[
                    //calc the Hash
                    Entry {data: context.alloc_finalize(&alloc)?},
                    Entry {u64: 0},
                ])?)})
            },
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 3 => {
                error(||"Not supported by this runtime yet")
            },
            _ => error(||"Requested value is not providable")
        }
    }
}

pub struct ServerSystem;
impl<'c> SystemContext<'c> for ServerSystem {
    type RE = ServerExternals;

    type S = SledStore;

    type B = BundleWithHash<'c>;
    type VC = StatefulEntryStoreVerifier<Self::B,ServerSystemDataManager>;
    type EC = StatefulEntryStoreExecutor<Self::B,ServerSystemDataManager>;

    fn parse_bundle<A: ParserAllocator>(data: &[u8], alloc: &'c A) -> Result<Self::B> {
        let txt_bundle:BaseTransactionBundle = Parser::parse_fully(data, CONFIG.max_structural_dept, alloc)?;
        let bundle_hash = HashingDomain::Bundle.hash(&data[..txt_bundle.core.byte_size.unwrap()]);
        Ok(BundleWithHash {
            txt_bundle,
            bundle_hash,
        })
    }
}
