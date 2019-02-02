
#[cfg(test)]
mod tests {
    use sanskrit_common::errors::*;
    use sanskrit_test_script_compiler::model::Id;
    use sanskrit_deploy::deploy_module;
    use sanskrit_compile::compile_module;
    use test::Bencher;
    use sanskrit_memory_store::BTreeMapStore;
    use sanskrit_test_script_compiler::transaction::Compiler;
    use sanskrit_runtime::execute;
    use std::env::current_dir;
    use sanskrit_common::store::StorageClass;
    use sanskrit_common::arena::Heap;
    use sanskrit_runtime::CONFIG;

    fn parse_and_compile_and_run(id:&str) -> Result<()>{
        let id = Id(id.into());
        let folder = current_dir().unwrap().join("transactions");
        let mut comp = Compiler::new(&folder);
        comp.parse_transactions(id.clone());
        comp.compile_transactions()?;
        let comp_res = comp.extract_results();
        let s = BTreeMapStore::new();
        for (_,r) in comp_res.modules {
            let res = deploy_module(&s, r)?;
            compile_module(&s, res)?;
        }
        let mut heap = Heap::new(CONFIG.calc_heap_size(2),2.0);
        for txt in comp_res.txts {
           execute(&s, &txt, 0, &heap)?;
            heap = heap.reuse();
        }
        Ok(())
    }

    struct HeapContainer {
        pub heap:Option<Heap>
    }

    impl HeapContainer {
        fn new(size:usize, convert:f64) -> Self {
            HeapContainer{
                heap: Some(Heap::new(size,convert))
            }
        }

        fn reuse(&mut self) {
            let mut heap_r = None;
            std::mem::swap(&mut self.heap, &mut heap_r);
            heap_r = Some(heap_r.unwrap().reuse());
            std::mem::swap(&mut self.heap, &mut heap_r);
        }
    }


    fn parse_and_compile_and_run_bench(id:&str,b: &mut Bencher) -> Result<()>{
        let id = Id(id.into());
        let folder = current_dir().unwrap().join("transactions");
        let mut comp = Compiler::new(&folder);
        comp.parse_transactions(id.clone());
        comp.compile_transactions()?;
        let comp_res = comp.extract_results();
        let s = BTreeMapStore::new();
        for (_, r) in &comp_res.modules {
            let res = deploy_module(&s, r.clone()).unwrap();
            compile_module(&s, res).unwrap();
        }
        let mut heap = HeapContainer::new(CONFIG.calc_heap_size(2), 2.0);

        b.iter(move || {
            for txt in &comp_res.txts {
                match heap.heap {
                    None => unreachable!(),
                    Some(ref heap) => execute(&s, txt, 0, heap).unwrap(),
                };
                heap.reuse();
            }
            s.clear_section(StorageClass::Elem);
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
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn double_borrow_fail() {
        parse_and_compile_and_run("testFailBorrowBorrowed").unwrap();
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn double_borrow_fail2() {
        parse_and_compile_and_run("testFailBorrowBorrowed2").unwrap();
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn double_borrow_fail3() {
        parse_and_compile_and_run("testFailBorrowBorrowed3").unwrap();
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn double_borrow_moved_fail() {
        parse_and_compile_and_run("testFailBorrowMoved").unwrap();
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn double_borrow_moved_fail2() {
        parse_and_compile_and_run("testFailBorrowMoved2").unwrap();
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn double_borrow_moved_fail3() {
        parse_and_compile_and_run("testFailBorrowMoved3").unwrap();
    }

    #[test]
    #[should_panic(expected="borrow declaration mismatch")]
    fn double_borrow_unpck_fail() {
        parse_and_compile_and_run("testFailBorrowUnpack").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn no_unpack_cap() {
        parse_and_compile_and_run("testFailCapUnpack").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn no_inspect_cap() {
        parse_and_compile_and_run("testFailCapUnpack2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_borrowed() {
        parse_and_compile_and_run("testFailConsumeBorrowed").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_borrowed2() {
        parse_and_compile_and_run("testFailConsumeBorrowed2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_borrowed3() {
        parse_and_compile_and_run("testFailConsumeBorrowed3").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_moved() {
        parse_and_compile_and_run("testFailConsumeMoved").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_moved2() {
        parse_and_compile_and_run("testFailConsumeMoved2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_moved3() {
        parse_and_compile_and_run("testFailConsumeMoved3").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not access already moved slot")]
    fn copy_moved() {
        parse_and_compile_and_run("testFailCopy").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not access already moved slot")]
    fn copy_moved2() {
        parse_and_compile_and_run("testFailCopyConsumed").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn create_fail() {
        parse_and_compile_and_run("testFailCreate").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn create_fail2() {
        parse_and_compile_and_run("testFailCreate2").unwrap();
    }

    #[test]
    #[should_panic(expected="Can only borrow if their is an argument")]
    fn create_fail3() {
        parse_and_compile_and_run("testFailCreate3").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn create_fail4() {
        parse_and_compile_and_run("testFailCreate4").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn create_fail5() {
        parse_and_compile_and_run("testFailCreate5").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn create_fail6() {
        parse_and_compile_and_run("testFailCreate6").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn create_fail7() {
        parse_and_compile_and_run("testFailCreate7").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not access already moved slot")]
    fn same_arg() {
        parse_and_compile_and_run("testFailDualArg").unwrap();
    }

    #[test]
    #[should_panic(expected="Borrowed value required")]
    fn free_unborrowed() {
        parse_and_compile_and_run("testFailFree").unwrap();
    }

    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn free_undropable() {
        parse_and_compile_and_run("testFailFree2").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn free_undropable2() {
        parse_and_compile_and_run("testFailFree3").unwrap();
    }

    #[test]
    #[should_panic(expected="Number of applied type parameters must match the number of declared generics")]
    fn wrong_num_fun_gens() {
        parse_and_compile_and_run("testFailFunApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Number of applied type parameters must match the number of declared generics")]
    fn wrong_num_fun_gens2() {
        parse_and_compile_and_run("testFailFunApply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Number of params is wrong")]
    fn wrong_num_fun_param() {
        parse_and_compile_and_run("testFailFunCall").unwrap();
    }

    #[test]
    #[should_panic(expected="Number of params is wrong")]
    fn wrong_num_fun_param2() {
        parse_and_compile_and_run("testFailFunCall2").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn wrong_fun_arg_type() {
        parse_and_compile_and_run("testFailFunCall3").unwrap();
    }

    #[test]
    #[should_panic(expected="An apply must have all capabilities required by the generic")]
    fn wrong_type_arg_contraint() {
        parse_and_compile_and_run("testFailFunConstraintApply").unwrap();
    }

    #[test]
    #[should_panic(expected="An apply must have all capabilities required by the generic")]
    fn wrong_type_arg_contraint2() {
        parse_and_compile_and_run("testFailTypeConstraintApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Physical generics can not be instantiated by phantom generics")]
    fn type_apply() {
        parse_and_compile_and_run("testFailTypeApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Physical generics can not be instantiated by phantom generics")]
    fn type_apply2() {
        parse_and_compile_and_run("testFailTypeApply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Physical generics can not be instantiated by phantom generics")]
    fn phantom_apply() {
        parse_and_compile_and_run("testFailFunPhantomApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Physical generics can not be instantiated by phantom generics")]
    fn phantom_apply2() {
        parse_and_compile_and_run("testFailFunPhantomApply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Physical generics can not be instantiated by phantom generics")]
    fn phantom_apply3() {
        parse_and_compile_and_run("testFailTypePhantomApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Physical generics can not be instantiated by phantom generics")]
    fn phantom_apply4() {
        parse_and_compile_and_run("testFailTypePhantomApply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn fail_pack_type() {
        parse_and_compile_and_run("testFailPackType").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn fail_pack_type2() {
        parse_and_compile_and_run("testFailPackType2").unwrap();
    }


    #[test]
    #[should_panic(expected="Number of supplied fields mismatch")]
    fn fail_pack_param() {
        parse_and_compile_and_run("testFailTypeParam").unwrap();
    }

    #[test]
    #[should_panic(expected="Number of supplied fields mismatch")]
    fn fail_pack_param2() {
        parse_and_compile_and_run("testFailTypeParam2").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested item not present")]
    fn fail_priv_call() {
        parse_and_compile_and_run("testFailPrivateCall").unwrap();
    }

    #[test]
    #[should_panic(expected="Function is not visible")]
    fn fail_prot_call() {
        parse_and_compile_and_run("testFailProtectedCall").unwrap();
    }

    #[test]
    #[should_panic(expected="Function is not visible")]
    fn fail_prot_call2() {
        parse_and_compile_and_run("testFailProtectedCall2").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn unpack_fail() {
        parse_and_compile_and_run("testFailUnpackType").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn unpack_fail2() {
        parse_and_compile_and_run("testFailUnpackType2").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong constructor specified")]
    fn unpack_fail3() {
        parse_and_compile_and_run("testFailUnpackType3").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong constructor specified")]
    fn unpack_fail4() {
        parse_and_compile_and_run("testFailUnpackType4").unwrap();
    }

    #[test]
    #[should_panic(expected="borrow declaration mismatch")]
    fn unpack_fail5() {
        parse_and_compile_and_run("testFailUnpackType5").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong number or type of generic arguments")]
    fn lit_missapply() {
        parse_and_compile_and_run("testFailLitMissapply").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong number or type of generic arguments")]
    fn lit_missapply2() {
        parse_and_compile_and_run("testFailLitMissapply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong number or type of generic arguments")]
    fn lit_missapply3() {
        parse_and_compile_and_run("testFailLitMissapply3").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong number or type of generic arguments")]
    fn lit_missapply4() {
        parse_and_compile_and_run("testFailLitMissapply4").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong number or type of generic arguments")]
    fn lit_missapply5() {
        parse_and_compile_and_run("testFailLitMissapply5").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong number or type of generic arguments")]
    fn lit_missapply6() {
        parse_and_compile_and_run("testFailLitMissapply6").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong number or type of generic arguments")]
    fn lit_missapply7() {
        parse_and_compile_and_run("testFailLitMissapply7").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn key_type_fail() {
        parse_and_compile_and_run("testFailKeyType").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn key_type_fail2() {
        parse_and_compile_and_run("testFailKeyType2").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn store_cap_fail() {
        parse_and_compile_and_run("testFailNoIndex").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn store_cap_fail2() {
        parse_and_compile_and_run("testFailNoPersist").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested item not present")]
    fn load_missing() {
        parse_and_compile_and_run("testFailLoad").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested item not present")]
    fn load_missing2() {
        parse_and_compile_and_run("testFailLoad2").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested item not present")]
    fn load_missing3() {
        parse_and_compile_and_run("testFailLoad3").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested item not present")]
    fn load_missing4() {
        parse_and_compile_and_run("testFailLoad4").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested item not present")]
    fn load_missing5() {
        parse_and_compile_and_run("testFailLoad5").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested item not present")]
    fn load_missing6() {
        parse_and_compile_and_run("testFailLoad6").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested item not present")]
    fn load_missing7() {
        parse_and_compile_and_run("testFailLoad7").unwrap();
    }

    #[test]
    #[should_panic(expected="Element already in store")]
    fn restore_loaded() {
        parse_and_compile_and_run("testFailLoad8").unwrap();
    }

    #[test]
    #[should_panic(expected="Element already in store")]
    fn restore_loaded2() {
        parse_and_compile_and_run("testFailLoad9").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn sig_load_mismatch() {
        parse_and_compile_and_run("testFailSigLoad").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn sig_load_mismatch2() {
        parse_and_compile_and_run("testFailSigLoad2").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn sig_load_mismatch3() {
        parse_and_compile_and_run("testFailSigLoad3").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn sig_load_mismatch4() {
        parse_and_compile_and_run("testFailSigLoad4").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn sig_load_mismatch5() {
        parse_and_compile_and_run("testFailSigLoad5").unwrap();
    }

    #[test]
    #[should_panic(expected="Element already in store")]
    fn store_full() {
        parse_and_compile_and_run("testFailStore").unwrap();
    }

    #[test]
    #[should_panic(expected="Element already in store")]
    fn store_full2() {
        parse_and_compile_and_run("testFailStore2").unwrap();
    }

    #[test]
    #[should_panic(expected="An apply must have all capabilities required by the generic")]
    fn fail_build_rec_copy() {
        parse_and_compile_and_run("testFailBuildRecCopy").unwrap();
    }

    #[test]
    #[should_panic(expected="An apply must have all capabilities required by the generic")]
    fn fail_build_rec_drop() {
        parse_and_compile_and_run("testFailBuildRecDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="An apply must have all capabilities required by the generic")]
    fn fail_build_rec_persist() {
        parse_and_compile_and_run("testFailBuildRecPersist").unwrap();
    }

    #[test]
    #[should_panic(expected="Error was produced")]
    fn fail_eq() {
        parse_and_compile_and_run("testFailEq").unwrap();
    }

    #[test]
    #[should_panic(expected="Error was produced")]
    fn fail_eq2() {
        parse_and_compile_and_run("testFailEq2").unwrap();
    }

    #[test]
    #[should_panic(expected="Error was produced")]
    fn fail_eq3() {
        parse_and_compile_and_run("testFailEq3").unwrap();
    }


    #[test]
    fn succ_borrow_release() {
        parse_and_compile_and_run("testSuccBorrowRelease").unwrap();
    }


    #[test]
    fn succ_move_release() {
        parse_and_compile_and_run("testSuccMoveRelease").unwrap();
    }

    #[test]
    fn succ_build_rec() {
        parse_and_compile_and_run("testSuccBuildRec").unwrap();
    }

    #[bench]
    fn succ_base_ops_bench(b: &mut Bencher) {
        parse_and_compile_and_run_bench("testSuccBaseOps",b).unwrap();
    }

    #[test]
    fn succ_base_ops() {
        parse_and_compile_and_run("testSuccBaseOps").unwrap();
    }

    #[test]
    fn succ_phantom_ops() {
        parse_and_compile_and_run("testSuccPhantomApply").unwrap();
    }

    #[test]
    fn succ_storage_drop() {
        parse_and_compile_and_run("testSuccStorageWithDrop").unwrap();
    }

    #[test]
    fn succ_storage_drop2() {
        parse_and_compile_and_run("testSuccStorageWithDrop2").unwrap();
    }

    #[test]
    fn succ_storage_copy() {
        parse_and_compile_and_run("testSuccStorageWithCopy").unwrap();
    }

    #[test]
    fn succ_storage_copy2() {
        parse_and_compile_and_run("testSuccStorageWithCopy2").unwrap();
    }

    #[bench]
    fn succ_storage_copy_drop_bench(b: &mut Bencher) {
        parse_and_compile_and_run_bench("testSuccStorageWithDropAndCopy",b).unwrap();
    }


    #[test]
    fn succ_storage_copy_drop() {
        parse_and_compile_and_run("testSuccStorageWithDropAndCopy").unwrap();
    }

    #[test]
    fn succ_storage_copy_drop2() {
        parse_and_compile_and_run("testSuccStorageWithDropAndCopy2").unwrap();
    }

    #[bench]
    fn succ_storage_priv_bench(b: &mut Bencher) {
        parse_and_compile_and_run_bench("testSuccPrivStorage",b).unwrap();
    }

    #[test]
    fn succ_storage_priv() {
        parse_and_compile_and_run("testSuccPrivStorage").unwrap();
    }

    #[test]
    fn succ_protected() {
        parse_and_compile_and_run("testSuccProtectedCall").unwrap();
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

    #[test]
    fn succ_cast_ops() {
        parse_and_compile_and_run("testSuccCastOps").unwrap();
    }

    #[bench]
    fn succ_compare_hash_ops_bench(b: &mut Bencher) {
        parse_and_compile_and_run_bench("testSuccCompareHashOps",b).unwrap();
    }

    #[test]
    fn succ_compare_hash_ops() {
        parse_and_compile_and_run("testSuccCompareHashOps").unwrap();
    }

    #[bench]
    fn succ_index_ops_bench(b: &mut Bencher) {
        parse_and_compile_and_run_bench("testSuccIndexOps",b).unwrap();
    }

    #[test]
    fn succ_index_ops() {
        parse_and_compile_and_run("testSuccIndexOps").unwrap();
    }

    #[test]
    fn succ_error_ops() {
        parse_and_compile_and_run("testSuccErrorOps").unwrap();
    }
}



