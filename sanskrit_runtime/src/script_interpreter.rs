use interpreter::*;
use script_stack::*;
use sanskrit_common::errors::*;
use sanskrit_common::model::*;
use sanskrit_common::linear_stack::*;
use sanskrit_common::store::*;
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
    pub witness:SlicePtr<'a,SlicePtr<'a,u8>>,
    pub newtypes:SlicePtr<'a,Ptr<'a,RuntimeType<'a>>>,
    pub imports:SlicePtr<'a, Hash>,
    pub stack:LinearScriptStack<'a, 'h>,
    pub env:ContextEnvironment,
    pub store:ElemStore<'a,S>,
    pub alloc:&'a VirtualHeapArena<'h>,
    pub code_alloc:&'a VirtualHeapArena<'h>,
    pub stack_alloc: &'a HeapArena<'h>
}

//Hashing Domains to ensure there are no collisions
pub enum HashingDomain {
    Unique,
    Singleton,
    Index,
    Derive,
    Object,
    Account
}

impl HashingDomain {

    pub fn get_domain_code(&self) -> u8 {
        match *self {
            HashingDomain::Unique => 0,
            HashingDomain::Singleton => 1,
            HashingDomain::Index => 2,
            HashingDomain::Derive => 3,
            HashingDomain::Object => 4,
            HashingDomain::Account => 5,
        }
    }

    pub fn get_domain_hasher(&self, input_len:i32) -> Blake2b {
        let mut context = Blake2b::new(20);
        //prepare the counter
        let mut input = [0u8; 4];
        LittleEndian::write_i32(&mut input, input_len);
        context.update(&[self.get_domain_code()]);
        context.update(&input);
        context
    }
}

//generates a new hash from a hash (usually txtHash) and a counter
pub fn unique_hash<'a, 'h>(base: &Hash, domain: HashingDomain, ctr: u64, alloc:&'a VirtualHeapArena<'h>) -> Result<Object<'a>> {
    //create the hasher
    let mut context = domain.get_domain_hasher(28);
    //prepare the counter
    let mut input = [0u8; 8];
    LittleEndian::write_u64(&mut input, ctr);
    //update the hasher with all information
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

    //helper to resolve keys
    fn elem_key(&self, ImpRef(idx):ImpRef, offset:u8) -> Result<Hash> {
        if idx as usize >= self.imports.len() {return item_not_found()}
        Ok(store_hash(&[&self.imports[idx as usize],&[offset]]))
    }

    fn import(&self, ImpRef(idx):ImpRef) -> Result<Hash> {
        if idx as usize >= self.imports.len() {return item_not_found()}
        Ok(self.imports[idx as usize])
    }

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
                ScriptCode::Wit(wit_ref, desc) => self.wit(wit_ref,desc)?,
                ScriptCode::Copy(val) => self.copy(val)?,
                ScriptCode::Fetch(val) => self.fetch(val, false)?,
                ScriptCode::BorrowFetch(val) => self.fetch(val, true)?,
                ScriptCode::Free(val) => self.free(val)?,
                ScriptCode::Drop(val) => self.drop(val)?,
                ScriptCode::Load(val) => self.load(val)?,
                ScriptCode::BorrowLoad(val) => self.borrow_load(val)?,
                ScriptCode::Store(val) => self.store(val)?,
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
                Ok((true,self.accounts[*a_idx as usize]))
            },
            TypeApplyRef::RemoteAccount(imp) => {
                Ok((false,self.alloc.alloc(RuntimeType::AccountType {
                    address: self.import(*imp)?
                })?))
            }
            //if it is an new type fetch it from th executor state
            TypeApplyRef::NewType(n_idx) => {
                if *n_idx as usize >= self.newtypes.len() { return type_index_error()}
                Ok((true,self.newtypes[*n_idx as usize]))
            },
            TypeApplyRef::RemoteNewType(imp, offset) => {
                Ok((false,self.alloc.alloc(RuntimeType::NewType {
                    txt: self.import(*imp)?,
                    offset: *offset
                })?))
            },
            //if it is an ref to a value extract the type of that value from the stack
            TypeApplyRef::TypeOf(idx) => Ok((false, self.stack.value_of(*idx)?.typ)),

            //if it is a path into a values type, extract it from the values type
            TypeApplyRef::ArgTypeOf(idx, select) => {
                let mut base_typ = self.stack.value_of(*idx)?.typ;
                for s in &**select {
                    match *base_typ {
                        RuntimeType::NativeType { ref applies, ..}
                        | RuntimeType::Custom { ref applies, ..} => {
                            if applies.len() >= *s as usize {return item_not_found()}
                            base_typ = applies[*s as usize]
                        },
                        _ => return item_not_found()
                    };
                }
                Ok((false, base_typ))
            },


            //if it is a native resolve the applies and construct the type
            TypeApplyRef::Native(ref typ, ref applies) => {
                let b_applies = self.alloc.iter_result_alloc_slice(applies.iter().map(|appl|self.resolve_type(appl).map(|r|r.1)))?;
                let code_alloc = self.code_alloc.temp_arena()?;
                let res = (false,to_runtime_type(*typ,b_applies, self.alloc, &code_alloc)?);
                Ok(res)
            },
            //if it is an adt resolve the applies and construct the type over a descriptor
            TypeApplyRef::Module(imp, offset, ref applies) => {
                let b_applies = self.alloc.iter_result_alloc_slice(applies.iter().map(|appl|self.resolve_type(appl).map(|r|r.1)))?;
                let code_alloc = self.code_alloc.temp_arena()?;
                let desc = self.store.backend.parsed_get::<AdtDescriptor, VirtualHeapArena>(StorageClass::AdtDesc, &self.elem_key(*imp, *offset)?, CONFIG.max_structural_dept, &code_alloc)?;
                let res = (false,desc.build_type(b_applies, self.alloc)?);
                Ok(res)
            }
        }
    }

    fn pack<'c>(&mut self, adt_ref:AdtRef, applies:&'c [Ptr<'c,TypeApplyRef>], tag:Tag, vals:&'c [ValueRef], is_borrowed:bool) -> Result<()> {
        let types = self.alloc.iter_result_alloc_slice(applies.iter().map(|t_ref| self.resolve_type(t_ref).map(|t|t.1)))?;
        let code_alloc = self.code_alloc.temp_arena()?;
        let desc = match adt_ref {
            AdtRef::Ref(imp, offset) => self.store.backend.parsed_get::<AdtDescriptor, VirtualHeapArena>(StorageClass::AdtDesc, &self.elem_key(imp,offset)?, CONFIG.max_structural_dept, &code_alloc)?,
            AdtRef::Native(typ) => typ.get_native_adt_descriptor(&code_alloc)?,
        };
        desc.pack(types, tag, &vals, is_borrowed, &mut self.stack, self.alloc)
    }

    fn unpack(&mut self, adt_ref:AdtRef, val:ValueRef, expected_tag:Tag, is_borrowed:bool, temporary_values:&HeapArena<'h>) -> Result<()> {
        let code_alloc = self.code_alloc.temp_arena()?;
        let desc = match adt_ref {
            AdtRef::Ref(imp, offset) => self.store.backend.parsed_get::<AdtDescriptor, VirtualHeapArena>(StorageClass::AdtDesc, &self.elem_key(imp,offset)?, CONFIG.max_structural_dept, &code_alloc)?,
            AdtRef::Native(typ) => typ.get_native_adt_descriptor(&code_alloc)?,
        };
        desc.unpack(val, expected_tag, is_borrowed, &mut self.stack, self.alloc, temporary_values)
    }

    fn invoke<'c>(&mut self, func_ref:FuncRef, applies:&'c[Ptr<'c,TypeApplyRef>], vals:&'c[ValueRef], temporary_values:&HeapArena<'h>) -> Result<()> {
        let tmp = temporary_values.temp_arena();
        let types = tmp.iter_result_alloc_slice(applies.iter().map(|t_ref| self.resolve_type(t_ref)))?;
        let code_alloc = self.code_alloc.temp_arena()?;
        let desc = self.store.backend.parsed_get::<FunctionDescriptor, VirtualHeapArena>(StorageClass::FunDesc, &self.elem_key(func_ref.module, func_ref.offset)?, CONFIG.max_structural_dept, &code_alloc)?;
        desc.apply(&types, &vals, &mut self.stack, self.env, self.alloc, &self.stack_alloc, &tmp)
    }

    fn lit(&mut self, data:&[u8], desc:LitDesc) -> Result<()>{
        let val = create_lit_object(&data,desc, self.alloc)?;
        let typ = desc.lit_typ(data.len() as u16, self.alloc)?;
        self.stack.provide(StackEntry::new( val, typ ))
    }

    fn wit(&mut self, data_ref:u8, desc:LitDesc) -> Result<()>{
        if data_ref as usize >= self.witness.len() {return item_not_found()}
        let data = self.witness[data_ref as usize];
        self.lit(&data,desc)
    }

    fn copy(&mut self, vl:ValueRef) -> Result<()> {
        let typ = &self.stack.value_of(vl)?.typ;
        if !typ.get_caps().contains(NativeCap::Copy) {
            return capability_missing_error()
        }
        self.stack.fetch(vl, FetchMode::Copy)
    }

    fn fetch(&mut self, vl:ValueRef, is_borrowed:bool) -> Result<()> {
        let mode = if is_borrowed {
            FetchMode::Borrow
        } else {
            FetchMode::Consume
        };
        self.stack.fetch(vl, mode)
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
        self.stack.free(v1)
    }

    fn drop(&mut self, vl:ValueRef) -> Result<()> {
        let typ = &self.stack.value_of(vl)?.typ;
        if !typ.get_caps().contains(NativeCap::Drop) {
            return capability_missing_error()
        }
        self.stack.drop(vl)
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
        self.store.store(*key,entry)?;
        self.stack.consume(vl)
    }

}
