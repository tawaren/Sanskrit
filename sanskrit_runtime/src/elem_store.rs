use sanskrit_common::store::*;
use alloc::collections::BTreeMap;
use sanskrit_common::model::*;
use script_stack::StackEntry;
use sanskrit_common::errors::*;
use model::*;
use sanskrit_common::encoding::*;
use core::mem;
use sanskrit_common::arena::*;

//a struct to hold a Slot
#[derive(Copy, Clone, Debug)]
struct CacheEntry<'a> {
    pub real_store: Option<StoreElem<'a>>,         //the value of the backing store: None means we expect none (but it has to be checked)
    pub txt_store: Option<(StoreElem<'a>, bool)>,  //the value the store has during txt: None means not stored, bool tells if borrowed
}

//Holding all the slots
pub struct ElemStore<'a, S:Store> {
    cache: BTreeMap<Hash,CacheEntry<'a>>,   //the cache holding the interacted entries
    pub backend:&'a S                       //the store holding all the entry
}

impl<'a> Object<'a> {
    //A helper to extract the storage index / slot from an object
    pub fn extract_key(&self) -> &Hash {
        match *self {
            //its always the first field in a ADT (its existence is enforced by the type checker)
            Object::Adt(_, ref fields) => fields[0].extract_key(),
            //found it
            Object::Data(ref data) if data.len() == 20 => array_ref!(data,0,20),
            _ => unreachable!(),
        }
    }
}

//a helper to delay store modifications until we are sure there are no storage failures
// this allowa a transactional behaviour where everithing or nothing is changed
#[derive(Copy, Clone, Debug)]
enum StorageCommand<'a> {
    Store(Hash, StoreElem<'a>),
    Delete(Hash),
    Replace(Hash, StoreElem<'a>),
}


impl<'a,S:Store> ElemStore<'a,S> {
    //Creates a new store instance for this transaction
    pub fn new(store:&'a S) -> Self {
        ElemStore {
            cache: BTreeMap::new(),
            backend: store
        }
    }

    //Loads a store entry
    pub fn load<'h>(&mut self, key:Hash, alloc:&'a VirtualHeapArena<'h>) -> Result<StackEntry<'a>> {
        //check if already in cache
        Ok(match self.cache.get_mut(&key){
            Some(ref mut entry) => {
                //it is in cache
                match entry.txt_store.clone() {
                    // Slot is empty or borrowed and thus can not be loaded
                    None | Some((_, true)) => return item_not_found(),
                    //slot is present
                    Some((res, false)) => {
                        //If it is not copy the slot is emptied on load
                        if !res.typ.get_caps().contains(NativeCap::Copy) {
                            entry.txt_store = None;
                        }
                        StackEntry::new(res.val,res.typ)
                    },
                }
            },
            None => {
                //Slot not cached, look it up in the backing store
                let res:StoreElem = self.backend.parsed_get(StorageClass::Elem,&key, alloc)?;
                //create the result
                let ret = StackEntry::new(res.val,res.typ);
                //calc the current slot cache value
                let txt_store =  if res.typ.get_caps().contains(NativeCap::Copy) {
                    Some((res.clone(),false))
                } else {
                    None
                };
                //update the cache
                self.cache.insert(key, CacheEntry{
                    real_store: Some(res),      //Remember the original Slot
                    txt_store,                  //Remeber the current slot
                });
                ret
            },
        })
    }

    //Borrows a store entry
    pub fn borrow<'h>(&mut self, key:Hash, alloc:&'a VirtualHeapArena<'h>) -> Result<StackEntry<'a>> {
        //check if already in cache
        Ok(match self.cache.get_mut(&key) {
            //it is in cache
            Some(ref mut entry) => {
                match entry.txt_store.clone() {
                    // Slot is empty or already borrowed and thus can not be borrowed
                    None | Some((_, true)) => return item_not_found(),
                    // it is available
                    Some((res, false)) => {
                        //mark borrowed
                        entry.txt_store = Some((res.clone(), true));
                        StackEntry::new_store_borrowed(res.val, res.typ)
                    },
                }
            },
            None => {
                //Slot not cached, look it up in the backing store
                let res:StoreElem = self.backend.parsed_get(StorageClass::Elem,&key, alloc)?;
                //create the result
                let ret = StackEntry::new_store_borrowed(res.val.clone(),res.typ.clone());
                self.cache.insert(key, CacheEntry{
                    real_store: Some(res.clone()),      //Mark the original as existent
                    txt_store: Some((res,true)),        //Mark the current as borrowed
                });
                ret
            },
        })
    }

    //stores an entry
    pub fn store(&mut self, key:Hash, elem:StackEntry<'a>) -> Result<()> {
        //check if already in cache
        match self.cache.get_mut(&key) {
            //it is in cache
            Some(ref mut entry) => {
                match &entry.txt_store {
                    // Slot is borrowed and thus can not be overwritten
                    Some((_, true)) => return item_already_exists(),
                    //Slot is already occupied
                    Some((res, false)) => {
                        //Check if it can be dropped / overwritten
                        if res.typ.get_caps().contains(NativeCap::Drop) {
                            //Overwrite with the new one
                            entry.txt_store = Some((StoreElem{
                                val: elem.val.clone(),
                                typ: elem.typ.clone(),
                            }, false))
                        } else {
                            //error can not drop existing item
                            return item_already_exists()
                        }
                    }
                    None => {
                        //Its empty so just store to it
                        entry.txt_store = Some((StoreElem{
                            val: elem.val.clone(),
                            typ: elem.typ.clone(),
                        }, false))
                    },
                }
            },
            None => {
                //Insert a fresh cache entry
                self.cache.insert(key, CacheEntry{
                    real_store: None,               //remark that the backing slot is expected to be empty
                    txt_store: Some((StoreElem{     //create the cache entry
                        val: elem.val.clone(),
                        typ: elem.typ.clone(),
                    }, false))
                });
            },
        }
        Ok(())
    }

    //free a borrowed slot
    pub fn free(&mut self, key:&Hash) {
        match self.cache.get_mut(key) {
            //it must be in the cache or else it culd not have been borrowed
            Some(ref mut entry) => {
                match &entry.txt_store {
                    //it must be in the cache as borrowed
                    Some((res, true)) => {
                        //add it back as stored
                        entry.txt_store = Some((res.clone(), false));
                    },
                    _ => unreachable!()
                }
            },
            None => unreachable!(),
        }
    }

    //writes the changes in the cache back to the store
    pub fn finish<'h>(&mut self, alloc:&VirtualHeapArena<'h>, temporary_values:&HeapArena<'h>) -> Result<()>{
        //exchange the cache with an empty one (and capture the current)
        let cache = mem::replace(
            &mut self.cache,
            BTreeMap::new()
        );

        let tmp = temporary_values.temp_arena()?;
        //collect all the necessary changes and check if they are valid
        let mut commands = tmp.slice_builder(cache.len())?;
        //go through all entries and check that they are ok
        for (key, entry) in cache {
            //match the state
            match (entry.real_store, entry.txt_store) {
                //everithing should be returned
                (_, Some((_,true))) => unreachable!(), //just to be safe
                //should be empty will be empty
                (None, None) => if self.backend.contains(StorageClass::Elem, &key){
                    //ups was full, check if the slot can be dropped to make it empty
                    let elem:StoreElem = self.backend.parsed_get::<StoreElem, VirtualHeapArena>(StorageClass::Elem, &key, alloc)?;
                    if elem.typ.get_caps().contains(NativeCap::Drop) {
                        //Drop it later
                        commands.push(StorageCommand::Delete(key))
                    } else {
                        //ups was full
                        return item_already_exists()
                    }
                },

                //Should be empty will be full
                (None, Some((val,false))) => if self.backend.contains(StorageClass::Elem, &key){
                    //ups was full, check if the slot can be dropped to make it empty
                    let elem:StoreElem = self.backend.parsed_get::<StoreElem, VirtualHeapArena>(StorageClass::Elem, &key, alloc)?;
                    if elem.typ.get_caps().contains(NativeCap::Drop) {
                        //Overwrite it later
                        commands.push(StorageCommand::Replace(key, val))
                    } else {
                        //ups was full
                        return item_already_exists()
                    }
                } else {
                    //Was empty fill it later
                    commands.push(StorageCommand::Store(key, val))
                },

                //Was full will be empty (So delete it later)
                (Some(_),None) => commands.push(StorageCommand::Delete(key)),
                //Was full will be full so overwrite it
                (Some(a),Some((b,false))) => if a != b {
                    commands.push(StorageCommand::Replace(key, b))
                },
            }
        }

        //all is ok, no errors apply the changes
        for cmd in commands.finish().iter() {
            match cmd {
                StorageCommand::Store(key, val) => self.backend.serialized_set(StorageClass::Elem, key.clone(), val)?,
                StorageCommand::Delete(key) => self.backend.delete(StorageClass::Elem, &key)?,
                StorageCommand::Replace(key, val) => self.backend.serialized_replace(StorageClass::Elem, key.clone(), val)?
            }
        }

        Ok(())
    }
}