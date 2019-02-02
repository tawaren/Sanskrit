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
use sanskrit_core::model::AdtLink;
use sanskrit_core::model::PublicImport;
use sanskrit_core::model::BodyImport;
use sanskrit_runtime::model::TypeApplyRef;
use sanskrit_runtime::model::FuncRef as RFuncRef;
use sanskrit_runtime::model::AdtRef as RAdtRef;
use sanskrit_runtime::model::NativeAdtType;
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
            Ref::Native(ref id) => {
                match id.0.as_ref() {
                    "NumericError" => ErrorImport::Native(NativeError::NumericError),
                    "IndexError" => ErrorImport::Native(NativeError::IndexError),
                    "Unexpected" => ErrorImport::Native(NativeError::Unexpected),
                    _ => return Err("Unsupported native error".into())

                }
            },
            Ref::This(ref id) => {
                let offset = self.this_module.index[id].elem_index as u8;
                ErrorImport::Module(ErrorLink{module:ModRef(0), offset,})
            },
            Ref::Module(ref m_id, ref e_id) => {
                let module = self.mod_import_ref(m_id)?;
                let offset = self.mapping[m_id].index[e_id].elem_index as u8;
                ErrorImport::Module(ErrorLink{module, offset,})
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
            Ref::Generic(_) | Ref::Account(_) | Ref::Txt(_,_) => return Err("Ref is not an funcion".into()),
            Ref::Native(ref id) => {
                match id.0.as_ref() {
                    "And" => FunctionImport::Native(NativeFunc::And,r_appl),
                    "Or" => FunctionImport::Native(NativeFunc::Or,r_appl),
                    "Xor" => FunctionImport::Native(NativeFunc::Xor,r_appl),
                    "Not" => FunctionImport::Native(NativeFunc::Not,r_appl),
                    "Extend" => FunctionImport::Native(NativeFunc::Extend,r_appl),
                    "Cut" => FunctionImport::Native(NativeFunc::Cut,r_appl),
                    "SignCast" => FunctionImport::Native(NativeFunc::SignCast,r_appl),
                    "Add" => FunctionImport::Native(NativeFunc::Add,r_appl),
                    "Sub" => FunctionImport::Native(NativeFunc::Sub,r_appl),
                    "Mul" => FunctionImport::Native(NativeFunc::Mul,r_appl),
                    "Div" => FunctionImport::Native(NativeFunc::Div,r_appl),
                    "Eq" => FunctionImport::Native(NativeFunc::Eq,r_appl),
                    "Hash" => FunctionImport::Native(NativeFunc::Hash, r_appl),
                    "PlainHash" => FunctionImport::Native(NativeFunc::PlainHash, r_appl),
                    "Lt" => FunctionImport::Native(NativeFunc::Lt,r_appl),
                    "Gt" => FunctionImport::Native(NativeFunc::Gt,r_appl),
                    "Lte" => FunctionImport::Native(NativeFunc::Lte,r_appl),
                    "Gte" => FunctionImport::Native(NativeFunc::Gte,r_appl),
                    "ToData" => FunctionImport::Native(NativeFunc::ToData,r_appl),
                    "Concat" => FunctionImport::Native(NativeFunc::Concat,r_appl),
                    "SetBit" => FunctionImport::Native(NativeFunc::SetBit,r_appl),
                    "GetBit" => FunctionImport::Native(NativeFunc::GetBit,r_appl),
                    "GenId" => FunctionImport::Native(NativeFunc::GenId, r_appl),
                    "ToRef" => FunctionImport::Native(NativeFunc::ToRef,r_appl),
                    "Derive" => FunctionImport::Native(NativeFunc::Derive,r_appl),
                    _ => return Err("Unsupported native function".into())

                }
            },
            Ref::This(ref id) => {
                let offset = self.this_module.index[id].elem_index as u8;
                FunctionImport::Module(FuncLink{module:ModRef(0), offset}, r_appl)
            },
            Ref::Module(ref m_id, ref e_id) => {
                let module = self.mod_import_ref(&m_id)?;
                let offset = self.mapping[m_id].index[e_id].elem_index as u8;
                FunctionImport::Module(FuncLink{module, offset}, r_appl)
            },
        };

        let pos = self.fun_import.len();
        self.fun_import.push(fun_import);
        self.fun_assoc.insert(key.clone(),FuncRef(pos as u8));
        Ok(FuncRef(pos as u8))
    }

    pub fn import_typ_ref(&mut self, typ:&Type) -> Result<TypeRef,String>{
        if self.type_assoc.contains_key(typ ) {
            return Ok(self.type_assoc[typ])
        }
        let r_appl = typ.applies.iter().map(|t|self.import_typ_ref(t)).collect::<Result<_,_>>()?;
        let rtyp = match typ.main{
            Ref::Account(_) | Ref::Txt(_,_) => return Err("Ref is not a sanskrit type (is a script type)".into()),
            Ref::Generic(ref id) => {
                let pos = match self.generics.iter().position(|g|&g.name==id){
                    Some(v) => v,
                    None => 0,
                };
                RType::Generic(GenRef(pos as u8))
            },
            Ref::Native(ref id) => {
                let bt = match id.0.as_ref() {
                    "bool" => NativeType::Bool,
                    "ref" => NativeType::Ref,
                    "id" => NativeType::Id,
                    "u8" => NativeType::UInt(1),
                    "u16" => NativeType::UInt(2),
                    "u32" => NativeType::UInt(4),
                    "u64" => NativeType::UInt(8),
                    "u128" => NativeType::UInt(16),
                    "i8" => NativeType::SInt(1),
                    "i16" => NativeType::SInt(2),
                    "i32" => NativeType::SInt(4),
                    "i64" => NativeType::SInt(8),
                    "i128" => NativeType::SInt(16),
                    "tuple0"  => NativeType::Tuple(0),
                    "tuple1"  => NativeType::Tuple(1),
                    "tuple2"  => NativeType::Tuple(2),
                    "tuple3"  => NativeType::Tuple(3),
                    "tuple4"  => NativeType::Tuple(4),
                    "tuple5"  => NativeType::Tuple(5),
                    "tuple6"  => NativeType::Tuple(6),
                    "tuple7"  => NativeType::Tuple(7),
                    "tuple8"  => NativeType::Tuple(8), //for tests enough
                    "alt0"  => NativeType::Alternative(0),
                    "alt1"  => NativeType::Alternative(1),
                    "alt2"  => NativeType::Alternative(2),
                    "alt3"  => NativeType::Alternative(3),
                    "alt4"  => NativeType::Alternative(4),
                    "alt5"  => NativeType::Alternative(5),
                    "alt6"  => NativeType::Alternative(6),
                    "alt7"  => NativeType::Alternative(7),
                    "alt8"  => NativeType::Alternative(8), //for tests enough
                    "data1"  => NativeType::Data(1),
                    "data2"  => NativeType::Data(2),
                    "data4"  => NativeType::Data(4),
                    "data8"  => NativeType::Data(8),
                    "data12"  => NativeType::Data(12),
                    "data16"  => NativeType::Data(16),
                    "data20"  => NativeType::Data(20),
                    "data24"  => NativeType::Data(24),
                    "data28"  => NativeType::Data(28),
                    "data32"  => NativeType::Data(32),
                    "data40"  => NativeType::Data(40),
                    "data48"  => NativeType::Data(48),
                    "data56"  => NativeType::Data(56),
                    "data64"  => NativeType::Data(64),
                    "data80"  => NativeType::Data(80),
                    "data96"  => NativeType::Data(96),
                    "data112"  => NativeType::Data(112),
                    "data128"  => NativeType::Data(128),
                    "data160"  => NativeType::Data(160),
                    "data192"  => NativeType::Data(192),
                    "data224"  => NativeType::Data(224),
                    _ => return Err("Unsupported native type".into())

                };
                RType::Real(BaseType::Native(bt),r_appl)
            },
            Ref::This(ref id) => {
                let offset = self.this_module.index[id].elem_index as u8;
                RType::Real(BaseType::Module(AdtLink{ module: ModRef(0), offset }), r_appl)
            },
            Ref::Module(ref m_id, ref e_id) => {
                let module = self.mod_import_ref(&m_id)?;
                let offset = self.mapping[&m_id].index[e_id].elem_index as u8;
                RType::Real(BaseType::Module(AdtLink{ module, offset }), r_appl)
            },
        };

        let pos = self.type_import.len();
        self.type_import.push(rtyp);
        self.type_assoc.insert(typ.clone(),TypeRef(pos as u8));
        Ok(TypeRef(pos as u8))    }

    pub fn get_ctr_order(&mut self, typ:&Type) -> Result<Vec<Id>,String>{
        match typ.main {
            Ref::Generic(_) | Ref::Account(_) | Ref::Txt(_,_) => return Err("Type has no Ctrs".into()),
            Ref::Native(ref id) => return Ok(match id.0.as_ref() {
                "u8" => vec![], //to test failures (not really a constructable type)
                "bool" => vec![Id("false".to_owned()),Id("true".to_owned())],
                "tuple0" => vec![Id("tuple".to_owned())],
                "tuple1" => vec![Id("tuple".to_owned())],
                "tuple2" => vec![Id("tuple".to_owned())],
                "tuple3" => vec![Id("tuple".to_owned())],
                "tuple4" => vec![Id("tuple".to_owned())],
                "tuple5" => vec![Id("tuple".to_owned())],
                "tuple6" => vec![Id("tuple".to_owned())],
                "tuple7" => vec![Id("tuple".to_owned())],
                "tuple8" => vec![Id("tuple".to_owned())],
                "alt1" => vec![Id("first".to_owned())],
                "alt2" => vec![Id("first".to_owned()), Id("second".to_owned())],
                "alt3" => vec![Id("first".to_owned()), Id("second".to_owned()), Id("third".to_owned())],
                "alt4" => vec![Id("first".to_owned()), Id("second".to_owned()), Id("third".to_owned()), Id("fourth".to_owned())],
                "alt5" => vec![Id("first".to_owned()), Id("second".to_owned()), Id("third".to_owned()), Id("fourth".to_owned()), Id("fifth".to_owned())],
                "alt6" => vec![Id("first".to_owned()), Id("second".to_owned()), Id("third".to_owned()), Id("fourth".to_owned()), Id("fifth".to_owned()), Id("sixth".to_owned())],
                "alt7" => vec![Id("first".to_owned()), Id("second".to_owned()), Id("third".to_owned()), Id("fourth".to_owned()), Id("fifth".to_owned()), Id("sixth".to_owned()), Id("seventh".to_owned())],
                "alt8" => vec![Id("first".to_owned()), Id("second".to_owned()), Id("third".to_owned()), Id("fourth".to_owned()), Id("fifth".to_owned()), Id("sixth".to_owned()), Id("seventh".to_owned()), Id("eight".to_owned())],
                _ => return Err("Unsupported native type".into())
            }),
            Ref::This(ref id) => {
                match *self.this_module.get_component(id){
                    Component::Adt { ref ctrs, .. } => return Ok(ctrs.iter().map(|ctr|ctr.name.clone()).collect()),
                    Component::Err { .. } => return Err("Not a Adt".into()),
                    Component::Fun { .. } =>  return Err("Not a Adt".into()),
                }
            },
            Ref::Module(ref m_id, ref e_id) => {
                match *self.mapping[m_id].get_component(e_id){
                    Component::Adt { ref ctrs, .. } => return Ok(ctrs.iter().map(|ctr|ctr.name.clone()).collect()),
                    Component::Err { .. } => return Err("Not a Adt".into()),
                    Component::Fun { .. } =>  return Err("Not a Adt".into()),
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

    pub fn generate_body_import(self) -> BodyImport {
        BodyImport {
            base: PublicImport {
                modules: self.modules_import,
                errors: self.err_import,
                types: self.type_import
            },
            functions: self.fun_import
        }

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
            Ref::This(_) | Ref::Generic(_) | Ref::Native(_) | Ref::Account(_) | Ref::Txt(_,_) => Err("Ref is not an funcion".into()),
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
                Ok(RAdtRef::Ref(self.generate_imp_ref(&module.hash.unwrap()), offset))
            },
            Ref::Native(ref id) => {
                let bt = match id.0.as_ref() {
                    "bool" => NativeAdtType::Bool,
                    "tuple0" => NativeAdtType::Tuple(0),
                    "tuple1" => NativeAdtType::Tuple(1),
                    "tuple2" => NativeAdtType::Tuple(2),
                    "tuple3" => NativeAdtType::Tuple(3),
                    "tuple4" => NativeAdtType::Tuple(4),
                    "tuple5" => NativeAdtType::Tuple(5),
                    "tuple6" => NativeAdtType::Tuple(6),
                    "tuple7" => NativeAdtType::Tuple(7),
                    "tuple8" => NativeAdtType::Tuple(8), //for tests enough
                    "alt0" => NativeAdtType::Alternative(0),
                    "alt1" => NativeAdtType::Alternative(1),
                    "alt2" => NativeAdtType::Alternative(2),
                    "alt3" => NativeAdtType::Alternative(3),
                    "alt4" => NativeAdtType::Alternative(4),
                    "alt5" => NativeAdtType::Alternative(5),
                    "alt6" => NativeAdtType::Alternative(6),
                    "alt7" => NativeAdtType::Alternative(7),
                    "alt8" => NativeAdtType::Alternative(8), //for tests enough
                    _ => return Err("Unsupported native type".into())
                };
                Ok(RAdtRef::Native(bt))
            },
        }
    }

    pub fn generate_type_ref(&mut self, t_ref:&Type) -> Result<Ptr<'c, TypeApplyRef<'c>>,String> {
        Ok(self.alloc.alloc(match t_ref.main {
            Ref::Generic(_) => return Err("Ref is not an adt".into()),
            Ref::Module(ref m_id, ref e_id) => {
                let module = &self.mapping[m_id];
                let offset = self.mapping[m_id].index[e_id].elem_index as u8;
                let applies = self.alloc.iter_result_alloc_slice(t_ref.applies.iter().map(|tar|self.generate_type_ref(tar)))?;
                TypeApplyRef::Module(self.generate_imp_ref(&module.hash.unwrap()), offset, applies)
            },
            Ref::Native(ref id) => {
                let typ = match id.0.as_ref() {
                    "bool" => NativeType::Bool,
                    "ref" => NativeType::Ref,
                    "id" => NativeType::Id,
                    "u8" => NativeType::UInt(1),
                    "u16" => NativeType::UInt(2),
                    "u32" => NativeType::UInt(4),
                    "u64" => NativeType::UInt(8),
                    "u128" => NativeType::UInt(16),
                    "i8" => NativeType::SInt(1),
                    "i16" => NativeType::SInt(2),
                    "i32" => NativeType::SInt(4),
                    "i64" => NativeType::SInt(8),
                    "i128" => NativeType::SInt(16),
                    "tuple0"  => NativeType::Tuple(0),
                    "tuple1"  => NativeType::Tuple(1),
                    "tuple2"  => NativeType::Tuple(2),
                    "tuple3"  => NativeType::Tuple(3),
                    "tuple4"  => NativeType::Tuple(4),
                    "tuple5"  => NativeType::Tuple(5),
                    "tuple6"  => NativeType::Tuple(6),
                    "tuple7"  => NativeType::Tuple(7),
                    "tuple8"  => NativeType::Tuple(8), //for tests enough
                    "alt0"  => NativeType::Alternative(0),
                    "alt1"  => NativeType::Alternative(1),
                    "alt2"  => NativeType::Alternative(2),
                    "alt3"  => NativeType::Alternative(3),
                    "alt4"  => NativeType::Alternative(4),
                    "alt5"  => NativeType::Alternative(5),
                    "alt6"  => NativeType::Alternative(6),
                    "alt7"  => NativeType::Alternative(7),
                    "alt8"  => NativeType::Alternative(8), //for tests enough
                    "data1"  => NativeType::Data(1),
                    "data2"  => NativeType::Data(2),
                    "data4"  => NativeType::Data(4),
                    "data8"  => NativeType::Data(8),
                    "data12"  => NativeType::Data(12),
                    "data16"  => NativeType::Data(16),
                    "data20"  => NativeType::Data(20),
                    "data24"  => NativeType::Data(24),
                    "data28"  => NativeType::Data(28),
                    "data32"  => NativeType::Data(32),
                    "data40"  => NativeType::Data(40),
                    "data48"  => NativeType::Data(48),
                    "data56"  => NativeType::Data(56),
                    "data64"  => NativeType::Data(64),
                    "data80"  => NativeType::Data(80),
                    "data96"  => NativeType::Data(96),
                    "data112"  => NativeType::Data(112),
                    "data128"  => NativeType::Data(128),
                    "data160"  => NativeType::Data(160),
                    "data192"  => NativeType::Data(192),
                    "data224"  => NativeType::Data(224),
                    _ => return Err("Unsupported native type".into())
                };
                let applies = self.alloc.iter_result_alloc_slice(t_ref.applies.iter().map(|tar|self.generate_type_ref(tar)))?;
                TypeApplyRef::Native(typ,applies)
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
        }))
    }

    pub fn get_tag(&mut self, r:&Ref, ctr:Id) -> Result<Tag,String>{
        match r {
            Ref::This(_) | Ref::Generic(_) | Ref::Account(_) | Ref::Txt(_,_)  => Err("Generics and Phantoms do not have Ctrs".into()),
            Ref::Native(ref id) => match id.0.as_ref() {
                "tuple0" | "tuple1" | "tuple2" | "tuple3"
                | "tuple4" | "tuple5" | "tuple6" | "tuple7"
                | "tuple8" => if ctr.0 == "tuple" {
                    Ok(Tag(0))
                } else {
                    return Err("Unsupported native type".into())
                }
                "alt1" | "alt2" | "alt3" | "alt4"
                | "alt5" | "alt6" | "alt7" | "alt8" => {
                    match  ctr.0.as_ref() {
                        "first" => Ok(Tag(0)),
                        "second" => Ok(Tag(1)),
                        "third" => Ok(Tag(2)),
                        "fourth" => Ok(Tag(3)),
                        "fifth" => Ok(Tag(4)),
                        "sixth" => Ok(Tag(5)),
                        "seventh" => Ok(Tag(6)),
                        "eight" => Ok(Tag(7)),
                        _ => Err("Unsupported native type".into())
                    }
                },
                _ => Err("Unsupported native type".into())
            },
            Ref::Module(ref m_id, ref e_id) => {
                match *self.mapping[m_id].get_component(e_id){
                    Component::Adt { ref ctrs, .. } => Ok(Tag(ctrs.iter().enumerate().find(|(_,case)|case.name == ctr).unwrap().0 as u8)),
                    Component::Err { .. } => Err("Not an Adt".into()),
                    Component::Fun { .. } => Err("Not an Adt".into()),
                }
            },
        }
    }
}