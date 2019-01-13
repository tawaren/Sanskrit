use std::fs::File;
use std::io::Read;
use std::path::Path;
use model::*;
use std::path::PathBuf;
use std::collections::HashMap;
use sanskrit_core::model::Module as RModule;
use sanskrit_common::model::Hash;
use sanskrit_common::model::LargeVec;
use environment::CodeImportBuilder;
use sanskrit_core::model::AdtComponent;
use environment::Environment;
use sanskrit_core::model::FunctionComponent;
use sanskrit_common::store::store_hash;
use sanskrit_common::encoding::*;
use itertools::Itertools;
use module_parser;
use std::collections::HashSet;

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
    compiled:HashMap<Id,Vec<u8>>
}


impl<'a> Compiler<'a> {
    pub fn new(folder:&'a Path) -> Self {
        Compiler{
            folder,
            parsed: HashMap::new(),
            compiled:  HashMap::new(),
        }
    }

    fn get_module_path(&self, module:&Id) -> PathBuf {
        self.folder.join(&module.0.to_lowercase()).with_extension("sans")
    }

    pub fn get_direct_dependencies(&self, module_id:Id) -> HashSet<Id> {
            let mut f = File::open(self.get_module_path(&module_id)).unwrap();
            let mut contents = String::new();
            f.read_to_string(&mut contents).unwrap();

            let module = match module_parser::ModuleParser::new().parse(&contents){
                Ok(m) => m,
                Err(err) => {
                    println!("{:?}",err);
                    panic!()
                },
            };

            if module.name.0.to_lowercase() != module_id.0.to_lowercase() {
                panic!()
            }
            module.collect_module_dependencies()
    }

    pub fn parse_module_tree(&mut self, module_id:Id) -> usize {
        if !self.parsed.contains_key(&module_id){
            let mut f = File::open(self.get_module_path(&module_id)).unwrap();
            let mut contents = String::new();
            f.read_to_string(&mut contents).unwrap();

            let module = match module_parser::ModuleParser::new().parse(&contents){
                Ok(m) => m,
                Err(err) => {
                    println!("{:?} in {:?}",err, module_id);
                    panic!()
                },
            };

            if module.name.0.to_lowercase() != module_id.0.to_lowercase() {
                panic!()
            }
            let deps = module.collect_module_dependencies();
            let index = module.create_index();
            let mut level = 0;
            for dep in deps {
                level = level.max(self.parse_module_tree(dep)+1)
            }
            self.parsed.insert(module_id, ModuleEntry{ level, index, module, hash: None });
            level
        } else {
            self.parsed[&module_id].level
        }
    }

    pub fn compile_module_tree(&mut self) -> Result<(),String> {
        let ordered:Vec<Id> = self.parsed
            .iter().sorted_by(|m1,m2|m1.1.level.cmp(&m2.1.level))
            .filter(|(id,_)|!self.compiled.contains_key(id))
            .map(|(id,_)| (*id).clone()).collect();
        for id in ordered {
            let mut r_module = RModule {
                meta: LargeVec(id.0.clone().into_bytes()),
                adts: Vec::new(),
                functions: Vec::new(),
                errors: 0
            };
            for comp in &self.parsed[&id].module.components {
                match *comp {
                    Component::Adt { caps, ref generics, ref ctrs, ref name,  .. } => {
                        let mut imp = CodeImportBuilder::new(&self.parsed[&id], generics, &self.parsed);

                        let generics = generics.iter().map(|g|g.compile(&mut imp)).collect();
                        let constructors =  ctrs.iter().map(|p|p.compile(&mut imp)).collect::<Result<_,_>>()?;
                        let import = imp.generate_import();

                        r_module.adts.push(AdtComponent {
                            provided_caps: caps,
                            generics,
                            import,
                            constructors,
                        })
                    },
                    Component::Err { .. } => r_module.errors+=1,
                    Component::Fun { ref vis, ref risks, ref generics, ref params, ref returns ,ref code, .. } => {
                        let mut imp = CodeImportBuilder::new(&self.parsed[&id], generics, &self.parsed);

                        let visibility = vis.compile(&generics,&mut imp);
                        let generics = generics.iter().map(|g|g.compile(&mut imp)).collect();
                        let risk = risks.iter().map(|r|imp.import_err_ref(r)).collect::<Result<_,_>>()?;

                        let alloc = NoCustomAlloc();
                        let mut env = Environment::new(&alloc);
                        let params =  params.iter().map(|p|p.compile(&mut env, &mut imp)).collect::<Result<_,_>>()?;
                        let code = code.compile(&mut env, &mut imp)?;
                        let returns =  returns.iter().map(|r|r.compile(&mut env, &mut imp)).collect::<Result<_,_>>()?;


                        let import = imp.generate_body_import();
                        r_module.functions.push(FunctionComponent {
                            generics,
                            visibility,
                            import,
                            risk,
                            params,
                            returns,
                            code
                        });
                    },
                }
            }

            let mut s = Serializer::new();
            r_module.serialize(&mut s);
            let res = s.extract();
            let hash = store_hash(&[&res]);
            let module = self.parsed.get_mut(&id).unwrap();
            module.hash = Some(hash);
            self.compiled.insert(id, res);
        }
        Ok(())
    }

    pub fn get_results(&self) -> Vec<(Id,Vec<u8>)> {
        self.parsed.iter()
            .filter(|(id,_)|self.compiled.contains_key(id))
            .sorted_by(|m1,m2|m1.1.level.cmp(&m2.1.level))
            .map(|(id,_)|((*id).clone(), self.compiled[id].clone())).collect()
    }
}
