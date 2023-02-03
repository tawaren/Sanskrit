
#[cfg(test)]
mod tests {
    use sanskrit_common::errors::*;
    use sanskrit_test_script_compiler::model::Id;
    use sanskrit_deploy::*;
    use sanskrit_compile::*;
    use test::Bencher;
    use sanskrit_memory_store::BTreeMapStore;
    use std::env::current_dir;
    use sanskrit_common::arena::Heap;
    use sanskrit_test_script_compiler::script::Compiler;
    use std::cell::Cell;
    use sanskrit_runtime::{CONFIG, execute, Tracker, Context};
    use sanskrit_common::store::StorageClass;
    use std::fs::File;
    use std::io::{BufReader, BufRead};
    use externals::{ScriptExternals, ScriptSystem};
    use sanskrit_interpreter::model::{TxTReturn, Entry, TxTParam};
    use sanskrit_runtime::model::{Transaction, RetType, BundleSection, ParamRef};
    use sanskrit_common::encoding::Serializer;
    use sanskrit_runtime::system::SystemContext;

    struct CheckedLogger{
        expects:Vec<String>
    }
    impl Tracker for CheckedLogger {
        fn section_start(&mut self, _section: &BundleSection) {}
        fn transaction_start(&mut self, _transaction: &Transaction) {}
        fn parameter_load(&mut self, _p_ref: &ParamRef, _p_desc: &TxTParam, _value: &Entry) {}

        fn return_value(&mut self, r_typ: &RetType, r_desc: &TxTReturn, value: &Entry) {
            if *r_typ == RetType::Log {
                let expect = self.expects.pop();
                let mut s = Serializer::new(CONFIG.max_structural_dept);
                r_desc.desc.serialize_value(*value, &mut s).unwrap();
                let data =  s.extract();
                assert!(expect.is_some(), format!("Found {:?} - But nothing expected", data));
                assert_eq!(format!("{:?}",data), expect.unwrap());
            }
        }

        fn transaction_finish(&mut self, _transaction: &Transaction, _success: bool) {}
        fn section_finish(&mut self, _section: &BundleSection, _success: bool) {}
    }


    fn parse_and_compile_and_run(id_name:&str) -> Result<()>{
        let id = Id(id_name.into());
        let folder = current_dir().unwrap().join("transactions");
        let mut comp = Compiler::new(&folder);
        comp.parse_module_tree(Id("system".into()), 0);
        comp.parse_and_compile_transactions(id.clone())?;
        let mod_res = comp.get_module_results();
        let txt_res = comp.get_functions_to_deploy();
        let s = BTreeMapStore::new();
        for (i,sys_mode, r) in mod_res {
            println!("M: {:?}",i);

            deploy_module(&s, r, sys_mode, true)?;
        }

        let mut heap = Heap::new(CONFIG.calc_heap_size(2),2.0);

        let res_path = folder.join(id_name.to_lowercase()).with_extension("res");
        let mut checker = if let Ok(res_file) = File::open(res_path) {
            let reader = BufReader::new(res_file);
            let mut expects: Vec<String>  = reader.lines().map(|l| l.expect("Could not parse line")).collect();
            expects.reverse();
            CheckedLogger{expects}
        } else {
            CheckedLogger{expects:vec![]}
        };

        let mut hashes = Vec::with_capacity(txt_res.len());
        for t in txt_res {
            let fun_id = deploy_function(&s, t.clone(), true)?;
            hashes.push(compile_function::<_,ScriptExternals>(&s,fun_id, true)?.0);
        }

        let bundle = comp.create_bundle(&hashes, &heap);


        heap = heap.reuse();
        let txt_bundle_alloc = heap.new_virtual_arena(CONFIG.max_txt_alloc);
        let txt_bundle= ScriptSystem::parse_bundle(&bundle,&txt_bundle_alloc)?;
        let ctx = Context {
            store: &s,
            txt_bundle: &txt_bundle
        };
        execute::<_,ScriptSystem>(ctx, 0, &heap, &mut checker).expect("Execute Failed");
        assert_eq!(checker.expects.len(), 0, "Expected more logs");

        Ok(())
    }

    struct NoLogger{}
    impl Tracker for NoLogger {
        fn section_start(&mut self, _section: &BundleSection) {}
        fn transaction_start(&mut self, _transaction: &Transaction) {}
        fn parameter_load(&mut self, _p_ref: &ParamRef, _p_desc: &TxTParam, _value: &Entry) {}
        fn return_value(&mut self, _r_typ: &RetType, _r_desc: &TxTReturn, _value: &Entry) {}
        fn transaction_finish(&mut self, _transaction: &Transaction, _success: bool) {}
        fn section_finish(&mut self, _section: &BundleSection, _success: bool) {}
    }

    fn parse_and_compile_and_run_bench(id_name:&str,b: &mut Bencher) -> Result<()>{
        let id = Id(id_name.into());
        let folder = current_dir().unwrap().join("transactions");
        let mut comp = Compiler::new(&folder);
        comp.parse_module_tree(Id("system".into()), 0);
        comp.parse_and_compile_transactions(id.clone())?;
        let mod_res = comp.get_module_results();
        let txt_res = comp.get_functions_to_deploy();
        let s = BTreeMapStore::new();
        for (i,sys_mode, r) in mod_res {
            println!("M: {:?}",i);
            deploy_module(&s, r, sys_mode, true)?;
        }

        let mut hashes = Vec::with_capacity(txt_res.len());
        for t in txt_res {
            let fun_id = deploy_function(&s, t.clone(), true)?;
            hashes.push(compile_function::<_,ScriptExternals>(&s,fun_id, true)?.0);
        }


        b.iter(move ||{
            let mut heap = Heap::new(CONFIG.calc_heap_size(2),2.0);
            let bundle = comp.create_bundle(&hashes, &heap);
            heap = heap.reuse();
            let txt_bundle_alloc = heap.new_virtual_arena(CONFIG.max_txt_alloc);
            let txt_bundle= ScriptSystem::parse_bundle(&bundle,&txt_bundle_alloc).unwrap();
            let ctx = Context {
                store: &s,
                txt_bundle: &txt_bundle
            };
            execute::<_,ScriptSystem>(ctx, 0, &heap, &mut NoLogger{}).expect("Execute Failed");
            s.clear_section(StorageClass::EntryHash);
            s.clear_section(StorageClass::EntryValue);
        });


        Ok(())
    }

    #[bench]
    fn build_diff_adts_bench_deploy(b: &mut Bencher) {
        parse_and_compile_and_run_bench("simple",b).unwrap();
    }

    #[test]
    fn simple_txt() {
        parse_and_compile_and_run("simple").unwrap();
    }

    #[test]
    fn succ_move() {
        parse_and_compile_and_run("testSuccMove").unwrap();
    }

    #[test]
    fn succ_pack() {
        parse_and_compile_and_run("testSuccPack").unwrap();
    }

    #[test]
    fn succ_build_rec() {
        parse_and_compile_and_run("testSuccBuildRec").unwrap();
    }

    #[test]
    fn succ_base_ops() {
        parse_and_compile_and_run("testSuccBaseOps").unwrap();
    }

    #[bench]
    fn succ_base_ops_bench(b: &mut Bencher) {
        parse_and_compile_and_run_bench("testSuccBaseOps",b).unwrap();
    }

    #[test]
    fn succ_storage_drop() {
        parse_and_compile_and_run("testSuccStorageWithDrop").unwrap();
    }

    #[test]
    fn succ_storage_copy() {
        parse_and_compile_and_run("testSuccStorageWithCopy").unwrap();
    }

    #[bench]
    fn succ_storage_copy_drop_bench(b: &mut Bencher) {
        parse_and_compile_and_run_bench("testSuccStorageWithDropAndCopy",b).unwrap();
    }

    #[test]
    fn succ_storage_copy_drop() {
        parse_and_compile_and_run("testSuccStorageWithDropAndCopy").unwrap();
    }
    #[bench]
    fn succ_math_bench(b: &mut Bencher) {
        parse_and_compile_and_run_bench("testSuccMath",b).unwrap();
    }

    #[test]
    fn succ_math() {
        parse_and_compile_and_run("testSuccMath").unwrap();
    }

    #[test]
    fn succ_bit_ops() {
        parse_and_compile_and_run("testSuccBitOps").unwrap();
    }


    #[bench]
    fn succ_compare_hash_ops_bench(b: &mut Bencher) {
        parse_and_compile_and_run_bench("testSuccCompareHashOps",b).unwrap();
    }

    #[test]
    fn succ_compare_hash_ops() {
        parse_and_compile_and_run("testSuccCompareHashOps").unwrap();
    }

    #[test]
    fn succ_error_ops() {
        parse_and_compile_and_run("testSuccErrorOps").unwrap();
    }


    #[bench]
    fn succ_index_ops_bench(b: &mut Bencher) {
        parse_and_compile_and_run_bench("testSuccIndexOps",b).unwrap();
    }

    #[test]
    fn succ_index_ops() {
        parse_and_compile_and_run("testSuccIndexOps").unwrap();
    }


    /*
    #[test]
    fn succ_cast_ops() {
        parse_and_compile_and_run("testSuccCastOps").unwrap();
    }
    */

    #[test]
    #[should_panic(expected="A private permission must be from the current module")]
    fn no_unpack_cap() {
        parse_and_compile_and_run("testFailCapUnpack").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_moved() {
        parse_and_compile_and_run("testFailConsumeMoved").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_moved2() {
        parse_and_compile_and_run("testFailConsumeMoved2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_moved3() {
        parse_and_compile_and_run("testFailConsumeMoved3").unwrap();
    }

    #[test]
    #[should_panic(expected="A consumed or locked element can not be fetched")]
    fn copy_moved() {
        parse_and_compile_and_run("testFailCopy").unwrap();
    }

    #[test]
    #[should_panic(expected="A consumed or locked element can not be fetched")]
    fn copy_moved2() {
        parse_and_compile_and_run("testFailCopyConsumed").unwrap();
    }

    #[test]
    #[should_panic(expected="A private permission must be from the current module")]
    fn create_fail() {
        parse_and_compile_and_run("testFailCreate").unwrap();
    }

    #[test]
    #[should_panic(expected="A consumed, locked or hidden element can not be hidden")]
    fn same_arg() {
        parse_and_compile_and_run("testFailDualArg").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn wrong_num_fun_gens() {
        parse_and_compile_and_run("testFailFunApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn wrong_num_fun_gens2() {
        parse_and_compile_and_run("testFailFunApply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong number of parameter for function call")]
    fn wrong_num_fun_param() {
        parse_and_compile_and_run("testFailFunCall").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong number of parameter for function call")]
    fn wrong_num_fun_param2() {
        parse_and_compile_and_run("testFailFunCall2").unwrap();
    }

    #[test]
    #[should_panic(expected="Parameter for function call has wrong type")]
    fn wrong_fun_arg_type() {
        parse_and_compile_and_run("testFailFunCall3").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn wrong_type_arg_contraint() {
        parse_and_compile_and_run("testFailFunConstraintApply").unwrap();
    }

    #[test]
    #[should_panic(expected= "Capabilities of type must full fill the constraints")]
    fn wrong_type_arg_contraint2() {
        parse_and_compile_and_run("testFailTypeConstraintApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn type_apply() {
        parse_and_compile_and_run("testFailTypeApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn type_apply2() {
        parse_and_compile_and_run("testFailTypeApply2").unwrap();
    }


    #[test]
    #[should_panic(expected="Parameter for data constructor has wrong type")]
    fn fail_pack_type() {
        parse_and_compile_and_run("testFailPackType").unwrap();
    }

    #[test]
    #[should_panic(expected="Parameter for data constructor has wrong type")]
    fn fail_pack_type2() {
        parse_and_compile_and_run("testFailPackType2").unwrap();
    }


    #[test]
    #[should_panic(expected="Requested constructor does not exist")]
    fn fail_pack_param() {
        parse_and_compile_and_run("testFailTypeParam").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested constructor does not exist")]
    fn fail_pack_param2() {
        parse_and_compile_and_run("testFailTypeParam2").unwrap();
    }

    #[test]
    #[should_panic(expected="A private permission must be from the current module")]
    fn fail_priv_call() {
        parse_and_compile_and_run("testFailPrivateCall").unwrap();
    }

    #[test]
    #[should_panic(expected="A type from the current module is required to be applied to a guarded generic")]
    fn fail_prot_call() {
        parse_and_compile_and_run("testFailProtectedCall").unwrap();
    }

    #[test]
    #[should_panic(expected="A type from the current module is required to be applied to a guarded generic")]
    fn fail_prot_call2() {
        parse_and_compile_and_run("testFailProtectedCall2").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong Permission supplied")]
    fn unpack_fail() {
        parse_and_compile_and_run("testFailUnpackType").unwrap();
    }


    #[test]
    #[should_panic(expected="Unpack must target a data type with a single constructor")]
    fn unpack_fail2() {
        parse_and_compile_and_run("testFailUnpackType2").unwrap();
    }


    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn lit_missapply() {
        parse_and_compile_and_run("testFailLitMissapply").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn lit_missapply2() {
        parse_and_compile_and_run("testFailLitMissapply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn lit_missapply3() {
        parse_and_compile_and_run("testFailLitMissapply3").unwrap();
    }

    #[test]
    #[should_panic(expected= "Applied types mismatch required generics")]
    fn lit_missapply4() {
        parse_and_compile_and_run("testFailLitMissapply4").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn lit_missapply5() {
        parse_and_compile_and_run("testFailLitMissapply5").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn lit_missapply6() {
        parse_and_compile_and_run("testFailLitMissapply6").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn lit_missapply7() {
        parse_and_compile_and_run("testFailLitMissapply7").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not store primitives")]
    fn store_cap_fail() {
        parse_and_compile_and_run("testFailNoIndex").unwrap();
    }

    #[test]
    #[should_panic(expected="Stored return must be an entry")]
    fn store_cap_fail2() {
        parse_and_compile_and_run("testFailNoIndex2").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn store_cap_fail3() {
        parse_and_compile_and_run("testFailNoPersist").unwrap();
    }

    #[test]
    #[should_panic(expected="Value was not in store")]
    fn load_missing() {
        parse_and_compile_and_run("testFailLoad").unwrap();
    }


    #[test]
    #[should_panic(expected="Value was not in store")]
    fn load_missing2() {
        parse_and_compile_and_run("testFailLoad2").unwrap();
    }

    #[test]
    #[should_panic(expected="Value was not in store")]
    fn load_missing3() {
        parse_and_compile_and_run("testFailLoad3").unwrap();
    }

    #[test]
    #[should_panic(expected="stored value had wrong type")]
    fn load_type_missmatch() {
        parse_and_compile_and_run("testFailLoadWrongType").unwrap();
    }

    #[test]
    #[should_panic(expected="Value was already in store")]
    fn store_full() {
        parse_and_compile_and_run("testFailStore").unwrap();
    }

    #[test]
    #[should_panic(expected="Value was already in store")]
    fn store_full2() {
        parse_and_compile_and_run("testFailStore2").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn fail_build_rec_copy() {
        parse_and_compile_and_run("testFailBuildRecCopy").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn fail_build_rec_drop() {
        parse_and_compile_and_run("testFailBuildRecDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn fail_build_rec_persist() {
        parse_and_compile_and_run("testFailBuildRecPersist").unwrap();
    }

    #[test]
    #[should_panic(expected="Transaction was rolled back")]
    fn fail_eq() {
        parse_and_compile_and_run("testFailEq").unwrap();
    }

}



