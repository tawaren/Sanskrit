use sanskrit_common::model::Hash;
use sanskrit_common::store::*;
use sanskrit_common::errors::*;
use frame_support::sp_std::collections::btree_map::BTreeMap;
use frame_support::sp_std::cell::RefCell;
use frame_support::sp_std::mem;
use ::{contains_key, get};
use ::{remove, insert};
use frame_support::sp_std::vec::Vec;


#[derive(Clone, Debug)]
struct Container {
    class:StorageClass,
    pending:BTreeMap<Hash, Option<Vec<u8>>>
}

impl Container {
    pub fn new(class:StorageClass)-> Self{
        Container{
            class,
            pending: BTreeMap::new()
        }
    }

    pub fn get(&self, key:&Hash) -> Result<Vec<u8>> {
        match self.pending.get(key) {
            None => match get(self.class, key){
                None => error(||"Value was not in store"),
                Some(res) => Ok(res)
            },
            Some(None) => error(||"Value was not in store"),
            Some(Some(res)) => Ok(res.clone()),
        }
    }

    pub fn insert(&mut self, key:Hash, value:Vec<u8>) -> Result<()>  {
        //todo: can we remove contains check??? -- whats the risk??
        if self.pending.contains_key(&key) || !contains_key(self.class, &key) {
            match self.pending.insert(key, Some(value)) {
                None | Some(None)=> Ok(()),
                Some(Some(_)) => error(||"Value was already in store")
            }
        } else  {
            error(||"Value was already in store")
        }
    }


    pub fn remove(&mut self, key:&Hash) -> Result<()> {
        let res = match self.pending.get(key) {
            None => true, //we assume it does, worst case we overwrite -- we charged for overwrite anyway //contains_key(self.class, &key),
            Some(None) => false,
            Some(Some(_)) => true
        };

        if res {
            self.pending.insert(key.clone(), None);
            Ok(())
        } else {
            error(||"Value was not in store")
        }
    }

    pub fn commit(&mut self) {
        let mut res = BTreeMap::new();
        mem::swap(&mut res, &mut self.pending);
        for (key, value) in res {
            match value {
                None => remove(self.class,&key),
                Some(data) => insert(self.class, key, data)
            };
        }
    }

    pub fn rollback(&mut self){
        self.pending.clear();
    }
}

//A container for the different storage sections
// represented as BTreeMaps (as they work with alloc and HashMap seem not to)
#[derive(Clone, Debug)]
pub struct InnerChainStore {
    read:Limit,
    write:Limit,
    hashs:Container,   //All the Modules (Serialized)
    modules:Container,   //All the Modules (Serialized)
    funs:Container,      //All the top level Functions(Serialized)
    descs:Container,     //All the top level Functions(Serialized)
    elems:Container,     //All the Elements (Serialized)
}

//A BTreeMap backed store for development
#[derive(Clone, Debug)]
pub struct ChainStore(RefCell<InnerChainStore>);

#[derive(Clone, Debug)]
pub struct Limit{
    pub invokes:u16,
    pub bytes:u32
}

impl ChainStore {
    //creates a single thread enabled store with inner mutability
    pub fn new(read:Limit, write:Limit)-> Self{
        ChainStore(
            RefCell::new(InnerChainStore {
                read,
                write,
                hashs: Container::new(StorageClass::EntryHash),
                modules: Container::new( StorageClass::Module),
                funs: Container::new( StorageClass::Transaction),
                descs: Container::new(StorageClass::Descriptor),
                elems: Container::new(StorageClass::EntryHash),
            })
        )
    }
}

impl Store for ChainStore {
    //delete a store entry
    fn delete(&self, class: StorageClass, key: &[u8; 20])  -> Result<()>  {
        {
            let mut write_limit = &mut self.0.borrow_mut().write;
            if write_limit.invokes == 0 {return error(||"To many writes")}
            write_limit.invokes -= 1;
        }

        fn process(map: &mut Container, key:&Hash) -> Result<()>  {
            //remove but gives an error if not their
            map.remove(key)
        }
        //select the right map
        match class {
            StorageClass::Module => process(&mut self.0.borrow_mut().modules,key),
            StorageClass::Transaction => process(&mut self.0.borrow_mut().funs, key),
            StorageClass::Descriptor => process(&mut self.0.borrow_mut().descs, key),
            StorageClass::EntryValue => process(&mut self.0.borrow_mut().elems, key),
            StorageClass::EntryHash => process(&mut self.0.borrow_mut().hashs, key),
        }
    }


    //Gets a value out and uses P as Parser
    fn get<P,F:FnOnce(&[u8])-> P>(&self, class:StorageClass, key: &Hash, f:F) -> Result<P> {
        let bytes_left = {
            let mut read_limit = &mut self.0.borrow_mut().read;
            if read_limit.invokes == 0 {return error(||"To many reads")}
            read_limit.invokes -= 1;
            read_limit.bytes as usize
        };

        fn process<P,F:FnOnce(&[u8])-> P>(bytes_left:usize, map: &Container, key:&Hash, f:F) -> Result<(usize,P)> {
            //get the key if available, else gives an error
            let data = &map.get(key)?;
            if bytes_left < data.len() {return error(||"To many bytes read")}
            Ok((data.len(), f(data)))
        }

        //select the right map
        let (data_size,res) = match class {
            StorageClass::Module => process(bytes_left, &self.0.borrow().modules,key,f),
            StorageClass::Transaction => process(bytes_left, &self.0.borrow().funs, key, f),
            StorageClass::Descriptor => process(bytes_left, &self.0.borrow().descs, key, f),
            StorageClass::EntryValue => process(bytes_left, &self.0.borrow().elems, key, f),
            StorageClass::EntryHash => process(bytes_left, &self.0.borrow().hashs, key, f),
        }?;

        //this is checked a bit late
        let mut read_limit = &mut self.0.borrow_mut().read;
        read_limit.bytes -= data_size as u32;
        Ok(res)
    }

    //Stores a value in the store
    fn set(&self, class:StorageClass, key:Hash, data: Vec<u8>) -> Result<()> {
        {
            let mut write_limit = &mut self.0.borrow_mut().write;
            if write_limit.invokes == 0 {return error(||"To many writes")}
            if write_limit.bytes as usize > data.len() {return error(||"To many bytes written")}
            write_limit.invokes -= 1;
            write_limit.bytes -= data.len() as u32;
        }

        fn process(map:&mut Container, key:Hash, data: Vec<u8>) -> Result<()> {
            //insert but gives an error if already in
            map.insert(key, data)
        }

        match class {
            StorageClass::Module => process(&mut self.0.borrow_mut().modules,key,data),
            StorageClass::Transaction => process(&mut self.0.borrow_mut().funs, key, data),
            StorageClass::Descriptor => process(&mut self.0.borrow_mut().descs, key, data),
            StorageClass::EntryValue => process(&mut self.0.borrow_mut().elems, key, data),
            StorageClass::EntryHash => process(&mut self.0.borrow_mut().hashs, key, data),
        }
    }

    fn commit(&self, class: StorageClass) {
        fn process(map:&mut Container) {
            map.commit()
        }

        match class {
            StorageClass::Module => process(&mut self.0.borrow_mut().modules),
            StorageClass::Transaction => process(&mut self.0.borrow_mut().funs),
            StorageClass::Descriptor => process(&mut self.0.borrow_mut().descs),
            StorageClass::EntryValue => process(&mut self.0.borrow_mut().elems),
            StorageClass::EntryHash =>  process(&mut self.0.borrow_mut().hashs),
        }
    }

    fn rollback(&self, class: StorageClass) {
        fn process(map:&mut Container)  {
            map.rollback()
        }

        match class {
            StorageClass::Module => process(&mut self.0.borrow_mut().modules),
            StorageClass::Transaction => process(&mut self.0.borrow_mut().funs),
            StorageClass::Descriptor => process(&mut self.0.borrow_mut().descs),
            StorageClass::EntryValue => process(&mut self.0.borrow_mut().elems),
            StorageClass::EntryHash => process(&mut self.0.borrow_mut().hashs),
        }
    }
}
