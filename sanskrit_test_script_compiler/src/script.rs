use std::fs::File;
use std::io::Read;
use std::path::Path;
use model::*;
use std::path::PathBuf;
use std::collections::HashMap;
use sanskrit_core::model::{Module as RModule, FunSigShared, CallableImpl, DataImpl, ImplementComponent, SigComponent, Generic as RGeneric, BodyImport, PublicImport, BitSerializedVec};
use sanskrit_common::model::*;
use environment::{CodeImportBuilder, Perm, BundleImportBuilder};
use sanskrit_core::model::DataComponent;
use environment::Environment;
use sanskrit_core::model::FunctionComponent;
use sanskrit_common::store::store_hash;
use sanskrit_common::encoding::*;
use itertools::Itertools;
use parser;
use std::collections::HashSet;
use sanskrit_runtime::model::{ParamRef, RetType, Transaction, TransactionBundle, ParamMode, BundleSection, SectionType, TransactionBundleCore};
use sanskrit_common::arena::{Heap, HeapArena};
use hex::decode;
use sanskrit_runtime::CONFIG;
use sanskrit_common::hashing::HashingDomain;

//Process text -> Adt -> Adt + Index (allows to get stuff by Id) -> EAst (has stuff like captures) -> Final
//All involved file go seq:
// First collect dependencies & Bring everithing into Adt Form
// Second Calc Index for everithing
// Third Enrich Ast with missing types & captures
// Produce finals (Starting at Leaves)
// Deploy everithing (Starting at Leaves)

pub struct Index{
    pub comp_index:usize,
    pub elem_index:usize
}

pub struct ModuleEntry {
    pub level:usize,
    pub module:Module,
    pub index:HashMap<Id, Index>,
    pub hash:Option<Hash>
}

impl ModuleEntry {
    pub fn get_component(&self, id:&Id) -> &Component {
        let index = &self.index[id];
        &self.module.components[index.comp_index]
    }
}

pub struct Compiler<'a> {
    folder:&'a Path,
    pub parsed:HashMap<Id,ModuleEntry>,
    txt_funs:Vec<TransactionCompilationResult>,
    compiled:HashMap<Id,Vec<u8>>
}


impl<'a> Compiler<'a> {
    pub fn new(folder:&'a Path) -> Self {
        Compiler{
            folder,
            parsed: HashMap::new(),
            txt_funs: Vec::new(),
            compiled:  HashMap::new(),
        }
    }

    fn get_module_path(&self, module:&Id) -> PathBuf {
        self.folder.join(&module.0.to_lowercase()).with_extension("sans")
    }

    fn get_txt_path(&self, txt:&Id) -> PathBuf {
        self.folder.join(&txt.0.to_lowercase()).with_extension("txt")
    }

    pub fn get_direct_dependencies(&self, module_id:Id) -> HashSet<Id> {
        let module_id = Id(module_id.0.to_lowercase());
        let mut f = File::open(self.get_module_path(&module_id)).unwrap();
        let mut contents = String::new();
        f.read_to_string(&mut contents).unwrap();

        let module = match parser::ModuleParser::new().parse(&contents) {
            Ok(m) => m,
            Err(err) => {
                println!("{:?}", err);
                panic!()
            },
        };

        if module.name.0.to_lowercase() != module_id.0.to_lowercase() {
            panic!()
        }
        module.collect_module_dependencies()
    }

    pub fn parse_and_compile_transactions(&mut self, txt_id:Id) -> Result<(),String> {
        let txt_id = Id(txt_id.0.to_lowercase());
        let mut f = File::open(self.get_txt_path(&txt_id)).unwrap();
        let mut contents = String::new();
        f.read_to_string(&mut contents).unwrap();
        let txts:Vec<Component> = match parser::TransactionsParser::new().parse(&contents){
            Ok(m) => m,
            Err(err) => {
                panic!("{:?} in {:?}",err, txt_id)
            },
        };
        let mut deps = HashSet::new();
        for c in &txts {
            c.collect_module_dependencies(&mut deps)
        }

        for dep in deps {
            self.parse_module_tree(dep,0);
        }

        self.compile_module_tree()?;

        for c in &txts {
            if let Component::Txt{ transactional, ref params, ref returns ,ref code, .. } = c {
                let imp = CodeImportBuilder::new_top(&self.parsed);
                //extract function prams
                let var_params:Vec<Var> = params.iter().map(|input| Var{
                    name: input.name(),
                    consume: input.consume(),
                    typ: input.typ().clone()
                }).collect();

                let ret_returns:Vec<Ret> = returns.iter().map(|output| Ret{
                    name: output.name(),
                    typ: output.typ().clone()
                }).collect();

                let comp = Self::compile_fun_comp(imp, &Visibility::Public, *transactional, &[],&var_params,&ret_returns,code)?;
                let mut s = Serializer::new(usize::max_value());
                comp.serialize(&mut s)?;
                let txt_fun = s.extract();

                fn derive(comp:&Compiler, module:&Id, id:&str ) -> Vec<u8> {
                    let mod_hash = store_hash(&[&comp.compiled.get(module).unwrap()]);
                    let key_data = decode(&id[2..]).unwrap();
                    let mut context = HashingDomain::Derive.get_domain_hasher();
                    context.update(&mod_hash);
                    context.update(&key_data);
                    context.finalize().to_vec()
                }

                let param_types = params.iter().map(|input| match input {
                    In::Copy(Key::Plain(id), _, _) => ParamData::Load(ParamMode::Copy, hash_from_slice(&decode(&id.0).unwrap())),
                    In::Copy(Key::Derived(module, id), _, _) => ParamData::Load(ParamMode::Copy, hash_from_slice(&derive(self,&module, &id.0))),
                    In::Borrow(Key::Plain(id), _, _) => ParamData::Load(ParamMode::Borrow, hash_from_slice(&decode(&id.0).unwrap())),
                    In::Borrow(Key::Derived(module, id), _, _) => ParamData::Load(ParamMode::Borrow, hash_from_slice(&derive(self,&module, &id.0))),
                    In::Consume(Key::Plain(id), _, _) => ParamData::Load(ParamMode::Consume, hash_from_slice(&decode(&id.0).unwrap())),
                    In::Consume(Key::Derived(module, id), _, _) => ParamData::Load(ParamMode::Consume,  hash_from_slice(&derive(self,&module, &id.0))),
                    In::Literal(ref data, _, _) => ParamData::Literal(self.serialized_object_from_data(data)),
                    In::Witness(ref data, _, _) => ParamData::Witness(self.serialized_object_from_data(data)),
                    In::Provided(_, _) => ParamData::Provided,

                }).collect();

                let ret_types:Vec<RetType> = returns.iter().map(|output| match *output {
                    Out::Store(_, _) => RetType::Store,
                    Out::Drop(_, _) => RetType::Drop,
                    Out::Log(_, _) => RetType::Log,
                }).collect();


                let res = TransactionCompilationResult {
                    txt_fun,
                    param_types,
                    ret_types,
                };
                self.txt_funs.push(res);
            }
        }

        //shall we store and provide a get result
        Ok(())
    }

    fn serialized_object_from_data(&self, data:&Data) -> Vec<u8> {
        let mut ser = Serializer::new(CONFIG.max_structural_dept);
        self.entry_from_data(data, &mut ser);
        ser.extract()
    }

    fn entry_from_data<'b,'h>(&self, data:&Data, s:&mut Serializer) {
        fn find_ctr_index(ctrs:&Vec<Case>, id:&Id) -> u8 {
            for (i,c) in ctrs.iter().enumerate() {
                if &c.name == id {
                    return i as u8;
                }
            };
            unimplemented!()
        }

        match data {
            Data::Adt { ref typ, ref ctr, ref params } => {
                if let Ref::Module(ref module, ref elem) = typ {
                    let mod_entry = self.parsed.get(module).unwrap();
                    if let  Component::Adt { ref ctrs, .. } = mod_entry.get_component(elem) {
                        if ctrs.len() != 1 {
                            find_ctr_index(ctrs, ctr).serialize(s).unwrap()
                        }
                        for d in params {
                            self.entry_from_data(d, s)
                        }
                    }
                }
                unimplemented!()
            },
            Data::Lit { ref typ, ref lit } => {
                if let Ref::Module(ref module, ref elem) = typ {
                    let mod_entry = self.parsed.get(module).unwrap();
                    match mod_entry.get_component(elem) {
                        Component::ExtLit { size, .. } => {
                            return if elem.0.starts_with("u") {
                                match size {
                                    1 => lit.0.parse::<u8>().unwrap().serialize(s).unwrap(),
                                    2 => lit.0.parse::<u16>().unwrap().serialize(s).unwrap(),
                                    4 => lit.0.parse::<u32>().unwrap().serialize(s).unwrap(),
                                    8 => lit.0.parse::<u64>().unwrap().serialize(s).unwrap(),
                                    16 => lit.0.parse::<u128>().unwrap().serialize(s).unwrap(),
                                    _ => unimplemented!()
                                }
                            } else {
                                match size {
                                    1 => lit.0.parse::<i8>().unwrap().serialize(s).unwrap(),
                                    2 => lit.0.parse::<i16>().unwrap().serialize(s).unwrap(),
                                    4 => lit.0.parse::<i32>().unwrap().serialize(s).unwrap(),
                                    8 => lit.0.parse::<i64>().unwrap().serialize(s).unwrap(),
                                    16 => lit.0.parse::<i128>().unwrap().serialize(s).unwrap(),
                                    _ => unimplemented!()
                                };
                            }
                        },
                        _ => unimplemented!()
                    }
                }
                unimplemented!()
            },
        }
    }

    pub fn parse_module_tree(&mut self, module_id:Id, min_level:usize) -> usize {
        let module_id = Id(module_id.0.to_lowercase());
        if !self.parsed.contains_key(&module_id){
            let mut f = File::open(self.get_module_path(&module_id)).unwrap();
            let mut contents = String::new();
            f.read_to_string(&mut contents).unwrap();

            let module = match parser::ModuleParser::new().parse(&contents){
                Ok(m) => m,
                Err(err) => {
                    panic!("{:?} in {:?}",err, module_id)
                },
            };

            if module.name.0.to_lowercase() != module_id.0.to_lowercase() {
                panic!()
            }
            let deps = module.collect_module_dependencies();
            let index = module.create_index();
            let mut level = min_level;
            for dep in deps {
                level = level.max(self.parse_module_tree(dep, min_level)+1)
            }
            self.parsed.insert(module_id, ModuleEntry{ level, index, module, hash: None });
            level
        } else {
            self.parsed[&module_id].level
        }
    }

    fn compile_fun_comp(mut imp:CodeImportBuilder, vis:&Visibility, transactional:bool, generics:&[Generic], params:&[Var], returns:&[Ret], code:&Block) -> Result<FunctionComponent,String> {
        let visibility = vis.compile(&generics);
        let generics = generics.iter().map(|g|g.compile()).collect();

        let alloc = NoCustomAlloc();
        let mut env = Environment::new(&alloc);
        let params =  params.iter().map(|p|p.compile(&mut env, &mut imp)).collect::<Result<_,_>>()?;
        let code = code.compile(&mut env, &mut imp)?;
        let returns =  returns.iter().map(|r|r.compile(&mut env, &mut imp)).collect::<Result<_,_>>()?;


        let (callables, permissions, import) = imp.generate_body_import();
        Ok(FunctionComponent {
            byte_size: None,
            shared: FunSigShared {
                generics,
                import,
                params,
                returns,
                transactional
            },
            scope: visibility,
            body: CallableImpl::Internal {
                byte_size: None,
                imports:BodyImport{
                    public: PublicImport {
                        modules: vec![],
                        types: vec![]
                    },
                    callables,
                    permissions
                },
                code
            }

        })
    }

    pub fn compile_module_tree(&mut self) -> Result<(),String> {
        let ordered:Vec<Id> = self.parsed
            .iter().sorted_by(|m1,m2|m1.1.level.cmp(&m2.1.level))
            .filter(|(id,_)|!self.compiled.contains_key(id))
            .map(|(id,_)| (*id).clone()).collect();
        for id in ordered {
            let mut r_module = RModule {
                byte_size: None,
                meta: LargeVec(id.0.to_lowercase().clone().into_bytes()),
                data: Vec::new(),
                sigs: Vec::new(),
                data_sig_order: BitSerializedVec(Vec::new()),
                functions: Vec::new(),
                implements: Vec::new(),
                fun_impl_order: BitSerializedVec(Vec::new())
            };
            for comp in &self.parsed[&id].module.components {
                match *comp {
                    Component::ExtLit { top, ref perms, caps, ref generics, size, .. } => {
                        let imp = CodeImportBuilder::new(&self.parsed[&id], generics, &self.parsed);
                        let create_vis = get_perm(&perms,"create").compile(&generics);
                        let consume_vis = get_perm(&perms,"consume").compile(&generics);
                        let inspect_vis = get_perm(&perms,"inspect").compile(&generics);

                        let generics = generics.iter().map(|g|g.compile()).collect();
                        let import = imp.generate_import();

                        r_module.data_sig_order.0.push(true);
                        r_module.data.push(DataComponent {
                            byte_size: None,
                            create_scope: create_vis,
                            consume_scope: consume_vis,
                            inspect_scope: inspect_vis,
                            provided_caps: caps,
                            top,
                            generics,
                            import,
                            body: DataImpl::External(size)
                        })
                    },

                    Component::Adt { top, caps, ref perms, ref generics, ref ctrs,  .. } => {
                        let mut imp = CodeImportBuilder::new(&self.parsed[&id], generics, &self.parsed);
                        let create_vis = get_perm(&perms,"create").compile(&generics);
                        let consume_vis = get_perm(&perms,"consume").compile(&generics);
                        let inspect_vis = get_perm(&perms,"inspect").compile(&generics);
                        let generics = generics.iter().map(|g|g.compile()).collect();
                        let constructors =  ctrs.iter().map(|p|p.compile(&mut imp)).collect::<Result<_,_>>()?;
                        let import = imp.generate_import();

                        r_module.data_sig_order.0.push(true);
                        r_module.data.push(DataComponent {
                            byte_size: None,
                            create_scope: create_vis,
                            consume_scope: consume_vis,
                            inspect_scope: inspect_vis,
                            provided_caps: caps,
                            top,
                            generics,
                            import,
                            body: DataImpl::Internal {
                                constructors
                            }
                        })
                    },
                    Component::ExtFun { ref perms, transactional, ref generics, ref params, ref returns , .. } => {
                        let mut imp = CodeImportBuilder::new(&self.parsed[&id], generics, &self.parsed);

                        let visibility = get_perm(&perms,"call").compile(&generics);
                        let generics = generics.iter().map(|g|g.compile()).collect();

                        let alloc = NoCustomAlloc();
                        let mut env = Environment::new(&alloc);
                        let params =  params.iter().map(|p|p.compile(&mut env, &mut imp)).collect::<Result<_,_>>()?;
                        let returns =  returns.iter().map(|r|r.compile(&mut env, &mut imp)).collect::<Result<_,_>>()?;

                        let import = imp.generate_import();
                        r_module.fun_impl_order.0.push(true);
                        r_module.functions.push(FunctionComponent {
                            byte_size: None,
                            shared: FunSigShared {
                                generics,
                                import,
                                params,
                                returns,
                                transactional
                            },
                            scope: visibility,
                            body: CallableImpl::External
                        });
                    },
                    Component::Fun { ref perms, transactional, ref generics, ref params, ref returns ,ref code, .. } => {
                        let imp = CodeImportBuilder::new(&self.parsed[&id], generics, &self.parsed);
                        r_module.fun_impl_order.0.push(true);
                        r_module.functions.push(Self::compile_fun_comp(imp,&get_perm(&perms,"call"),transactional, generics,params,returns,code)?);
                    },
                    Component::Impl { ref perms, ref generics, ref captures, ref sig, ref params, ref code, .. } => {
                        let mut imp = CodeImportBuilder::new(&self.parsed[&id], generics, &self.parsed);
                        let visibility = get_perm(&perms,"call").compile(&generics);
                        let generics = generics.iter().map(|g|g.compile()).collect();
                        let sig_type = imp.import_perm_ref(&(Perm::Implement,sig.clone()))?;

                        let alloc = NoCustomAlloc();
                        let mut env = Environment::new(&alloc);
                        let captures =  captures.iter().map(|p|p.compile(&mut env, &mut imp)).collect::<Result<_,_>>()?;
                        for p in params {env.push_new(p.clone())}
                        let code = code.compile(&mut env, &mut imp)?;

                        let (callables,  permissions, import) = imp.generate_body_import();
                        r_module.fun_impl_order.0.push(false);
                        r_module.implements.push(ImplementComponent {
                            byte_size: None,
                            generics,
                            import,
                            params:captures,
                            scope: visibility,
                            sig:sig_type,
                            body: CallableImpl::Internal {
                                byte_size: None,
                                imports:BodyImport{
                                    public: PublicImport {
                                        modules: vec![],
                                        types: vec![]
                                    },
                                    callables,
                                    permissions
                                },
                                code
                            }
                        });
                    },
                    Component::Sig{  ref perms, transactional, caps, ref generics, ref params, ref returns, ..} => {
                        let mut imp = CodeImportBuilder::new(&self.parsed[&id], generics, &self.parsed);

                        let call_vis = get_perm(&perms,"call").compile(&generics);
                        let implement_vis = get_perm(&perms,"implement").compile(&generics);
                        let generics:Vec<RGeneric> = generics.iter().map(|g|g.compile()).collect();

                        let alloc = NoCustomAlloc();
                        let mut env = Environment::new(&alloc);
                        let params =  params.iter().map(|p|p.compile(&mut env, &mut imp)).collect::<Result<_,_>>()?;
                        let returns =  returns.iter().map(|r|r.compile(&mut env, &mut imp)).collect::<Result<_,_>>()?;

                        let import = imp.generate_import();
                        r_module.data_sig_order.0.push(false);
                        r_module.sigs.push(SigComponent {
                            byte_size: None,
                            call_scope: call_vis,
                            implement_scope: implement_vis,
                            shared: FunSigShared {
                                generics,
                                import,
                                params,
                                returns,
                                transactional
                            },
                            provided_caps:caps,
                        });
                    }
                    Component::Txt { .. } => unreachable!()
                }
            }

            let mut s = Serializer::new(usize::max_value());
            r_module.serialize(&mut s)?;
            let res = s.extract();
            let hash = store_hash(&[&res]);
            let module = self.parsed.get_mut(&id).unwrap();
            module.hash = Some(hash);
            self.compiled.insert(id, res);
        }
        Ok(())
    }

    pub fn get_module_results(&self) -> Vec<(Id, Vec<u8>)> {
        self.parsed.iter()
            .filter(|(id,_)|self.compiled.contains_key(id))
            .sorted_by(|m1,m2|m1.1.level.cmp(&m2.1.level))
            .map(|(id,_)|((*id).clone(), self.compiled[id].clone())).collect()
    }

    pub fn get_functions_to_deploy(&self) -> Vec<Vec<u8>> {
        self.txt_funs.iter().map(|tcr| tcr.txt_fun.clone()).collect()
    }

    pub fn create_bundle(&self, hashes:&[Hash], heap:&Heap) -> Vec<u8> {
        let alloc =  heap.new_arena(CONFIG.max_transaction_memory);

        let mut env = BundleImportBuilder::new();

        let built_txts:Vec<Transaction> = self.txt_funs.iter()
            .zip(hashes.iter())
            .map(|(txt,hash)|self.build_txt(txt,hash.clone(), &mut env, &alloc))
            .collect();

        /*let pay_section = BundleSection {
            typ: SectionType::Payment,
            extra_entries_limit: 0,
            storage_volume_limit: 0,
            gas_limit: 0,
            txts: SlicePtr::empty(),
        };*/

        //Todo: Compute these costs
        let main_section = BundleSection {
            typ: SectionType::Custom,
            entries_loaded: 1000,
            entries_created: 1000,
            entries_deleted: 1000,
            gas_limit: 10000000,
            txts: alloc.copy_alloc_slice(&built_txts).unwrap()
        };

        let store_witness = env.empty_storage_witnesses(&alloc);
        let witness = env.witnesses(&alloc);
        let witness_size = witness.iter().map(|w|w.len() + 2).sum::<usize>() + 2;  //2 is num wittness / Num Bytes
        let store_witness_size = store_witness.iter().map(|w|w.map_or(0,|d|d.len()+2)+1).sum::<usize>() + 2;  //2 is num wittness / Num Bytes

        let bundle = TransactionBundle {
            byte_size: None,
            core: TransactionBundleCore {
                byte_size: None,
                meta:SlicePtr::empty(),
                deploy: None,
                transaction_storage_heap: 10000,
                param_heap_limit: 10000,
                stack_elem_limit: 1000,
                stack_frame_limit: 1000,
                runtime_heap_limit: 10000,
                sections: alloc.copy_alloc_slice(&[/*pay_section, */main_section]).unwrap(),
                descriptors: alloc.copy_alloc_slice(env.descs()).unwrap(),
                stored: env.values(&alloc),
                literal: env.literals(&alloc),
                witness_bytes_limit: (witness_size + store_witness_size) as u32,               
            },
            witness,
            //todo: fill over state provider runtime extension later: when ready
            store_witness
        };
        Serializer::serialize_fully(&bundle,CONFIG.max_structural_dept).unwrap()
    }


    fn build_txt<'b>(&self, txt_res:&TransactionCompilationResult, fun:Hash, env:&mut BundleImportBuilder, alloc:&'b HeapArena) -> Transaction<'b> {

        let params: Vec<ParamRef> = txt_res.param_types.iter().map(|p|env.param_ref(p)).collect();

        Transaction{
            txt_desc: env.desc_ref(&fun),
            params: alloc.copy_alloc_slice(&params).unwrap(),
            returns: alloc.copy_alloc_slice(&txt_res.ret_types).unwrap()
        }

    }
}

fn get_perm(perms:&[(Vec<Id>, Visibility)], id:&str) -> Visibility {
    let search_id = &Id(id.to_string());
    let default_id = &Id("default".to_string());
    let mut default = Visibility::Private;
    for (ids,vis) in perms {
        if ids.contains(search_id) {
            return vis.clone()
        }
        if ids.contains(default_id) {
            default = vis.clone()
        }
    }
    return default;
}
