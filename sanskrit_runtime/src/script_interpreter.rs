use interpreter::*;
use script_stack::*;
use sanskrit_common::errors::*;
use sanskrit_common::model::*;
use sanskrit_common::linear_stack::*;
use sanskrit_common::store::*;
use sanskrit_common::encoding::*;
use alloc::prelude::*;
use model::*;
use native::*;
use blake2_rfc::blake2b::{Blake2b};
use byteorder::{LittleEndian, ByteOrder};
use elem_store::ElemStore;
use ContextEnvironment;
use sanskrit_common::arena::*;
use CONFIG;

//The state of the script execution
pub struct Executor<'a, 'h, S:Store>{
    pub accounts:SlicePtr<'a,Ptr<'a,RuntimeType<'a>>>,
    pub newtypes:Vec<Ptr<'a,RuntimeType<'a>>>,
    pub stack:LinearScriptStack<'a, 'h>,
    pub env:ContextEnvironment,
    pub store:ElemStore<'a,S>,
    pub alloc:&'a VirtualHeapArena<'h>,
    pub code_alloc:&'a VirtualHeapArena<'h>,
    pub stack_alloc: &'a HeapArena<'h>
}

//Hashing Domains to ensure there are no collisions
pub enum UniqueDomain {
    Unique,
    Singleton
}

//generates a new hash from a hash (usually txtHash) and a counter
pub fn unique_hash<'a, 'h>(base: &Hash, domain:UniqueDomain, ctr: u64, alloc:&'a VirtualHeapArena<'h>) -> Result<Object<'a>> {
    //create the hasher
    let mut context = Blake2b::new(20);
    //prepare the counter
    let mut input = [0; 8];
    LittleEndian::write_u64(&mut input, ctr);
    //prepare the domain
    let dom = match domain {
        UniqueDomain::Unique => 0u8,
        UniqueDomain::Singleton => 1u8,
    };
    //update the hasher with all information
    context.update(&[dom]);
    context.update(&input);
    context.update(base);
    //create the hash
    let hash = context.finalize();
    //generate a array to the hash
    let hash_data_ref = array_ref!(hash.as_bytes(),0,20);
    //get ownership and return
    Ok(Object::Data(alloc.copy_alloc_slice(hash_data_ref)?))
}


//helper th extract a key from a stack entry
// used as input to load & store
fn extract_key(entry:&StackEntry) -> Result<Hash> {
    //ensure it is a ref
    match &*entry.typ {
        RuntimeType::NativeType { typ:NativeType::Ref, .. } => {},
        _ => return type_mismatch()
    }

    //fetch the value
    Ok(match &*entry.val {
        Object::Data(key) => array_ref!(key,0,20).to_owned(),
        _ => unreachable!()
    })
}

impl<'a, 'h, S:Store> Executor<'a,'h,S> {
    //execute all codes
    pub fn execute<'c, 's>(&mut self, codes:&'c [Ptr<'c, ScriptCode>], temporary_values:&'s HeapArena<'h>) -> Result<()> {
        for code in codes {
            match **code {
                ScriptCode::Pack(adt_ref, ref applies, tag, ref vals) => self.pack(adt_ref,applies, tag, vals, false)?,
                ScriptCode::BorrowPack(adt_ref, ref applies, tag, ref vals) => self.pack(adt_ref, applies, tag, vals, true)?,
                ScriptCode::Unpack(adt_ref, tag, val) => self.unpack(adt_ref, val, tag, false, temporary_values)?,
                ScriptCode::BorrowUnpack(adt_ref, tag, val) => self.unpack(adt_ref, val, tag ,true, temporary_values)?,
                ScriptCode::Invoke(func_ref, ref applies, ref vals) => self.invoke(func_ref, applies, vals, temporary_values)?,
                ScriptCode::Lit(ref data, desc) => self.lit(data,desc)?,
                ScriptCode::Copy(val) => self.copy(val)?,
                ScriptCode::Fetch(val) => self.fetch(val, false)?,
                ScriptCode::BorrowFetch(val) => self.fetch(val, true)?,
                ScriptCode::Free(val) => self.free(val)?,
                ScriptCode::Drop(val) => self.drop(val)?,
                ScriptCode::Load(val) => self.load(val)?,
                ScriptCode::BorrowLoad(val) => self.borrow_load(val)?,
                ScriptCode::Store(val) => self.store(val)?,
                ScriptCode::NewType => self.new_type()?,
            }
        }
        //first clean up the stack (may release elems from store)
        self.stack.checked_clean_up(&mut self.store)?;
        //second sync store with backend and clean it up
        self.store.finish(self.alloc, temporary_values)?;
        Ok(())
    }

    //helper to resolve a type
    fn resolve_type(&mut self, tar:&TypeApplyRef) -> Result<(bool, Ptr<'a,RuntimeType<'a>>)>{
        match tar {
            //if it is an account type fetch it from th executor state
            TypeApplyRef::Account(a_idx) => {
                if *a_idx as usize >= self.accounts.len() { return type_index_error()}
                Ok((true,self.accounts[*a_idx as usize].clone()))
            },
            TypeApplyRef::RemoteAccount(ref addr) => {
                Ok((false,self.alloc.alloc(RuntimeType::AccountType {
                    address: addr.clone()
                })?))
            }
            //if it is an new type fetch it from th executor state
            TypeApplyRef::NewType(n_idx) => {
                if *n_idx as usize >= self.newtypes.len() { return type_index_error()}
                Ok((true,self.newtypes[*n_idx as usize].clone()))
            },
            TypeApplyRef::RemoteNewType(ref txt, offset) => {
                Ok((false,self.alloc.alloc(RuntimeType::NewType {
                    txt: txt.clone(),
                    offset: *offset
                })?))
            },
            //if it is an ref to a value extract the type of that value from the stack
            TypeApplyRef::Value(idx) => Ok((false,self.stack.value_of(*idx)?.typ.clone())),
            //if it is a native resolve the applies and construct the type
            TypeApplyRef::Native(ref typ, ref applies) => {
                let b_applies = self.alloc.iter_result_alloc_slice(applies.iter().map(|appl|self.resolve_type(appl).map(|r|r.1)))?;
                let code_alloc = self.code_alloc.temp_arena()?;
                let res = (false,to_runtime_type(*typ,b_applies, self.alloc, &code_alloc)?);
                Ok(res)
            },
            //if it is an adt resolve the applies and construct the type over a descriptor
            TypeApplyRef::Module(ref hash, ref applies) => {
                let b_applies = self.alloc.iter_result_alloc_slice(applies.iter().map(|appl|self.resolve_type(appl).map(|r|r.1)))?;
                let code_alloc = self.code_alloc.temp_arena()?;
                let desc = self.store.backend.parsed_get::<AdtDescriptor, VirtualHeapArena>(StorageClass::AdtDesc, hash, CONFIG.max_structural_dept, &code_alloc)?;
                let res = (false,desc.build_type(b_applies, self.alloc)?);
                Ok(res)
            }
        }
    }

    fn extract_desc<'b,T:Parsable<'b>>(&self, val:ValueRef, class:StorageClass, code_alloc:&'b VirtualHeapArena<'h>) -> Result<T> {
        let res = self.stack.value_of(val)?;
        let hash = match *res.typ {
            RuntimeType::NativeType {typ:NativeType::Data(20), ..} => match *res.val{
                Object::Data(ref data) => array_ref!(data,0,20),
                _ => unreachable!()
            }
            _ => return type_mismatch()
        };
        self.store.backend.parsed_get::<T, VirtualHeapArena>(class, hash, CONFIG.max_structural_dept, code_alloc)
    }

    fn pack<'c>(&mut self, adt_ref:AdtRef, applies:&'c [Ptr<'c,TypeApplyRef>], tag:Tag, vals:&'c [ValueRef], is_borrowed:bool) -> Result<()> {
        let types = self.alloc.iter_result_alloc_slice(applies.iter().map(|t_ref| self.resolve_type(t_ref).map(|t|t.1)))?;
        let code_alloc = self.code_alloc.temp_arena()?;
        let desc = match adt_ref {
            AdtRef::Dynamic(val) => self.extract_desc(val, StorageClass::AdtDesc, &code_alloc)?,
            AdtRef::Ref(hash) => self.store.backend.parsed_get::<AdtDescriptor, VirtualHeapArena>(StorageClass::AdtDesc, &hash, CONFIG.max_structural_dept, &code_alloc)?,
            AdtRef::Native(typ) => typ.get_native_adt_descriptor(&code_alloc)?,
        };
        desc.pack(types, tag, &vals, is_borrowed, &mut self.stack, self.alloc)?;
        Ok(())
    }

    fn unpack(&mut self, adt_ref:AdtRef, val:ValueRef, expected_tag:Tag, is_borrowed:bool, temporary_values:&HeapArena<'h>) -> Result<()> {
        let code_alloc = self.code_alloc.temp_arena()?;
        let desc = match adt_ref {
            AdtRef::Dynamic(val) => self.extract_desc(val, StorageClass::AdtDesc, &code_alloc)?,
            AdtRef::Ref(hash) => self.store.backend.parsed_get::<AdtDescriptor, VirtualHeapArena>(StorageClass::AdtDesc, &hash, CONFIG.max_structural_dept, &code_alloc)?,
            AdtRef::Native(typ) => typ.get_native_adt_descriptor(&code_alloc)?,
        };
        desc.unpack(val, expected_tag, is_borrowed, &mut self.stack, self.alloc, temporary_values)?;
        Ok(())
    }

    fn invoke<'c>(&mut self, func_ref:FuncRef, applies:&'c[Ptr<'c,TypeApplyRef>], vals:&'c[ValueRef], temporary_values:&HeapArena<'h>) -> Result<()> {
        let tmp = temporary_values.temp_arena();
        let types = tmp.iter_result_alloc_slice(applies.iter().map(|t_ref| self.resolve_type(t_ref)))?;
        let code_alloc = self.code_alloc.temp_arena()?;
        let desc = match func_ref {
            FuncRef::Dynamic(val) => self.extract_desc(val, StorageClass::FunDesc, &code_alloc)?,
            FuncRef::Ref(ref hash) => self.store.backend.parsed_get::<FunctionDescriptor, VirtualHeapArena>(StorageClass::FunDesc, hash, CONFIG.max_structural_dept, &code_alloc)?,
        };
        desc.apply(&types, &vals, &mut self.stack, self.env, self.alloc, &self.stack_alloc, &tmp)?;
        Ok(())
    }

    fn lit(&mut self, data:&[u8], desc:LitDesc) -> Result<()>{
        let val = create_lit_object(&data,desc, self.alloc)?;
        let typ = desc.lit_typ(data.len() as u16, self.alloc)?;
        self.stack.provide(StackEntry::new( val, typ ))
    }

    fn copy(&mut self, vl:ValueRef) -> Result<()> {
        let typ = &self.stack.value_of(vl)?.typ;
        if !typ.get_caps().contains(NativeCap::Copy) {
            return capability_missing_error()
        }
        self.stack.fetch(vl, FetchMode::Copy)?;
        Ok(())
    }

    fn fetch(&mut self, vl:ValueRef, is_borrowed:bool) -> Result<()> {
        let mode = if is_borrowed {
            FetchMode::Borrow
        } else {
            FetchMode::Consume
        };
        self.stack.fetch(vl, mode)?;
        Ok(())
    }

    fn free(&mut self, v1:ValueRef) -> Result<()> {
        if !self.stack.is_borrowed(v1)? {
            return borrow_missing()
        }
        let freed = self.stack.value_of(v1)?;
        if freed.store_borrow {
            let key = freed.val.extract_key();
            self.store.free(key)
        }
        self.stack.free(v1)?;
        Ok(())
    }

    fn drop(&mut self, vl:ValueRef) -> Result<()> {
        let typ = &self.stack.value_of(vl)?.typ;
        if !typ.get_caps().contains(NativeCap::Drop) {
            return capability_missing_error()
        }
        self.stack.drop(vl)?;
        Ok(())
    }

    fn load(&mut self, v1:ValueRef) -> Result<()> {
        let key = extract_key(&self.stack.value_of(v1)?)?;
        let res = self.store.load(key, self.alloc)?;
        self.stack.provide(res)
    }

    fn borrow_load(&mut self, v1:ValueRef) -> Result<()> {
        let key = extract_key(&self.stack.value_of(v1)?)?;
        let res = self.store.borrow(key, self.alloc)?;
        self.stack.store_borrow(res)
    }

    fn store(&mut self, vl:ValueRef) -> Result<()> {
        let entry = self.stack.value_of(vl)?;

        let caps = entry.typ.get_caps();
        if !caps.contains(NativeCap::Persist) || !caps.contains(NativeCap::Indexed){
            return capability_missing_error()
        }

        let key = entry.val.extract_key();
        self.store.store(*key,entry.clone())?;
        self.stack.consume(vl)
    }

    fn new_type(&mut self) -> Result<()> {
        if self.newtypes.len() > u8::max_value() as usize {
            return size_limit_exceeded_error();
        }
        let offset = self.newtypes.len() as u8;
        let n_type = self.alloc.alloc(RuntimeType::NewType {
            txt: self.env.txt_hash.clone(),
            offset,
        })?;
        self.newtypes.push(n_type);
        //create and push singleton
        let val = unique_hash(&self.env.txt_hash, UniqueDomain::Singleton, offset as u64, self.alloc)?;
        let singleton = StackEntry::new(
            self.alloc.alloc(val)?,
            self.alloc.alloc(RuntimeType::NativeType {
                caps: NativeType::Singleton.base_caps(), //ok as n_type is phantom
                typ: NativeType::Singleton,
                applies: self.alloc.copy_alloc_slice(&[n_type])?
            })?
        );
        self.stack.provide(singleton)
    }

}
