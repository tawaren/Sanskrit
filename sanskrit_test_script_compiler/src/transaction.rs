use std::fs::File;
use std::io::Read;
use std::path::Path;
use model::*;
use std::path::PathBuf;
use std::collections::HashMap;
use environment::ScriptContext;
use environment::Environment;
use sanskrit_common::encoding::*;
use transaction_parser;
use script::Compiler as MCompiler;
use ed25519_dalek::Keypair;
use sha2::{Sha512};
use rand::rngs::OsRng;
use sanskrit_runtime::model::Transaction as RTransaction;
use std::mem;
use sanskrit_common::arena::HeapArena;
use sanskrit_common::model::SlicePtr;
use sanskrit_common::arena::Heap;
use sanskrit_runtime::model::ImpRef;
//Process text -> Adt -> Adt + Index (allows to get stuff by Id) -> EAst (has stuff like captures) -> Final
//All involved file go seq:
// First collect dependencies & Bring everithing into Adt Form
// Second Calc Index for everithing
// Third Enrich Ast with missing types & captures
// Produce finals (Starting at Leaves)
// Deploy everithing (Starting at Leaves)



pub struct Compiler<'a> {
    folder:&'a Path,
    modules: MCompiler<'a>,
    parsed_txts:Vec<Transaction>,
    compiled_txts:Vec<Vec<u8>>,
}

pub struct CompiledTransactions{
    pub modules:Vec<(Id,Vec<u8>)>,
    pub txts:Vec<Vec<u8>>,
}


impl<'a> Compiler<'a> {
    pub fn new(folder:&'a Path) -> Self {
        Compiler{
            folder,
            modules: MCompiler::new(folder),
            parsed_txts: vec![],
            compiled_txts: vec![],

        }
    }

    fn get_transaction_path(&self, txt:&Id) -> PathBuf {
        self.folder.join(&txt.0.to_lowercase()).with_extension("txt")
    }

    pub fn parse_transactions(&mut self, txt:Id){
            let mut f = File::open(self.get_transaction_path(&txt)).unwrap();
            let mut contents = String::new();
            f.read_to_string(&mut contents).unwrap();

            self.parsed_txts = match transaction_parser::TransactionsParser::new().parse(&contents){
                Ok(m) => m,
                Err(ref err) => {
                    println!("{}",err);
                    panic!()
                }
            };

            for txt in &self.parsed_txts {
                let deps = txt.collect_module_dependencies();
                for dep in deps {
                   self.modules.parse_module_tree(dep);
                }
            }
    }

    pub fn compile_transactions(&mut self) -> Result<(),String> {
        self.compile_transactions_with_accounts(HashMap::new())
    }

    pub fn compile_transactions_with_accounts(&mut self, mut sks:HashMap<Id,Keypair>) -> Result<(),String> {
        self.modules.compile_module_tree()?;
        let mut csprng: OsRng = OsRng::new().unwrap();
        for txt in &self.parsed_txts {
            for s in &txt.sigs {
                if !sks.contains_key(s){
                    let kp = Keypair::generate::<Sha512, _>(&mut csprng);
                    sks.insert(s.clone(),kp);
                }
            }
        }

        //Helper to aid the borrow checker realize that txt_alloc is no longer used after this
        fn compile_txt(txt:&Transaction, modules:&MCompiler, sks:&HashMap<Id,Keypair>, txt_alloc:&HeapArena) -> Result<Vec<u8>,String> {
            let sigs = txt.sigs.iter().enumerate().map(|(idx,id)|(id.clone(),idx as u8)).collect();
            let news = txt.news.iter().enumerate().map(|(idx,id)|(id.clone(),idx as u8)).collect();
            let signers = txt_alloc.iter_alloc_slice(txt.sigs.iter().map(|id|sks[id].public.to_bytes()))?;

            let mut code = txt_alloc.slice_builder(txt.codes.len())?;
            let mut env = Environment::new_with_ctx(txt_alloc);
            let mut imp = ScriptContext::new(&modules.parsed, &sigs,&news, txt_alloc);
            for c in &txt.codes {
                code.push(c.compile(&mut env,&mut imp)?)
            }

            let mut tmp_vec = vec![[0;20];imp.imports.len()];
            for (hash,ImpRef(idx)) in imp.imports {
                assert_eq!(tmp_vec[idx as usize], [0;20]);
                tmp_vec[idx as usize] = hash
            }


            let mut r_txt = RTransaction {
                start_block_no: 0,
                signers,
                imports: txt_alloc.copy_alloc_slice(&tmp_vec)?,
                new_types: txt.news.len() as u8,
                code:code.finish(),
                signatures: SlicePtr::empty(),
                witness: SlicePtr::empty(),
            };

            let mut s = Serializer::new(usize::max_value());
            r_txt.serialize(&mut s)?;
            let txt_data = s.extract();
            //Serialization Specific
            r_txt.signatures =  txt_alloc.iter_alloc_slice(txt.sigs.iter().map(|id|{
                sks[id].sign::<Sha512>(&txt_data[0..(txt_data.len()-4)]).to_bytes() //-4 are the 4 bytes serialized for signatures & witness
            }))?;

            r_txt.witness = txt_alloc.copy_alloc_slice(&imp.wit)?;

            let mut s = Serializer::new(usize::max_value());
            r_txt.serialize(&mut s)?;
            Ok(s.extract())
        }

        let heap = Heap::new(10000,1.0);
        let mut txt_alloc = heap.new_arena(10000);
        for txt in &self.parsed_txts {
            self.compiled_txts.push(compile_txt(txt, &self.modules, &sks, &txt_alloc)?);
            txt_alloc = txt_alloc.reuse();
        }

        Ok(())
    }

    pub fn extract_results(&mut self) -> CompiledTransactions {
        let mut txts = vec![];
        mem::swap(&mut txts, &mut self.compiled_txts);

        CompiledTransactions{
            modules: self.modules.get_results(),
            txts
        }
    }

    pub fn get_module_compiler<'b>(&'b mut self) -> &'b mut MCompiler<'a> {
        &mut self.modules
    }
}
