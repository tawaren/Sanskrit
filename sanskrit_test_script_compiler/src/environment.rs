use std::collections::HashMap;
use model::*;
use sanskrit_common::model::*;
use script::ModuleEntry;
use sanskrit_core::model::{TypeImport as RType, ImplLink, CallableImport, Permission, PermRef, PublicImport, PermissionImport};
use sanskrit_core::model::ModRef;
use sanskrit_core::model::CallRef;
use sanskrit_core::model::TypeRef;
use sanskrit_core::model::FuncLink;
use sanskrit_core::model::DataLink;
use sanskrit_common::encoding::ParserAllocator;
use sanskrit_core::model::bitsets::{PermSet, BitSet};
use sanskrit_runtime::model::ParamRef;
use sanskrit_common::arena::HeapArena;


pub struct Region(pub HashMap<Id,usize>);

pub struct Environment<'c,A: ParserAllocator> {
    pub stack_depth:usize,
    pub frames:Vec<Region>,
    pub alloc:&'c A,
}

#[derive(Eq, PartialEq, Clone, Hash, Copy)]
pub enum Perm {
    Call,
    DataCreate,
    LitCreate,
    Inspect,
    Consume,
    Implement
}

impl<'c,A: ParserAllocator> Environment<'c,A> {

    pub fn new(alloc:&'c A) -> Self {
        Environment {
            stack_depth: 0,
            frames: vec![Region(HashMap::new())],
            alloc
        }
    }

    pub fn new_with_ctx(alloc:&'c A) -> Self {
        let mut default = HashMap::new();
        default.insert(Id("ctx".into()),0);
        Environment {
            stack_depth: 1,
            frames: vec![Region(default)],
            alloc
        }
    }

    pub fn get_id_pos(&mut self, var:&Id) -> Option<ValueRef> {
        for reg in self.frames.iter().rev() {
           match reg.0.get(var)  {
               None => {},
               Some(pos) => {
                   let rel_pos = self.stack_depth - *pos -1;
                   assert!(rel_pos <= u16::max_value() as usize);
                   return Some(ValueRef(rel_pos as u16));
               },
           }
        }
        None
    }

    pub fn push_new(&mut self, var:Id){
        let pos = self.stack_depth;
        self.stack_depth+=1;
        let len = self.frames.len();
        self.frames[len-1].0.insert(var,pos);
    }
}

pub struct CodeImportBuilder<'a>{
    this_module:Option<&'a ModuleEntry>,
    generics:&'a [Generic],
    mapping:&'a HashMap<Id,ModuleEntry>,

    modules_import:Vec<ModuleLink>,
    module_assoc:HashMap<Id,ModRef>,

    call_import:Vec<CallableImport>,
    call_assoc:HashMap<(Ref, Vec<Type>),CallRef>,

    type_import:Vec<RType>,
    type_assoc:HashMap<Type,TypeRef>,

    perm_import:Vec<PermissionImport>,
    perm_assoc:HashMap<(Perm,Type),PermRef>,
    fperm_assoc:HashMap<CallRef,PermRef>

}

impl<'a> CodeImportBuilder<'a> {

    pub fn new(this_module:&'a ModuleEntry,  generics:&'a [Generic], mapping:&'a HashMap<Id,ModuleEntry>) -> Self {
        let mut builder = CodeImportBuilder {
            this_module:Some(this_module),
            generics,
            mapping,
            modules_import: Vec::new(),
            module_assoc: HashMap::new(),
            call_import: Vec::new(),
            call_assoc: HashMap::new(),
            type_import: Vec::new(),
            type_assoc: HashMap::new(),
            perm_import: Vec::new(),
            perm_assoc: HashMap::new(),
            fperm_assoc: HashMap::new(),
        };

        for (i,g) in generics.iter().enumerate() {
            builder.type_assoc.insert(Type{
                main: Ref::Generic(g.name.clone()),
                applies: vec![],
                projections: 0
            }, TypeRef(i as u8));
        }

        builder
    }

    pub fn new_top(mapping:&'a HashMap<Id,ModuleEntry>) -> Self {
        CodeImportBuilder {
            this_module:None,
            generics: &[],
            mapping,
            modules_import: Vec::new(),
            module_assoc: HashMap::new(),
            call_import: Vec::new(),
            call_assoc: HashMap::new(),
            type_import: Vec::new(),
            type_assoc: HashMap::new(),
            perm_import: Vec::new(),
            perm_assoc: HashMap::new(),
            fperm_assoc: HashMap::new(),
        }
    }

    fn mod_import_ref(&mut self, id:&Id) -> Result<ModRef,String>{
        let id = Id(id.0.to_lowercase());
        if self.module_assoc.contains_key(&id) {
            return Ok(self.module_assoc[&id])
        }
        let link = ModuleLink::Remote(self.mapping[&id].hash.unwrap());
        let pos =if self.this_module.is_some(){
            self.modules_import.len()+1
        } else {
            self.modules_import.len()
        };
        self.modules_import.push(link);
        self.module_assoc.insert(id.clone(),ModRef(pos as u8));
        Ok(ModRef(pos as u8))
    }

    pub fn import_fun_ref(&mut self, r:&Ref, appls:Vec<Type>) -> Result<CallRef,String> {
        let key = &(r.clone(),appls.clone());
        if self.call_assoc.contains_key(key ) {
            return Ok(self.call_assoc[key])
        }
        let r_appl = appls.into_iter().map(|t|self.import_typ_ref(&t)).collect::<Result<_,_>>()?;
        let fun_import = match *r {
            Ref::Generic(_) => return Err("Ref is not a function".into()),
            Ref::This(ref id) => {
                let offset = self.this_module.unwrap().index[id].elem_index as u8;
                CallableImport::Function{link:FuncLink{module:ModRef(0), offset}, applies:r_appl}
            },
            Ref::Module(ref m_id, ref e_id) => {
                let module = self.mod_import_ref(&m_id)?;
                if !self.mapping.contains_key(m_id) {panic!("{:?} in {:?}",m_id,self.mapping.keys())}
                if !self.mapping[m_id].index.contains_key(e_id) {panic!("{:?}.{:?} in {:?}",m_id,e_id,self.mapping[m_id].index.keys())}
                let offset = self.mapping[m_id].index[e_id].elem_index as u8;
                CallableImport::Function{link:FuncLink{module, offset}, applies:r_appl}
            },
        };

        let pos = self.call_import.len();
        self.call_import.push(fun_import);
        self.call_assoc.insert(key.clone(), CallRef(pos as u8));
        Ok(CallRef(pos as u8))
    }

    pub fn import_impl_ref(&mut self, r:&Ref, appls:Vec<Type>) -> Result<CallRef,String> {
        let key = &(r.clone(),appls.clone());
        if self.call_assoc.contains_key(key ) {
            return Ok(self.call_assoc[key])
        }
        let r_appl = appls.into_iter().map(|t|self.import_typ_ref(&t)).collect::<Result<_,_>>()?;
        let fun_import = match *r {
            Ref::Generic(_) => return Err("Ref is not a function".into()),
            Ref::This(ref id) => {
                let offset = self.this_module.unwrap().index[id].elem_index as u8;
                CallableImport::Implement{link:ImplLink{module:ModRef(0), offset}, applies:r_appl}
            },
            Ref::Module(ref m_id, ref e_id) => {
                let module = self.mod_import_ref(&m_id)?;
                let offset = self.mapping[m_id].index[e_id].elem_index as u8;
                CallableImport::Implement{link:ImplLink{module, offset}, applies:r_appl}
            },
        };

        let pos = self.call_import.len();
        self.call_import.push(fun_import);
        self.call_assoc.insert(key.clone(), CallRef(pos as u8));
        Ok(CallRef(pos as u8))
    }

    pub fn imported_lit_size(&mut self,  typ:&Type) -> Result<u16,String> {
        match typ.main{
            Ref::Generic(_) =>  return Err("Ref is not a literal type".into()),
            Ref::This(ref id) => {
                let comp = self.this_module.unwrap().get_component(id);
                match comp {
                    Component::ExtLit {size, ..} => Ok(*size),
                    _ => Err("Ref is not a literal type".into())
                }
            },
            Ref::Module(ref m_id, ref e_id) => {
                let comp = self.mapping[&m_id].get_component(e_id);
                match comp {
                    Component::ExtLit {size, ..} => Ok(*size),
                    _ => Err("Ref is not a literal type".into())
                }
            },
        }
    }

    pub fn import_call_ref(&mut self, f_ref:CallRef) -> Result<PermRef,String>{
        if self.fperm_assoc.contains_key(&f_ref ) {
            return Ok(self.fperm_assoc[&f_ref])
        }

        let perm = PermissionImport::Callable(PermSet::from_entry(Permission::Call),f_ref);
        let pos = self.perm_import.len();
        self.perm_import.push(perm);
        self.fperm_assoc.insert(f_ref, PermRef(pos as u8));
        Ok(PermRef(pos as u8))
    }


    pub fn import_perm_ref(&mut self, typ:&(Perm, Type)) -> Result<PermRef,String>{
        if self.perm_assoc.contains_key(typ ) {
            return Ok(self.perm_assoc[typ])
        }
        let t_ref = self.import_typ_ref(&typ.1)?;
        let perm = PermissionImport::Type(PermSet::from_entry(match typ.0 {
            Perm::Call => Permission::Call,
            Perm::DataCreate => Permission::Create,
            Perm::LitCreate => Permission::Create,
            Perm::Inspect => Permission::Inspect,
            Perm::Consume => Permission::Consume,
            Perm::Implement => Permission::Implement,
        }), t_ref);

        let pos = self.perm_import.len();
        self.perm_import.push(perm);
        self.perm_assoc.insert((typ.0,typ.1.clone()), PermRef(pos as u8));
        Ok(PermRef(pos as u8))
    }

    pub fn import_typ_ref(&mut self, typ:&Type) -> Result<TypeRef,String>{
        if self.type_assoc.contains_key(typ ) {
            return Ok(self.type_assoc[typ])
        }
        let r_type = if typ.projections != 0 {
            let plain = self.import_typ_ref(&Type{
                main: typ.main.clone(),
                applies: typ.applies.clone(),
                projections: typ.projections - 1
            })?;
            RType::Projection{typ:plain}
        } else {
            let r_appl = typ.applies.iter().map(|t|self.import_typ_ref(t)).collect::<Result<_,_>>()?;
            match typ.main{
                Ref::Generic(_) => unimplemented!(),
                Ref::This(ref id) => {
                    let offset = self.this_module.unwrap().index[id].elem_index as u8;
                    RType::Data{
                        link:DataLink { module: ModRef(0), offset },
                        applies:r_appl
                    }
                },
                Ref::Module(ref m_id, ref e_id) => {
                    let module = self.mod_import_ref(&m_id)?;
                    let offset = self.mapping[&m_id].index[e_id].elem_index as u8;
                    RType::Data{
                        link:DataLink { module, offset },
                        applies:r_appl
                    }
                },
            }
        };

        let pos = self.type_import.len() + self.generics.len();
        self.type_import.push(r_type);
        self.type_assoc.insert(typ.clone(),TypeRef(pos as u8));
        Ok(TypeRef(pos as u8))
    }

    pub fn get_ctr_order(&mut self, typ:&Type) -> Result<Vec<Id>,String>{
        match typ.main {
            Ref::Generic(_)  => return Ok(vec![]),
            Ref::This(ref id) => {
                match *self.this_module.unwrap().get_component(id){
                    Component::Adt { ref ctrs, .. } => return Ok(ctrs.iter().map(|ctr|ctr.name.clone()).collect()),
                    _ => return Ok(vec![]),
                }
            },
            Ref::Module(ref m_id, ref e_id) => {
                match *self.mapping[m_id].get_component(e_id){
                    Component::Adt { ref ctrs, .. } => return Ok(ctrs.iter().map(|ctr|ctr.name.clone()).collect()),
                    _ => return Ok(vec![]),
                }

            },
        };
    }


    pub fn generate_import(self) -> PublicImport {
        PublicImport {
            modules: self.modules_import,
            types: self.type_import
        }
    }

    pub fn generate_body_import(self) -> (Vec<CallableImport>, Vec<PermissionImport>, PublicImport) {
        (self.call_import, self.perm_import, PublicImport {
                modules: self.modules_import,
                types: self.type_import
        })
    }
}

pub struct BundleImportBuilder{

    literal_import:Vec<Vec<u8>>,
    literal_assoc:HashMap<Vec<u8>,u16>,

    witness_import:Vec<Vec<u8>>,
    witness_assoc:HashMap<Vec<u8>,u16>,

    value_import:Vec<Hash>,
    //this would only be appropriate if we know that it has the same value each time
    //value_assoc:HashMap<Vec<u8>,u16>,

    desc_import:Vec<Hash>,
    desc_assoc:HashMap<Hash,u16>,
}


impl BundleImportBuilder {

    pub fn new() -> Self {
        BundleImportBuilder {
            literal_import: Vec::new(),
            literal_assoc: HashMap::new(),
            witness_import: Vec::new(),
            witness_assoc: HashMap::new(),
            value_import: Vec::new(),
            //value_assoc: HashMap::new(),
            desc_import: Vec::new(),
            desc_assoc: HashMap::new()
        }

    }

    pub fn param_ref(&mut self, param:&ParamData) -> ParamRef{
        match *param {
            ParamData::Load(mode, ref data) => ParamRef::Load(mode, self.value_ref(data)),
            ParamData::Literal(ref data) => ParamRef::Literal(self.literal_ref(data)),
            ParamData::Witness(ref data) => ParamRef::Witness(self.witness_ref(data)),
            ParamData::Provided => ParamRef::Provided
        }
    }

    pub fn literal_ref(&mut self, lit:&Vec<u8>) -> u16{
        if self.literal_assoc.contains_key(lit) {
            return self.literal_assoc[lit]
        }

        let pos = self.literal_import.len() as u16;
        self.literal_import.push(lit.clone());
        self.literal_assoc.insert(lit.clone(),pos);
        pos
    }

    pub fn witness_ref(&mut self, wit:&Vec<u8>) -> u16{
        if self.witness_assoc.contains_key(wit) {
            return self.witness_assoc[wit]
        }

        let pos = self.witness_import.len() as u16;
        self.witness_import.push(wit.clone());
        self.witness_assoc.insert(wit.clone(), pos);
        pos
    }

    pub fn value_ref(&mut self, val:&Hash) -> u16{
        /* we can not ensure that it has the same witness if id is te same
        if self.value_assoc.contains_key(val) {
            return self.value_assoc[val]
        }
        */
        let pos = self.value_import.len() as u16;
        self.value_import.push(val.clone());
        //self.value_assoc.insert(val.clone(), pos);
        pos
    }

    pub fn desc_ref(&mut self, desc:&Hash) -> u16{
        if self.desc_assoc.contains_key(desc) {
            return self.desc_assoc[desc]
        }

        let pos = self.desc_import.len() as u16;
        self.desc_import.push(desc.clone());
        self.desc_assoc.insert(desc.clone(), pos);
        pos
    }

    pub fn literals<'a>(&self, alloc:&'a HeapArena) -> SlicePtr<'a, SlicePtr<'a, u8>>  {
        let mut builder = alloc.slice_builder(self.literal_import.len()).unwrap();
        for v in &self.literal_import {
            builder.push(alloc.copy_alloc_slice(v).unwrap());
        }
        builder.finish()    }

    pub fn witnesses<'a>(&self, alloc:&'a HeapArena) -> SlicePtr<'a, SlicePtr<'a, u8>>  {
        let mut builder = alloc.slice_builder(self.witness_import.len()).unwrap();
        for v in &self.witness_import {
            builder.push(alloc.copy_alloc_slice(v).unwrap());
        }
        builder.finish()
    }

    pub fn values<'a>(&self, alloc:&'a HeapArena) -> SlicePtr<'a, Hash>  {
        let mut builder = alloc.slice_builder(self.value_import.len()).unwrap();
        for v in &self.value_import {
            builder.push(v.clone());
        }
        builder.finish()
    }

    pub fn empty_storage_witnesses<'a>(&self,  alloc:&'a HeapArena) -> SlicePtr<'a, Option<SlicePtr<'a, u8>>>  {
        alloc.repeated_slice(None, self.value_import.len()).unwrap()
    }

    pub fn descs(&self) -> &[Hash] {
        &self.desc_import
    }
}

