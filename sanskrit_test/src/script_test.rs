
#[cfg(test)]
mod tests {
    use sanskrit_common::errors::*;
    use sanskrit_common::encoding::*;
    use sanskrit_test_script_compiler::model::Id;
    use sanskrit_test_script_compiler::script::Compiler;
    use std::fs::File;
    use sanskrit_core::model::Module;
    use sanskrit_deploy::deploy_module;
    use sanskrit_compile::compile_module;
    use std::io::Write;
    use test::Bencher;
    use sanskrit_memory_store::BTreeMapStore;
    use std::env::current_dir;

    fn parse_and_compile(id:&str) -> Result<()>{
        let id = Id(id.into());
        let folder = current_dir().unwrap().join("scripts");
        let out_folder = folder.join("out");
        let mut comp = Compiler::new(&folder);
        comp.parse_module_tree(id.clone());
        comp.compile_module_tree().unwrap();
        let s = BTreeMapStore::new();
        for (c_id,r) in comp.get_results() {
            if c_id == id {
                let fb_path = out_folder.join(&id.0.to_lowercase()).with_extension("bin");
                let mut bin = File::create(fb_path).expect("file not found");
                bin.write_all(&r).unwrap();
                let fa_path = out_folder.join(&id.0.to_lowercase()).with_extension("asm");
                let mut asm = File::create(fa_path).expect("file not found");
                asm.write_all(format!("{:?}",Parser::parse_fully::<Module,NoCustomAlloc>(&r,&NoCustomAlloc()).unwrap()).as_bytes()).unwrap();
            }
            let res = deploy_module(&s, r)?;
            compile_module(&s, res)?;
        }
        Ok(())
    }

    fn parse_and_compile_deploy_bench(id:&str,b: &mut Bencher) -> Result<()>{
        let id = Id(id.into());
        let folder = current_dir().unwrap().join("scripts");
        let mut comp = Compiler::new(&folder);
        comp.parse_module_tree(id.clone());
        comp.compile_module_tree().unwrap();
        b.iter(|| {
            let s = BTreeMapStore::new();
            for (_,r) in comp.get_results() {
                let res = deploy_module(&s, r).unwrap();
                compile_module(&s, res).unwrap();
            }
        });
        Ok(())
    }

    #[bench]
    fn build_diff_adts_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench("testSucAdt",b).unwrap();
    }

    #[test]
    fn build_diff_adts() {
        parse_and_compile("testSucAdt").unwrap();
    }

    #[test]
    fn build_diff_open_adts() {
        parse_and_compile("testSucOpenAdt").unwrap();
    }

    #[bench]
    fn build_diff_adts_import_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench("testSucAdtImport",b).unwrap();
    }

    #[test]
    fn build_diff_adts_import() {
        parse_and_compile("testSucAdtImport").unwrap();
    }

    #[bench]
    fn build_create_open_adts_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench("testSucCreateOpenAdts",b).unwrap();
    }

    #[test]
    fn build_create_open_adts() {
        parse_and_compile("testSucCreateOpenAdts").unwrap();
    }

    #[bench]
    fn build_create_closed_adts_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench("testSucCreateClosedAdts",b).unwrap();
    }

    #[test]
    fn build_create_closed_adts() {
        parse_and_compile("testSucCreateClosedAdts").unwrap();
    }

    #[bench]
    fn build_borrow_create_open_adts_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench("testSucBorrowCreateOpenAdts",b).unwrap();
    }

    #[test]
    fn build_borrow_create_open_adts() {
        parse_and_compile("testSucBorrowCreateOpenAdts").unwrap();
    }

    #[bench]
    fn define_and_call_functions_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench("testSucFun",b).unwrap();
    }

    #[test]
    fn define_and_call_functions() {
        parse_and_compile("testSucFun").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn unsat_adts_embed() {
        parse_and_compile("testFailAdtWrongCapsEmbed").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn unsat_adts_embed2() {
        parse_and_compile("testFailAdtWrongCapsEmbed2").unwrap();
    }

    #[test]
    #[should_panic(expected="Type does not full fill capability requirements")]
    fn rec_unsat_adts_copy() {
        parse_and_compile("testFailAdtWrongRecCapsCopy").unwrap();
    }

    #[test]
    #[should_panic(expected="Type does not full fill capability requirements")]
    fn rec_unsat_adts_drop() {
        parse_and_compile("testFailAdtWrongRecCapsDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Type does not full fill capability requirements")]
    fn rec_unsat_adts_persist() {
        parse_and_compile("testFailAdtWrongRecCapsPersist").unwrap();
    }

    #[test]
    #[should_panic(expected="An apply must have all capabilities required by the generic")]
    fn rec_unsat_adts_copy_infer() {
        parse_and_compile("testFailAdtWrongRecCapsCopyInfer").unwrap();
    }

    #[test]
    #[should_panic(expected="An apply must have all capabilities required by the generic")]
    fn rec_unsat_adts_drop_infer() {
        parse_and_compile("testFailAdtWrongRecCapsDropInfer").unwrap();
    }

    #[test]
    #[should_panic(expected="An apply must have all capabilities required by the generic")]
    fn rec_unsat_adts_persist_infer() {
        parse_and_compile("testFailAdtWrongRecCapsPersistInfer").unwrap();
    }

    #[test]
    #[should_panic(expected="Type does not full fill capability requirements")]
    fn rec_unsat_adts_copy_remote() {
        parse_and_compile("testFailAdtWrongRecCapsCopyRemote").unwrap();
    }

    #[test]
    #[should_panic(expected="Type does not full fill capability requirements")]
    fn rec_unsat_adts_drop_remote() {
        parse_and_compile("testFailAdtWrongRecCapsDropRemote").unwrap();
    }

    #[test]
    #[should_panic(expected="Type does not full fill capability requirements")]
    fn rec_unsat_adts_persist_remote() {
        parse_and_compile("testFailAdtWrongRecCapsPersistRemote").unwrap();
    }

    #[test]
    #[should_panic(expected="Type does not full fill capability requirements")]
    fn identity_unsat_adts() {
        parse_and_compile("testFailAdtWrongCapsIdentity").unwrap();
    }

    #[test]
    #[should_panic(expected="Illegal capability set")]
    fn consume_implicatons() {
        parse_and_compile("testFailInspectImplied").unwrap();
    }

    #[test]
    #[should_panic(expected="Number of applied type parameters must match the number of declared generics")]
    fn wrong_num_args_adt() {
        parse_and_compile("testFailAdtApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Number of applied type parameters must match the number of declared generics")]
    fn wrong_num_args_adt2() {
        parse_and_compile("testFailAdtApply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Number of applied type parameters must match the number of declared generics")]
    fn wrong_num_args_fun() {
        parse_and_compile("testFailFunApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Number of applied type parameters must match the number of declared generics")]
    fn wrong_num_args_fun2() {
        parse_and_compile("testFailFunApply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Number of applied type parameters must match the number of declared generics")]
    fn wrong_num_args_prim() {
        parse_and_compile("testFailPrimApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Physical generics can not be instantiated by phantom generics")]
    fn wrong_phantom_args_fun() {
        parse_and_compile("testFailFunPhantomApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Physical generics can not be instantiated by phantom generics")]
    fn wrong_phantom_args_adt() {
        parse_and_compile("testFailAdtPhantomApply").unwrap();
    }

    #[test]
    #[should_panic(expected="An apply must have all capabilities required by the generic")]
    fn wrong_constraint_args_fun() {
        parse_and_compile("testFailFunConstraintApply").unwrap();
    }

    #[test]
    #[should_panic(expected="An apply must have all capabilities required by the generic")]
    fn wrong_constraint_args_adt() {
        parse_and_compile("testFailAdtConstraintApply").unwrap();
    }

    #[test]
    #[should_panic(expected="An apply must have all capabilities required by the generic")]
    fn wrong_constraint_args_adt2() {
        parse_and_compile("testFailAdtConstraintApply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Phantom Generics can not be used for values")]
    fn phantom_adt_field() {
        parse_and_compile("testFailAdtPhantomUse").unwrap();
    }

    #[test]
    #[should_panic(expected="Phantom Generics can not be used for values")]
    fn phantom_fun_param() {
        parse_and_compile("testFailFunPhantomParam").unwrap();
    }

    #[test]
    #[should_panic(expected="Phantom Generics can not be used for values")]
    fn phantom_fun_return() {
        parse_and_compile("testFailFunPhantomReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="Function is not visible")]
    fn protected_fun_call_fail1() {
        parse_and_compile("testFailProtectedCall1").unwrap();
    }

    #[test]
    #[should_panic(expected="Function is not visible")]
    fn protected_fun_call_fail2() {
        parse_and_compile("testFailProtectedCall2").unwrap();
    }

    #[test]
    #[should_panic(expected="Function is not visible")]
    fn protected_fun_call_fail3() {
        parse_and_compile("testFailProtectedCall3").unwrap();
    }

    #[test]
    #[should_panic(expected="Function is not visible")]
    fn protected_fun_call_fail4() {
        parse_and_compile("testFailProtectedCall4").unwrap();
    }

    #[test]
    #[should_panic(expected="Function is not visible")]
    fn protected_fun_call_fail5() {
        parse_and_compile("testFailProtectedCall5").unwrap();
    }

    #[test]
    #[should_panic(expected="Function is not visible")]
    fn private_fun_call_fail() {
        parse_and_compile("testFailPrivateCall").unwrap();
    }

    #[test]
    #[should_panic(expected="Risk is not declared")]
    fn throw_fun_fail() {
        parse_and_compile("testFailThrow").unwrap();
    }

    #[test]
    #[should_panic(expected="Risk is not declared")]
    fn throw_fun_fail2() {
        parse_and_compile("testFailThrow2").unwrap();
    }

    #[test]
    #[should_panic(expected="Risk is not declared")]
    fn throw_fun_fail3() {
        parse_and_compile("testFailThrow3").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn copy_adt_fail() {
        parse_and_compile("testFailCopy").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn copy_adt_fail2() {
        parse_and_compile("testFailCopy2").unwrap();
    }

    #[test]
    #[should_panic(expected="Borrowed value required")]
    fn free_fail() {
        parse_and_compile("testFailFree").unwrap();
    }

    #[test]
    #[should_panic(expected="Borrowed value required")]
    fn free_fail2() {
        parse_and_compile("testFailFree2").unwrap();
    }

    #[test]
    #[should_panic(expected="An apply must have all capabilities required by the generic")]
    fn persist_fail() {
        parse_and_compile("testFailPersist").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn drop_fail() {
        parse_and_compile("testFailDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn drop_fail2() {
        parse_and_compile("testFailDrop2").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn drop_fail3() {
        parse_and_compile("testFailDrop3").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn drop_fail4() {
        parse_and_compile("testFailDrop4").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn unpack_type_fail() {
        parse_and_compile("testFailUnpackType").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn borrow_unpack_type_fail() {
        parse_and_compile("testFailUnpackType2").unwrap();
    }

    #[test]
    #[should_panic(expected="Type has no constructors")]
    fn lit_unpack_fail() {
        parse_and_compile("testFailUnpackType3").unwrap();
    }

    #[test]
    #[should_panic(expected="Type has no constructors")]
    fn lit_borrow_unpack_fail() {
        parse_and_compile("testFailUnpackType4").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested Constructor unavailable")]
    fn empty_unpack_fail() {
        parse_and_compile("testFailUnpackType5").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested Constructor unavailable")]
    fn empty_borrow_unpack_fail() {
        parse_and_compile("testFailUnpackType6").unwrap();
    }

    #[test]
    #[should_panic(expected="opcode not defined for the requested type")]
    fn union_unpack_fail() {
        parse_and_compile("testFailUnpackType7").unwrap();
    }

    #[test]
    #[should_panic(expected="opcode not defined for the requested type")]
    fn union_borrow_unpack_fail() {
        parse_and_compile("testFailUnpackType8").unwrap();
    }

    #[test]
    #[should_panic(expected="borrow declaration mismatch")]
    fn borrow_unpack_fail() {
        parse_and_compile("testFailBorrowUnpack").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn unpack_cap_fail() {
        parse_and_compile("testFailCapUnpack").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn unpack_cap_fail2() {
        parse_and_compile("testFailCapUnpack2").unwrap();
    }

    #[test]
    #[should_panic(expected="opcode not defined for the requested type")]
    fn switch_type_fail() {
        parse_and_compile("testFailSwitchType").unwrap();
    }

    #[test]
    #[should_panic(expected="opcode not defined for the requested type")]
    fn switch_type_fail2() {
        parse_and_compile("testFailSwitchType2").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested Constructor unavailable")]
    fn switch_type_fail3() {
        parse_and_compile("testFailSwitchType3").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested Constructor unavailable")]
    fn switch_type_fail4() {
        parse_and_compile("testFailSwitchType4").unwrap();
    }

    #[test]
    #[should_panic(expected="Input not allowed to be borrowed")]
    fn borrow_switch_fail() {
        parse_and_compile("testFailBorrowSwitch").unwrap();
    }


    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn pack_caps_fail() {
        parse_and_compile("testFailCreate").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn pack_caps_fail2() {
        parse_and_compile("testFailCreate2").unwrap();
    }

    #[test]
    #[should_panic(expected="Required capability is missing")]
    fn pack_lit_fail() {
        parse_and_compile("testFailCreate3").unwrap();
    }


    #[test]
    #[should_panic(expected="Requested Constructor unavailable")]
    fn pack_no_ctr_fail() {
        parse_and_compile("testFailCreate4").unwrap();
    }

    #[test]
    #[should_panic(expected="Can only borrow if their is an argument")]
    fn pack_no_arg_fail() {
        parse_and_compile("testFailCreate5").unwrap();
    }

    #[test]
    #[should_panic(expected="Input not allowed to be borrowed")]
    fn pack_borrow_fail() {
        parse_and_compile("testFailCreate6").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn pack_type_fail() {
        parse_and_compile("testFailPackType").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn pack_type_fail2() {
        parse_and_compile("testFailPackType2").unwrap();
    }

    #[test]
    #[should_panic(expected="Risk is not declared")]
    fn risky_call_fail() {
        parse_and_compile("testFailRiskyCall").unwrap();
    }

    #[test]
    #[should_panic(expected="Risk is not declared")]
    fn risky_call_fail2() {
        parse_and_compile("testFailRiskyCall2").unwrap();
    }

    #[test]
    #[should_panic(expected="Number of params is wrong")]
    fn wrong_arg_call_fail() {
        parse_and_compile("testFailFunCall").unwrap();
    }

    #[test]
    #[should_panic(expected="Number of params is wrong")]
    fn wrong_arg_call_fail2() {
        parse_and_compile("testFailFunCall2").unwrap();
    }

    #[test]
    #[should_panic(expected="Type mismatch")]
    fn wrong_arg_type_fail() {
        parse_and_compile("testFailFunCall3").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_moved_fail() {
        parse_and_compile("testFailConsumeMoved").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_moved_fail2() {
        parse_and_compile("testFailConsumeMoved2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_moved_fail3() {
        parse_and_compile("testFailConsumeMoved3").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_borrowed_fail() {
        parse_and_compile("testFailConsumeBorrowed").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_borrowed_fail2() {
        parse_and_compile("testFailConsumeBorrowed2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn consume_borrowed_fail3() {
        parse_and_compile("testFailConsumeBorrowed3").unwrap();
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn borrow_borrowed_fail() {
        parse_and_compile("testFailBorrowBorrowed").unwrap();
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn borrow_borrowed_fail2() {
        parse_and_compile("testFailBorrowBorrowed2").unwrap();
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn borrow_borrowed_fail3() {
        parse_and_compile("testFailBorrowBorrowed3").unwrap();
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn borrow_moved_fail() {
        parse_and_compile("testFailBorrowMoved").unwrap();
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn borrow_moved_fail2() {
        parse_and_compile("testFailBorrowMoved2").unwrap();
    }

    #[test]
    #[should_panic(expected="Locking moved slot is forbidden")]
    fn borrow_moved_fail3() {
        parse_and_compile("testFailBorrowMoved3").unwrap();
    }

    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn free_unconsumed_fail() {
        parse_and_compile("testFailFreeUnconsumed").unwrap();
    }

    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn free_unconsumed_fail2() {
        parse_and_compile("testFailFreeUnconsumed2").unwrap();
    }

    #[test]
    #[should_panic(expected="Only consumed and not locked elem slots can be freed")]
    fn free_unconsumed_fail3() {
        parse_and_compile("testFailFreeUnconsumed3").unwrap();
    }

    #[test]
    #[should_panic(expected="A consumed slots can not be returned")]
    fn return_consumed_fail() {
        parse_and_compile("testFailReturnMoved").unwrap();
    }

    #[test]
    #[should_panic(expected="A consumed slots can not be returned")]
    fn return_consumed_fail2() {
        parse_and_compile("testFailReturnMoved2").unwrap();
    }

    #[test]
    #[should_panic(expected="A consumed slots can not be returned")]
    fn return_consumed_fail3() {
        parse_and_compile("testFailReturnMoved3").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not access already moved slot")]
    fn dual_arg_fail() {
        parse_and_compile("testFailDualArg").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not access already moved slot")]
    fn copy_consumed_fail() {
        parse_and_compile("testFailCopyConsumed").unwrap();
    }
    
    #[test]
    #[should_panic(expected="can not steal borrows")]
    fn steal_dropped_fail() {
        parse_and_compile("testFailStealDropped").unwrap();
    }

    #[test]
    #[should_panic(expected="can not steal borrows")]
    fn steal_dropped_fail2() {
        parse_and_compile("testFailStealDropped2").unwrap();
    }

    #[test]
    #[should_panic(expected="can not steal borrows")]
    fn steal_dropped_fail3() {
        parse_and_compile("testFailStealDropped3").unwrap();
    }

    #[test]
    #[should_panic(expected="can not steal borrows")]
    fn steal_later_fail() {
        parse_and_compile("testFailStealLater").unwrap();
    }

    #[test]
    #[should_panic(expected="can not steal borrows")]
    fn steal_later_fail2() {
        parse_and_compile("testFailStealLater2").unwrap();
    }

    #[test]
    #[should_panic(expected="can not steal borrows")]
    fn steal_later_fail3() {
        parse_and_compile("testFailStealLater3").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn fun_return_fail() {
        parse_and_compile("testFailFunReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn let_return_fail() {
        parse_and_compile("testFailLetReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn try_return_fail() {
        parse_and_compile("testFailTryReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn try_return_fail2() {
        parse_and_compile("testFailTryReturn2").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn switch_return_fail() {
        parse_and_compile("testFailSwitchReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn switch_return_fail2() {
        parse_and_compile("testFailSwitchReturn2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn fun_double_return_fail() {
        parse_and_compile("testFailFunDoubleReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn let_double_return_fail() {
        parse_and_compile("testFailLetDoubleReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn try_double_return_fail() {
        parse_and_compile("testFailTryDoubleReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn try_double_return_fail2() {
        parse_and_compile("testFailTryDoubleReturn2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn switch_double_return_fail() {
        parse_and_compile("testFailSwitchDoubleReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn switch_double_return_fail2() {
        parse_and_compile("testFailSwitchDoubleReturn2").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn fun_drop_fail() {
        parse_and_compile("testFailFunDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn let_drop_fail() {
        parse_and_compile("testFailLetDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn try_drop_fail() {
        parse_and_compile("testFailTryDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn try_drop_fail2() {
        parse_and_compile("testFailTryDrop2").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn switch_drop_fail() {
        parse_and_compile("testFailSwitchDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Can not handle element from outside of the active frame")]
    fn switch_drop_fail2() {
        parse_and_compile("testFailSwitchDrop2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn fun_double_drop_fail() {
        parse_and_compile("testFailFunDoubleDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn let_double_drop_fail() {
        parse_and_compile("testFailLetDoubleDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn try_double_drop_fail() {
        parse_and_compile("testFailTryDoubleDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn try_double_drop_fail2() {
        parse_and_compile("testFailTryDoubleDrop2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn switch_double_drop_fail() {
        parse_and_compile("testFailSwitchDoubleDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn switch_double_drop_fail2() {
        parse_and_compile("testFailSwitchDoubleDrop2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn fun_return_drop_fail() {
        parse_and_compile("testFailFunReturnDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn let_return_drop_fail() {
        parse_and_compile("testFailLetReturnDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn try_return_drop_fail() {
        parse_and_compile("testFailTryReturnDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn try_return_drop_fail2() {
        parse_and_compile("testFailTryReturnDrop2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn switch_return_drop_fail() {
        parse_and_compile("testFailSwitchReturnDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consuming moved slot is forbidden")]
    fn switch_return_drop_fail2() {
        parse_and_compile("testFailSwitchReturnDrop2").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_return_type_fail() {
        parse_and_compile("testFailBranchReturnType").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_return_type_fail2() {
        parse_and_compile("testFailBranchReturnType2").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_return_type_fail3() {
        parse_and_compile("testFailBranchReturnType3").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_return_type_fail4() {
        parse_and_compile("testFailBranchReturnType4").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_return_type_fail5() {
        parse_and_compile("testFailBranchReturnType5").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_return_type_fail6() {
        parse_and_compile("testFailBranchReturnType6").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_return_type_fail7() {
        parse_and_compile("testFailBranchReturnType7").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_return_type_fail8() {
        parse_and_compile("testFailBranchReturnType8").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_return_type_fail9() {
        parse_and_compile("testFailBranchReturnType9").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_return_type_fail10() {
        parse_and_compile("testFailBranchReturnType10").unwrap();
    }


    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_diff_consumes_fail() {
        parse_and_compile("testFailBranchDiffConsumes").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_diff_consumes_fail2() {
        parse_and_compile("testFailBranchDiffConsumes2").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_diff_consumes_fail3() {
        parse_and_compile("testFailBranchDiffConsumes3").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_diff_consumes_fail4() {
        parse_and_compile("testFailBranchDiffConsumes4").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_diff_consumes_fail5() {
        parse_and_compile("testFailBranchDiffConsumes5").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_diff_consumes_fail6() {
        parse_and_compile("testFailBranchDiffConsumes6").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_diff_consumes_fail7() {
        parse_and_compile("testFailBranchDiffConsumes7").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_diff_consumes_fail8() {
        parse_and_compile("testFailBranchDiffConsumes8").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_diff_consumes_fail9() {
        parse_and_compile("testFailBranchDiffConsumes9").unwrap();
    }

    #[test]
    #[should_panic(expected="branches must induce the same post state")]
    fn branch_diff_consumes_fail10() {
        parse_and_compile("testFailBranchDiffConsumes10").unwrap();
    }

    #[test]
    #[should_panic(expected= "Function signature mismatch")]
    fn param_consume_fail() {
        parse_and_compile("testFailFunParamConsume").unwrap();
    }

    #[test]
    #[should_panic(expected= "Function signature mismatch")]
    fn param_consume_fail2() {
        parse_and_compile("testFailFunParamConsume2").unwrap();
    }

    #[test]
    #[should_panic(expected="Function signature mismatch")]
    fn ret_borrow_fail() {
        parse_and_compile("testFailFunRetBorrow").unwrap();
    }

    #[test]
    #[should_panic(expected="Function signature mismatch")]
    fn ret_borrow_fail2() {
        parse_and_compile("testFailFunRetBorrow2").unwrap();
    }

    #[test]
    #[should_panic(expected="Function signature mismatch")]
    fn ret_borrow_fail3() {
        parse_and_compile("testFailFunRetBorrow3").unwrap();
    }

    #[test]
    #[should_panic(expected="Function signature mismatch")]
    fn ret_borrow_fail4() {
        parse_and_compile("testFailFunRetBorrow4").unwrap();
    }

    #[test]
    #[should_panic(expected="Function signature mismatch")]
    fn ret_borrow_fail5() {
        parse_and_compile("testFailFunRetBorrow5").unwrap();
    }

    #[bench]
    fn validate_succ_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench("testSucValidate",b).unwrap();
    }

    #[test]
    fn validate_succ(){
        parse_and_compile("testSucValidate").unwrap();
    }

    #[bench]
    fn check_succ_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench("testSucTypeCheck",b).unwrap();
    }

    #[test]
    fn check_succ(){
        parse_and_compile("testSucTypeCheck").unwrap();
    }

    #[bench]
    fn lin_succ_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench("testSucLinearity",b).unwrap();
    }

    #[test]
    fn lin_succ(){
        parse_and_compile("testSucLinearity").unwrap();
    }

    #[test]
    fn cap_succ(){
        parse_and_compile("testSucCaps").unwrap();
    }

}
