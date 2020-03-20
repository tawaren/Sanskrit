
#[cfg(test)]
mod tests {
    use sanskrit_common::errors::*;
    use sanskrit_common::encoding::*;
    use sanskrit_test_script_compiler::model::Id;
    use sanskrit_test_script_compiler::script::Compiler;
    use std::fs::File;
    use sanskrit_core::model::Module;
    use sanskrit_deploy::deploy_module;
    use std::io::Write;
    use test::Bencher;
    use sanskrit_memory_store::BTreeMapStore;
    use std::env::current_dir;
    use sanskrit_core::accounting::Accounting;
    use std::cell::Cell;

    fn max_accounting() -> Accounting {
        Accounting {
            load_byte_budget: Cell::new(usize::max_value()),
            store_byte_budget: Cell::new(usize::max_value()),
            process_byte_budget: Cell::new(usize::max_value()),
            stack_elem_budget: Cell::new(usize::max_value()),
            nesting_limit: 10,
            input_limit: 1000000
        }
    }

    fn parse_and_deploy(id:&str) -> Result<()>{
        parse_and_deploy_template(id,true)
    }
    fn parse_and_deploy_plain(id:&str) -> Result<()>{
        parse_and_deploy_template(id,false)
    }
    fn parse_and_deploy_template(id:&str, include_system:bool) -> Result<()>{
        let id = Id(id.to_lowercase().into());
        let folder = current_dir().unwrap().join("scripts");
        let out_folder = folder.join("out");
        let mut comp = Compiler::new(&folder);
        let system_level = if !include_system {
            0
        } else {
            comp.parse_module_tree(Id("system".into()), 0)
        };
        comp.parse_module_tree(id.clone(), system_level+1);
        comp.compile_module_tree().unwrap();
        let s = BTreeMapStore::new();
        let accounting = max_accounting();
        let res = comp.get_module_results();
        for (c_id,r) in comp.get_module_results() {
            if c_id == id {
                let fb_path = out_folder.join(&id.0.to_lowercase()).with_extension("bin");
                let mut bin = File::create(fb_path).expect("file not found");
                bin.write_all(&r).unwrap();
                let fa_path = out_folder.join(&id.0.to_lowercase()).with_extension("asm");
                let mut asm = File::create(fa_path).expect("file not found");
                asm.write_all(format!("{:?}",Parser::parse_fully::<Module,NoCustomAlloc>(&r,usize::max_value(), &NoCustomAlloc()).unwrap()).as_bytes()).unwrap();
            }
            println!("Deploying module {:?} of {} Bytes", c_id, r.len());
            let res = deploy_module(&s, &accounting,r, true)?;

            if c_id == id {
                let fh_path = out_folder.join(&id.0.to_lowercase()).with_extension("hash");
                let mut hs = File::create(fh_path).expect("file not found");
                hs.write_all(format!("{:?}",res).as_bytes()).unwrap();
            }
        }
        Ok(())
    }

    fn parse_and_deploy_bench(id:&str, b: &mut Bencher) -> Result<()>{
        let id = Id(id.into());
        let folder = current_dir().unwrap().join("scripts");
        let mut comp = Compiler::new(&folder);
        let system_level = comp.parse_module_tree(Id("system".into()), 0);
        comp.parse_module_tree(id.clone(), system_level+1);
        comp.compile_module_tree().unwrap();

        let res = comp.get_module_results().into_iter().map(|(_,r)|r).collect::<Vec<_>>();
        b.iter(|| {
            let s = BTreeMapStore::new();
            let accounting = max_accounting();
            for r in res.clone() {
                let res = deploy_module(&s, &accounting, r, true).unwrap();
            }
        });
        Ok(())
    }

    #[test]
    fn build_system() {
        parse_and_deploy_plain("system").unwrap();
    }

    #[test]
    fn build_alt() {
        parse_and_deploy_plain("alt").unwrap();
    }

    #[test]
    fn build_data() {
        parse_and_deploy_plain("data").unwrap();
    }

    #[test]
    fn build_bool() {
        parse_and_deploy_plain("bool").unwrap();
    }

    #[test]
    fn build_ids() {
        parse_and_deploy("ids").unwrap();
    }

    #[test]
    fn build_int_i8() {
        parse_and_deploy_plain("intI8").unwrap();
    }

    #[test]
    fn build_int_i16() {
        parse_and_deploy_plain("intI16").unwrap();
    }

    #[test]
    fn build_int_i32() {
        parse_and_deploy_plain("intI32").unwrap();
    }

    #[test]
    fn build_int_i64() {
        parse_and_deploy_plain("intI64").unwrap();
    }

    #[test]
    fn build_int_i128() {
        parse_and_deploy_plain("intI128").unwrap();
    }

    #[test]
    fn build_int_u8() {
        parse_and_deploy_plain("intU8").unwrap();
    }

    #[test]
    fn build_int_u16() {
        parse_and_deploy_plain("intU16").unwrap();
    }

    #[test]
    fn build_int_u32() {
        parse_and_deploy_plain("intU32").unwrap();
    }

    #[test]
    fn build_int_u64() {
        parse_and_deploy_plain("intU64").unwrap();
    }

    #[test]
    fn build_int_u128() {
        parse_and_deploy_plain("intU128").unwrap();
    }

    #[bench]
    fn build_diff_adts_bench_deploy(b: &mut Bencher) {
        parse_and_deploy_bench("testSucAdt", b).unwrap();
    }

    #[bench]
    fn build_system_bench(b: &mut Bencher) {
        parse_and_deploy_bench("system", b).unwrap();
    }

    #[test]
    fn build_diff_adts() {
        parse_and_deploy("testSucAdt").unwrap();
    }

    #[test]
    fn build_diff_open_adts() {
        parse_and_deploy("testSucOpenAdt").unwrap();
    }

    #[bench]
    fn build_diff_adts_import_bench_deploy(b: &mut Bencher) {
        parse_and_deploy_bench("testSucAdtImport", b).unwrap();
    }

    #[test]
    fn build_diff_adts_import() {
        parse_and_deploy("testSucAdtImport").unwrap();
    }

    #[bench]
    fn build_create_open_adts_bench(b: &mut Bencher) {
        parse_and_deploy_bench("testSucCreateOpenAdts", b).unwrap();
    }

    #[test]
    fn build_create_open_adts() {
        parse_and_deploy("testSucCreateOpenAdts").unwrap();
    }

    #[bench]
    fn build_create_closed_adts_bench(b: &mut Bencher) {
        parse_and_deploy_bench("testSucCreateClosedAdts", b).unwrap();
    }

    #[test]
    fn build_create_closed_adts() {
        parse_and_deploy("testSucCreateClosedAdts").unwrap();
    }

    #[bench]
    fn define_and_call_functions_bench(b: &mut Bencher) {
        parse_and_deploy_bench("testSucFun", b).unwrap();
    }

    #[test]
    fn define_and_call_functions() {
        parse_and_deploy("testSucFun").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn rec_unsat_adts_copy() {
        parse_and_deploy("testFailAdtWrongRecCapsCopy").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn rec_unsat_adts_drop() {
        parse_and_deploy("testFailAdtWrongRecCapsDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn rec_unsat_adts_persist() {
        parse_and_deploy("testFailAdtWrongRecCapsPersist").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn rec_unsat_adts_copy_infer() {
        parse_and_deploy("testFailAdtWrongRecCapsCopyInfer").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn rec_unsat_adts_drop_infer() {
        parse_and_deploy("testFailAdtWrongRecCapsDropInfer").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn rec_unsat_adts_persist_infer() {
        parse_and_deploy("testFailAdtWrongRecCapsPersistInfer").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn rec_unsat_adts_copy_remote() {
        parse_and_deploy("testFailAdtWrongRecCapsCopyRemote").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn rec_unsat_adts_drop_remote() {
        parse_and_deploy("testFailAdtWrongRecCapsDropRemote").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn rec_unsat_adts_persist_remote() {
        parse_and_deploy("testFailAdtWrongRecCapsPersistRemote").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn wrong_num_args_adt() {
        parse_and_deploy("testFailAdtApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn wrong_num_args_adt2() {
        parse_and_deploy("testFailAdtApply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn wrong_num_args_fun() {
        parse_and_deploy("testFailFunApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn wrong_num_args_fun2() {
        parse_and_deploy("testFailFunApply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Applied types mismatch required generics")]
    fn wrong_num_args_prim() {
        parse_and_deploy("testFailPrimApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Phantom types can not be used as to apply non phantom generics")]
    fn wrong_phantom_args_fun() {
        parse_and_deploy("testFailFunPhantomApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Phantom types can not be used as to apply non phantom generics")]
    fn wrong_phantom_args_adt() {
        parse_and_deploy("testFailAdtPhantomApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn wrong_constraint_args_fun() {
        parse_and_deploy("testFailFunConstraintApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn wrong_constraint_args_adt() {
        parse_and_deploy("testFailAdtConstraintApply").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn wrong_constraint_args_adt2() {
        parse_and_deploy("testFailAdtConstraintApply2").unwrap();
    }

    #[test]
    #[should_panic(expected="Phantom types can not be used as constructor fields")]
    fn phantom_adt_field() {
        parse_and_deploy("testFailAdtPhantomUse").unwrap();
    }

    #[test]
    #[should_panic(expected="Phantom types can not be used as parameter types")]
    fn phantom_fun_param() {
        parse_and_deploy("testFailFunPhantomParam").unwrap();
    }

    #[test]
    #[should_panic(expected="Phantom types can not be used as return types")]
    fn phantom_fun_return() {
        parse_and_deploy("testFailFunPhantomReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="A type from the current module is required to be applied to a protected generic")]
    fn protected_fun_call_fail1() {
        parse_and_deploy("testFailProtectedCall1").unwrap();
    }

    #[test]
    #[should_panic(expected="A type from the current module is required to be applied to a protected generic")]
    fn protected_fun_call_fail2() {
        parse_and_deploy("testFailProtectedCall2").unwrap();
    }

    #[test]
    #[should_panic(expected="A type from the current module is required to be applied to a protected generic")]
    fn protected_fun_call_fail3() {
        parse_and_deploy("testFailProtectedCall3").unwrap();
    }

    #[test]
    #[should_panic(expected="A type from the current module is required to be applied to a protected generic")]
    fn protected_fun_call_fail4() {
        parse_and_deploy("testFailProtectedCall4").unwrap();
    }

    #[test]
    #[should_panic(expected="A type from the current module is required to be applied to a protected generic")]
    fn protected_fun_call_fail5() {
        parse_and_deploy("testFailProtectedCall5").unwrap();
    }

    #[test]
    #[should_panic(expected="A private permission must be from the current module")]
    fn private_fun_call_fail() {
        parse_and_deploy("testFailPrivateCall").unwrap();
    }

    #[test]
    #[should_panic(expected="Copy requires copy capability for input")]
    fn copy_adt_fail() {
        parse_and_deploy("testFailCopy").unwrap();
    }

    #[test]
    #[should_panic(expected="Copy requires copy capability for input")]
    fn copy_adt_fail2() {
        parse_and_deploy("testFailCopy2").unwrap();
    }

    #[test]
    #[should_panic(expected="Capabilities of type must full fill the constraints")]
    fn persist_fail() {
        parse_and_deploy("testFailPersist").unwrap();
    }

    #[test]
    #[should_panic(expected="Discard requires drop capability for input")]
    fn drop_fail() {
        parse_and_deploy("testFailDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Discard requires drop capability for input")]
    fn drop_fail2() {
        parse_and_deploy("testFailDrop2").unwrap();
    }

    #[test]
    #[should_panic(expected="Discard requires drop capability for input")]
    fn drop_fail3() {
        parse_and_deploy("testFailDrop3").unwrap();
    }

    #[test]
    #[should_panic(expected="Discard requires drop capability for input")]
    fn drop_fail4() {
        parse_and_deploy("testFailDrop4").unwrap();
    }

    #[test]
    #[should_panic(expected="Types with the Primitive capability must have Drop, Copy, Persist, and Unbound as well")]
    fn prim_fail() {
        parse_and_deploy("testFailPrimitive").unwrap();
    }

    #[test]
    #[should_panic(expected="Types with the Primitive capability must have Drop, Copy, Persist, and Unbound as well")]
    fn prim_fail2() {
        parse_and_deploy("testFailPrimitive2").unwrap();
    }

    #[test]
    #[should_panic(expected="Types with the Primitive capability must have Drop, Copy, Persist, and Unbound as well")]
    fn prim_fail3() {
        parse_and_deploy("testFailPrimitive3").unwrap();
    }

    #[test]
    #[should_panic(expected="Types with the Primitive capability must have Drop, Copy, Persist, and Unbound as well")]
    fn prim_fail4() {
        parse_and_deploy("testFailPrimitive4").unwrap();
    }

    #[test]
    #[should_panic(expected="A primitive data type can only have public permissions")]
    fn prim_fail5() {
        parse_and_deploy("testFailPrimitive5").unwrap();
    }

    #[test]
    #[should_panic(expected="A primitive data type can only have public permissions")]
    fn prim_fail6() {
        parse_and_deploy("testFailPrimitive6").unwrap();
    }

    #[test]
    #[should_panic(expected="A primitive data type can only have public permissions")]
    fn prim_fail7() {
        parse_and_deploy("testFailPrimitive7").unwrap();
    }

    #[test]
    #[should_panic(expected="A primitive data type can only have public permissions")]
    fn prim_fail8() {
        parse_and_deploy("testFailPrimitive8").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong Permission supplied")]
    fn unpack_type_fail() {
        parse_and_deploy("testFailUnpackType").unwrap();
    }

    #[test]
    #[should_panic(expected="Permissions not applicable to literal")]
    fn lit_unpack_fail() {
        parse_and_deploy("testFailUnpackType3").unwrap();
    }

    #[test]
    #[should_panic(expected="Unpack must target a data type with a single constructor")]
    fn empty_unpack_fail() {
        parse_and_deploy("testFailUnpackType5").unwrap();
    }

    #[test]
    #[should_panic(expected="Unpack must target a data type with a single constructor")]
    fn union_unpack_fail() {
        parse_and_deploy("testFailUnpackType7").unwrap();
    }

    #[test]
    #[should_panic(expected="Returned value has different type from return type declaration of function")]
    fn unpack_image_type_fail() {
        parse_and_deploy("testFailImageUnpack").unwrap();
    }

    #[test]
    #[should_panic(expected="Returned value has different type from return type declaration of function")]
    fn unpack_image_type_fail2() {
        parse_and_deploy("testFailImageUnpack2").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong Permission supplied")]
    fn unpack_image_type_fail3() {
        parse_and_deploy("testFailImageUnpack3").unwrap();
    }


    #[test]
    #[should_panic(expected="Returned value has different type from return type declaration of function")]
    fn switch_image_type_fail() {
        parse_and_deploy("testFailImageSwitch").unwrap();
    }

    #[test]
    #[should_panic(expected="Returned value has different type from return type declaration of function")]
    fn switch_image_type_fail2() {
        parse_and_deploy("testFailImageSwitch2").unwrap();
    }

    #[test]
    #[should_panic(expected="Returned value has different type from return type declaration of function")]
    fn switch_image_type_fail3() {
        parse_and_deploy("testFailImageSwitch3").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn switch_read_type_fail() {
        parse_and_deploy("testFailReadSwitch").unwrap();
    }

    #[test]
    #[should_panic(expected="A consumed or locked element can not be fetched")]
    fn switch_read_type_fail2() {
        parse_and_deploy("testFailReadSwitch2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn switch_read_type_fail3() {
        parse_and_deploy("testFailReadSwitch3").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn switch_read_type_fail4() {
        parse_and_deploy("testFailReadSwitch4").unwrap();
    }


    #[test]
    #[should_panic(expected="Returned value has different type from return type declaration of function")]
    fn field_image_type_fail() {
        parse_and_deploy("testFailImageField").unwrap();
    }

    #[test]
    #[should_panic(expected="Returned value has different type from return type declaration of function")]
    fn field_image_type_fail2() {
        parse_and_deploy("testFailImageField").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong Permission supplied")]
    fn field_image_type_fail3() {
        parse_and_deploy("testFailImageField3").unwrap();
    }

    #[test]
    #[should_panic(expected="A private permission must be from the current module")]
    fn unpack_cap_fail() {
        parse_and_deploy("testFailCapUnpack").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn switch_type_fail() {
        parse_and_deploy("testFailSwitchType").unwrap();
    }

    #[test]
    #[should_panic(expected="Requested constructor does not exist")]
    fn switch_type_fail4() {
        parse_and_deploy("testFailSwitchType4").unwrap();
    }

    #[test]
    #[should_panic(expected="Returned value has different type from return type declaration of function")]
    fn image_fail() {
        parse_and_deploy("testFailImage").unwrap();
    }

    #[test]
    #[should_panic(expected="Returned value has different type from return type declaration of function")]
    fn image_fail2() {
        parse_and_deploy("testFailImage2").unwrap();
    }


    #[test]
    #[should_panic(expected="Returned value has different type from return type declaration of function")]
    fn pack_image_type_fail() {
        parse_and_deploy("testFailImagePack").unwrap();
    }

    #[test]
    #[should_panic(expected="Parameter for data constructor has wrong type")]
    fn pack_image_type_fail2() {
        parse_and_deploy("testFailImagePack2").unwrap();
    }

    #[test]
    #[should_panic(expected="Returned value has different type from return type declaration of function")]
    fn pack_image_type_fail3() {
        parse_and_deploy("testFailImagePack3").unwrap();
    }

    #[test]
    #[should_panic(expected="A private permission must be from the current module")]
    fn pack_caps_fail() {
        parse_and_deploy("testFailCreate").unwrap();
    }

    #[test]
    #[should_panic(expected="Only create, consume & inspect permissions have constructors")]
    fn pack_lit_fail() {
        parse_and_deploy("testFailCreate3").unwrap();
    }


    #[test]
    #[should_panic(expected="Requested constructor does not exist")]
    fn pack_no_ctr_fail() {
        parse_and_deploy("testFailCreate4").unwrap();
    }

    #[test]
    #[should_panic(expected="Parameter for data constructor has wrong type")]
    fn pack_type_fail() {
        parse_and_deploy("testFailPackType").unwrap();
    }

    #[test]
    #[should_panic(expected="Parameter for data constructor has wrong type")]
    fn pack_type_fail2() {
        parse_and_deploy("testFailPackType2").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong number of parameter for function call")]
    fn wrong_arg_call_fail() {
        parse_and_deploy("testFailFunCall").unwrap();
    }

    #[test]
    #[should_panic(expected="Wrong number of parameter for function call")]
    fn wrong_arg_call_fail2() {
        parse_and_deploy("testFailFunCall2").unwrap();
    }

    #[test]
    #[should_panic(expected="Parameter for function call has wrong type")]
    fn wrong_arg_type_fail() {
        parse_and_deploy("testFailFunCall3").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_moved_fail() {
        parse_and_deploy("testFailConsumeMoved").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_moved_fail2() {
        parse_and_deploy("testFailConsumeMoved2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn consume_moved_fail3() {
        parse_and_deploy("testFailConsumeMoved3").unwrap();
    }

    #[test]
    #[should_panic(expected="Discard requires drop capability for input")]
    fn free_unconsumed_fail() {
        parse_and_deploy("testFailFreeUnconsumed").unwrap();
    }

    #[test]
    #[should_panic(expected="Discard requires drop capability for input")]
    fn free_unconsumed_fail2() {
        parse_and_deploy("testFailFreeUnconsumed2").unwrap();
    }

    #[test]
    #[should_panic(expected="Discard requires drop capability for input")]
    fn free_unconsumed_fail3() {
        parse_and_deploy("testFailFreeUnconsumed3").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn return_consumed_fail() {
        parse_and_deploy("testFailReturnMoved").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn return_consumed_fail2() {
        parse_and_deploy("testFailReturnMoved2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn return_consumed_fail3() {
        parse_and_deploy("testFailReturnMoved3").unwrap();
    }

    #[test]
    #[should_panic(expected="A consumed, locked or hidden element can not be hidden")]
    fn dual_arg_fail() {
        parse_and_deploy("testFailDualArg").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn copy_consumed_fail() {
        parse_and_deploy("testFailCopyConsumed").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn fun_double_return_fail() {
        parse_and_deploy("testFailFunDoubleReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn let_double_return_fail() {
        parse_and_deploy("testFailLetDoubleReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn switch_double_return_fail() {
        parse_and_deploy("testFailSwitchDoubleReturn").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn switch_double_return_fail2() {
        parse_and_deploy("testFailSwitchDoubleReturn2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn fun_double_drop_fail() {
        parse_and_deploy("testFailFunDoubleDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn let_double_drop_fail() {
        parse_and_deploy("testFailLetDoubleDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn switch_double_drop_fail() {
        parse_and_deploy("testFailSwitchDoubleDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn switch_double_drop_fail2() {
        parse_and_deploy("testFailSwitchDoubleDrop2").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn fun_return_drop_fail() {
        parse_and_deploy("testFailFunReturnDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn let_return_drop_fail() {
        parse_and_deploy("testFailLetReturnDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn switch_return_drop_fail() {
        parse_and_deploy("testFailSwitchReturnDrop").unwrap();
    }

    #[test]
    #[should_panic(expected="Consumed, borrowed, or locked element can not be consumed")]
    fn switch_return_drop_fail2() {
        parse_and_deploy("testFailSwitchReturnDrop2").unwrap();
    }

    #[test]
    #[should_panic(expected="Branches must produce same returns")]
    fn branch_return_type_fail6() {
        parse_and_deploy("testFailBranchReturnType6").unwrap();
    }

    #[test]
    #[should_panic(expected="Branches must produce same returns")]
    fn branch_return_type_fail7() {
        parse_and_deploy("testFailBranchReturnType7").unwrap();
    }

    #[test]
    #[should_panic(expected="Branches must produce same returns")]
    fn branch_return_type_fail8() {
        parse_and_deploy("testFailBranchReturnType8").unwrap();
    }

    #[test]
    #[should_panic(expected="Branches must produce same returns")]
    fn branch_return_type_fail9() {
        parse_and_deploy("testFailBranchReturnType9").unwrap();
    }

    #[test]
    #[should_panic(expected="Branches must consume same stack slots")]
    fn branch_diff_consumes_fail6() {
        parse_and_deploy("testFailBranchDiffConsumes6").unwrap();
    }

    #[test]
    #[should_panic(expected="Branches must consume same stack slots")]
    fn branch_diff_consumes_fail7() {
        parse_and_deploy("testFailBranchDiffConsumes7").unwrap();
    }

    #[test]
    #[should_panic(expected="Branches must consume same stack slots")]
    fn branch_diff_consumes_fail8() {
        parse_and_deploy("testFailBranchDiffConsumes8").unwrap();
    }

    #[test]
    #[should_panic(expected="Branches must consume same stack slots")]
    fn branch_diff_consumes_fail9() {
        parse_and_deploy("testFailBranchDiffConsumes9").unwrap();
    }

    #[test]
    #[should_panic(expected="Branches must consume same stack slots")]
    fn branch_diff_consumes_fail10() {
        parse_and_deploy("testFailBranchDiffConsumes10").unwrap();
    }

    #[test]
    #[should_panic(expected= "Parameters must be borrowed or consumed at the end of a function body")]
    fn param_consume_fail() {
        parse_and_deploy("testFailFunParamConsume").unwrap();
    }

    #[test]
    #[should_panic(expected= "Consumed, borrowed, or locked element can not be consumed")]
    fn param_consume_fail2() {
        parse_and_deploy("testFailFunParamConsume2").unwrap();
    }

    #[bench]
    fn validate_succ_bench_deploy(b: &mut Bencher) {
        parse_and_deploy_bench("testSucValidate", b).unwrap();
    }

    #[test]
    fn validate_succ(){
        parse_and_deploy("testSucValidate").unwrap();
    }

    #[bench]
    fn check_succ_bench_deploy(b: &mut Bencher) {
        parse_and_deploy_bench("testSucTypeCheck", b).unwrap();
    }

    #[test]
    fn check_succ(){
        parse_and_deploy("testSucTypeCheck").unwrap();
    }

    #[bench]
    fn lin_succ_bench_deploy(b: &mut Bencher) {
        parse_and_deploy_bench("testSucLinearity", b).unwrap();
    }

    #[test]
    fn lin_succ(){
        parse_and_deploy("testSucLinearity").unwrap();
    }

    #[test]
    fn cap_succ(){
        parse_and_deploy("testSucCaps").unwrap();
    }

}
