use std::collections::HashMap;
use model::*;
use sanskrit_common::model::*;
use script::ModuleEntry;
use sanskrit_core::model::Type as RType;
use sanskrit_core::model::ModRef;
use sanskrit_core::model::ErrorImport;
use sanskrit_core::model::ErrorRef;
use sanskrit_core::model::FunctionImport;
use sanskrit_core::model::FuncRef;
use sanskrit_core::model::TypeRef;
use sanskrit_core::model::ErrorLink;
use sanskrit_core::model::FuncLink;
use sanskrit_core::model::GenRef;
use sanskrit_core::model::BaseType;
use sanskrit_core::model::DataLink;
use sanskrit_core::model::PublicImport;
use sanskrit_runtime::model::TypeApplyRef;
use sanskrit_runtime::model::FuncRef as RFuncRef;
use sanskrit_runtime::model::AdtRef as RAdtRef;
use hex::decode;
use sanskrit_common::arena::HeapArena;
use sanskrit_common::encoding::ParserAllocator;
use sanskrit_runtime::model::ImpRef;


pub struct Region(pub HashMap<Id,usize>);

pub struct Environment<'c,A: ParserAllocator> {
    pub stack_depth:usize,
    pub frames:Vec<Region>,
    pub alloc:&'c A,
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
                   let rel_pos = self.stack_depth - pos -1;
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
    this_module:&'a ModuleEntry,
    generics:&'a [Generic],
    mapping:&'a HashMap<Id,ModuleEntry>,

    modules_import:Vec<ModuleLink>,
    module_assoc:HashMap<Id,ModRef>,

    err_import:Vec<ErrorImport>,
    err_assoc:HashMap<Ref,ErrorRef>,

    fun_import:Vec<FunctionImport>,
    fun_assoc:HashMap<(Ref,Vec<Type>),FuncRef>,

    type_import:Vec<RType>,
    type_assoc:HashMap<Type,TypeRef>
}

impl<'a> CodeImportBuilder<'a> {

    pub fn new(this_module:&'a ModuleEntry,  generics:&'a [Generic], mapping:&'a HashMap<Id,ModuleEntry>) -> Self {
        CodeImportBuilder {
            this_module,
            generics,
            mapping,
            modules_import: Vec::new(),
            module_assoc: HashMap::new(),
            err_import: Vec::new(),
            err_assoc: HashMap::new(),
            fun_import: Vec::new(),
            fun_assoc: HashMap::new(),
            type_import: Vec::new(),
            type_assoc: HashMap::new(),
        }
    }

    fn mod_import_ref(&mut self, id:&Id) -> Result<ModRef,String>{
        let id = Id(id.0.to_lowercase());
        if self.module_assoc.contains_key(&id) {
            return Ok(self.module_assoc[&id])
        }
        let link = ModuleLink::Remote(self.mapping[&id].hash.unwrap());
        let pos = self.modules_import.len()+1;
        self.modules_import.push(link);
        self.module_assoc.insert(id.clone(),ModRef(pos as u8));
        Ok(ModRef(pos as u8))
    }

    pub fn import_err_ref(&mut self, r:&Ref) -> Result<ErrorRef,String>{
        if self.err_assoc.contains_key(r) {
            return Ok(self.err_assoc[r])
        }
        let error_imp = match *r {
            Ref::Generic(_) | Ref::Account(_) | Ref::Txt(_,_) => return Err("Ref is not an error".into()),
            Ref::This(ref id) => {
                let offset = self.this_module.index[id].elem_index as u8;
                ErrorImport{link:ErrorLink{module:ModRef(0), offset,}}
            },
            Ref::Module(ref m_id, ref e_id) => {
                let module = self.mod_import_ref(m_id)?;
                let offset = self.mapping[m_id].index[e_id].elem_index as u8;
                ErrorImport{link:ErrorLink{module, offset,}}
            },
        };

        let pos = self.err_import.len();
        self.err_import.push(error_imp);
        self.err_assoc.insert(r.clone(),ErrorRef(pos as u8));
        Ok(ErrorRef(pos as u8))
    }

    pub fn import_fun_ref(&mut self, r:&Ref, appls:Vec<Type>) -> Result<FuncRef,String> {
        let key = &(r.clone(),appls.clone());
        if self.fun_assoc.contains_key(key ) {
            return Ok(self.fun_assoc[key])
        }
        let r_appl = appls.into_iter().map(|t|self.import_typ_ref(&t)).collect::<Result<_,_>>()?;
        let fun_import = match *r {
            Ref::Generic(_) | Ref::Account(_) | Ref::Txt(_,_) => return Err("Ref is not a funcion".into()),
            Ref::This(ref id) => {
                let offset = self.this_module.index[id].elem_index as u8;
                FunctionImport{link:FuncLink{module:ModRef(0), offset}, applies:r_appl}
            },
            Ref::Module(ref m_id, ref e_id) => {
                let module = self.mod_import_ref(&m_id)?;
                let offset = self.mapping[m_id].index[e_id].elem_index as u8;
                FunctionImport{link:FuncLink{module, offset}, applies:r_appl}
            },
        };

        let pos = self.fun_import.len();
        self.fun_import.push(fun_import);
        self.fun_assoc.insert(key.clone(),FuncRef(pos as u8));
        Ok(FuncRef(pos as u8))
    }

    pub fn imported_lit_size(&mut self,  typ:&Type) -> Result<u16,String> {
        match typ.main{
            Ref::Account(_) | Ref::Txt(_,_) => return Err("Ref is not a sanskrit type (is a script type)".into()),
            Ref::Generic(ref id) =>  return Err("Ref is not a literal type".into()),
            Ref::This(ref id) => {
                let comp = self.this_module.get_component(id);
                match comp {
                   Component::Lit {size, ..}
                   | Component::ExtLit {size, ..} => Ok(*size),
                    _ => Err("Ref is not a literal type".into())
                }
            },
            Ref::Module(ref m_id, ref e_id) => {
                let comp = self.mapping[&m_id].get_component(e_id);
                match comp {
                    Component::Lit {size, ..}
                    | Component::ExtLit {size, ..} => Ok(*size),
                    _ => Err("Ref is not a literal type".into())
                }
            },
        }
    }


    pub fn import_typ_ref(&mut self, typ:&Type) -> Result<TypeRef,String>{
        if self.type_assoc.contains_key(typ ) {
            return Ok(self.type_assoc[typ])
        }
        let rtyp = if typ.is_image {
            let plain = self.import_typ_ref(&Type{
                main: typ.main.clone(),
                applies: typ.applies.clone(),
                is_image: false
            })?;
            RType::Image(plain)
        } else {
            let r_appl = typ.applies.iter().map(|t|self.import_typ_ref(t)).collect::<Result<_,_>>()?;
            match typ.main{
                Ref::Account(_) | Ref::Txt(_,_) => return Err("Ref is not a sanskrit type (is a script type)".into()),
                Ref::Generic(ref id) => {
                    let pos = match self.generics.iter().position(|g|&g.name==id){
                        Some(v) => v,
                        None => 0,
                    };
                    RType::Generic(GenRef(pos as u8))
                },
                Ref::This(ref id) => {
                    let offset = self.this_module.index[id].elem_index as u8;
                    RType::Real(BaseType::Data(DataLink { module: ModRef(0), offset }), r_appl)
                },
                Ref::Module(ref m_id, ref e_id) => {
                    let module = self.mod_import_ref(&m_id)?;
                    let offset = self.mapping[&m_id].index[e_id].elem_index as u8;
                    RType::Real(BaseType::Data(DataLink { module, offset }), r_appl)
                },
            }
        };

        let pos = self.type_import.len();
        self.type_import.push(rtyp);
        self.type_assoc.insert(typ.clone(),TypeRef(pos as u8));
        Ok(TypeRef(pos as u8))
    }

    pub fn get_ctr_order(&mut self, typ:&Type) -> Result<Vec<Id>,String>{
        match typ.main {
            Ref::Generic(_) | Ref::Account(_) | Ref::Txt(_,_) => return Err("Type has no Ctrs".into()),
            Ref::This(ref id) => {
                match *self.this_module.get_component(id){
                    Component::Adt { ref ctrs, .. } => return Ok(ctrs.iter().map(|ctr|ctr.name.clone()).collect()),
                    _ =>  return Err("Not an internal Adt".into()),
                }
            },
            Ref::Module(ref m_id, ref e_id) => {
                match *self.mapping[m_id].get_component(e_id){
                    Component::Adt { ref ctrs, .. } => return Ok(ctrs.iter().map(|ctr|ctr.name.clone()).collect()),
                    _ => return Err("Not an internal Adt".into()),
                }

            },
        };
    }


    pub fn generate_import(self) -> PublicImport {
        PublicImport {
            modules: self.modules_import,
            errors: self.err_import,
            types: self.type_import
        }
    }

    pub fn generate_body_import(self) -> (Vec<FunctionImport>, PublicImport) {
        (self.fun_import, PublicImport {
                modules: self.modules_import,
                errors: self.err_import,
                types: self.type_import
        })
    }
}


pub struct ScriptContext<'a,'c, 'h>{
    mapping:&'a HashMap<Id,ModuleEntry>,
    pub imports: HashMap<Hash,ImpRef>,
    pub wit: Vec<SlicePtr<'c,u8>>,
    sigs:&'a HashMap<Id,u8>,
    news:&'a HashMap<Id,u8>,
    pub alloc:&'c HeapArena<'h>,
}


impl<'a,'c, 'h> ScriptContext<'a,'c, 'h> {

    pub fn new(mapping:&'a HashMap<Id,ModuleEntry>, sigs:&'a HashMap<Id,u8>, news:&'a HashMap<Id,u8>, alloc:&'c HeapArena<'h> ) -> Self {
        ScriptContext {
            mapping,
            imports: HashMap::new(),
            wit: Vec::new(),
            sigs,
            news,
            alloc
        }
    }

    pub fn get_token_offset(&self, new_type:&Id) -> u8 {
        let res =if self.news.contains_key(new_type){
            self.news.len()  - self.news[new_type] as usize - 1
        } else {
            self.sigs.len()  - self.sigs[new_type] as usize - 1 + self.news.len()
        };
        res as u8
    }

    pub fn generate_imp_ref(&mut self, imp:&Hash) -> ImpRef {
        if self.imports.contains_key(imp) {
            return self.imports[imp]
        }

        let imp_ref = ImpRef(self.imports.len() as u8);
        self.imports.insert(*imp,imp_ref);
        imp_ref
    }

    pub fn generate_wit_ref(&mut self, num:SlicePtr<'c, u8>) -> u8 {
        let wit_ref = self.wit.len() as u8;
        self.wit.push(num);
        wit_ref
    }

    pub fn generate_func_ref(&mut self, r:&Ref) -> Result<RFuncRef,String> {
        match *r {
            Ref::This(_) | Ref::Generic(_) | Ref::Account(_) | Ref::Txt(_,_) => Err("Ref is not an funcion".into()),
            Ref::Module(ref m_id, ref e_id) => {
                let module = &self.mapping[m_id];
                let offset = module.index[e_id].elem_index as u8;
                Ok(RFuncRef{
                    module: self.generate_imp_ref(&module.hash.unwrap()),
                    offset
                })
            },
        }
    }

    pub fn generate_adt_ref(&mut self, r:&Ref) -> Result<RAdtRef,String> {
        match *r {
            Ref::This(_) | Ref::Generic(_) | Ref::Account(_) | Ref::Txt(_,_) => return Err("Ref is not an adt".into()),
            Ref::Module(ref m_id, ref e_id) => {
                let module = &self.mapping[m_id];
                let offset = module.index[e_id].elem_index as u8;
                Ok(RAdtRef{module:self.generate_imp_ref(&module.hash.unwrap()), offset})
            },
        }
    }

    pub fn generate_type_ref(&mut self, t_ref:&Type) -> Result<Ptr<'c, TypeApplyRef<'c>>,String> {
        Ok(if t_ref.is_image {
            let plain = self.generate_type_ref(&Type {
                main: t_ref.main.clone(),
                applies: t_ref.applies.clone(),
                is_image: false
            })?;
            self.alloc.alloc(TypeApplyRef::Image(plain))
        } else {
            self.alloc.alloc(match t_ref.main {
                Ref::Generic(_) => return Err("Ref is not an adt".into()),
                Ref::Module(ref m_id, ref e_id) => {
                    let module = &self.mapping[m_id];
                    let offset = self.mapping[m_id].index[e_id].elem_index as u8;
                    let applies = self.alloc.iter_result_alloc_slice(t_ref.applies.iter().map(|tar|self.generate_type_ref(tar)))?;
                    TypeApplyRef::Module(self.generate_imp_ref(&module.hash.unwrap()), offset, applies)
                },

                Ref::This(ref id) => {
                    match self.news.get(id) {
                        Some(idx) => TypeApplyRef::NewType(*idx),
                        None => match self.sigs.get(id) {
                            Some(idx) => TypeApplyRef::Account(*idx),
                            None => return Err("Unsupported local type".into())
                        }
                    }
                }

                Ref::Account(ref addr) => {
                    let decoded = decode(&addr.0[2..42]).unwrap();
                    let hash_data_ref = array_ref!(decoded,0,20);
                    TypeApplyRef::RemoteAccount(self.generate_imp_ref(hash_data_ref))
                }

                Ref::Txt(ref txt, ref num) => {
                    let decoded = decode(&txt.0[2..42]).unwrap();
                    let hash_data_ref = array_ref!(decoded,0,20);
                    TypeApplyRef::RemoteNewType(self.generate_imp_ref(hash_data_ref),num.0.parse::<u8>().unwrap())
                }
            })
        })
    }

    pub fn get_tag(&mut self, r:&Ref, ctr:Id) -> Result<Tag,String>{
        match r {
            Ref::This(_) | Ref::Generic(_) | Ref::Account(_) | Ref::Txt(_,_)  => Err("Generics and Phantoms do not have Ctrs".into()),
            Ref::Module(ref m_id, ref e_id) => {
                match *self.mapping[m_id].get_component(e_id){
                    Component::Adt { ref ctrs, .. } => Ok(Tag(ctrs.iter().enumerate().find(|(_,case)|case.name == ctr).unwrap().0 as u8)),
                    _ => Err("Not an internal Adt".into()),
                }
            },
        }
    }
}