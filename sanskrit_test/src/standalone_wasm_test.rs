//todo: remove the sig stuff

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
    use std::io::{Write, Read};
    use test::{Bencher, iter};
    use sanskrit_memory_store::BTreeMapStore;
    use std::env::current_dir;
    use wasmi::{ModuleInstance, ImportsBuilder, RuntimeValue, NopExternals, MemoryDescriptor};
    use rand::rngs::OsRng;
    use sha2::Sha512;
    use ed25519_dalek::{Keypair, Signature};
    use wasmer_runtime::Value;
    #[macro_use]
    use wasmer_runtime::{imports,func};
    use std::cell::{RefCell, Cell};
    use wasmer_runtime::cache::WasmHash;

    fn load_from_file(file:&str) -> Vec<u8> {
        let wasm = current_dir().unwrap().join(file);
        let mut f = File::open(wasm).unwrap();
        let mut contents = vec![];
        f.read_to_end(&mut contents).unwrap();
        contents
    }

    trait Engine {
        type Module;
        type Instance;
        type State;
        fn load() -> Self::Module;
        fn instantiate(module:&Self::Module) -> Self::Instance;
        fn state() -> Self::State;
        fn execute(inst:&mut Self::Instance, state:&Self::State, exec_code:&[u8]) -> Option<u64>;
        fn reset(inst:&mut Self::Instance);
    }


    struct WasmI;
    impl Engine for WasmI {
        type Module = wasmi::Module;
        type Instance = wasmi::ModuleRef;
        type State = ();
        fn load() -> Self::Module {
            let wasm_code = load_from_file("wasm/sanskrit_wasm_deploy.wasm");
            wasmi::Module::from_buffer(&wasm_code).unwrap()
        }

        fn instantiate(module:&Self::Module) -> Self::Instance {
            ModuleInstance::new(
                module,
                &ImportsBuilder::default()
            ).expect("failed to instantiate wasm module").assert_no_start()
        }

        fn state() -> Self::State { }

        fn execute(instance:&mut Self::Instance, gas:&Self::State, exec_code:&[u8]) -> Option<u64> {
            let ptr = match instance.invoke_export(
                "alloc",
                &[RuntimeValue::I32(exec_code.len() as i32)],
                &mut NopExternals,
            ).expect("failed to execute alloc export").unwrap()  {
                RuntimeValue::I32(ptr) => ptr,
                RuntimeValue::I64(_) => panic!(),
                RuntimeValue::F32(_) => panic!(),
                RuntimeValue::F64(_) => panic!(),
            };

            let mem = instance.export_by_name("memory")
                .unwrap().as_memory().cloned().unwrap();

            mem.set(ptr as u32, &exec_code).expect("memory set failed");

            match instance.invoke_export(
                "call",
                &[RuntimeValue::I32(ptr), RuntimeValue::I32(exec_code.len() as i32)],
                &mut NopExternals,
            ).expect("failed to execute call export") {
                Some(RuntimeValue::I32(res)) => if res == 0 {
                    Some(0)
                } else {
                    None
                },
                _ => panic!()
            }
        }

        fn reset(instance: &mut Self::Instance) {
            instance.invoke_export(
                "reset",
                &[],
                &mut NopExternals,
            ).expect("failed to execute reset export");
        }
    }


    struct Native;
    impl Engine for Native {
        type Module = ();
        type Instance = ();
        type State = BTreeMapStore;
        fn load() -> Self::Module {}
        fn instantiate(module:&Self::Module) -> Self::Instance { }
        fn state() -> Self::State { BTreeMapStore::new() }
        fn execute(instance:&mut Self::Instance, s:&Self::State, exec_code:&[u8]) -> Option<u64> {
            deploy_module(s, exec_code.to_vec()).unwrap();
            Some(0)
        }

        fn reset(inst: &mut Self::Instance) { }
    }

    thread_local! {
        pub static Calls: RefCell<u32> = RefCell::new(0);
    }

    trait WasmerConfig {
        fn target_file() -> &'static str;
        fn imports(module:&wasmer_runtime::Module) -> wasmer_runtime::ImportObject;
        fn extract_gas(instance:&wasmer_runtime::Instance)->u32;
    }

    impl<T:WasmerConfig> Engine for T{
        type Module = wasmer_runtime::Module;
        type Instance = wasmer_runtime::Instance;
        type State = ();

        fn load() -> Self::Module {
            let wasm_code = load_from_file(Self::target_file());
            wasmer_runtime::compile(&wasm_code ).unwrap()
        }
        fn instantiate(module:&Self::Module) -> Self::Instance {
            let import_object = Self::imports(module);
            module.instantiate(&import_object).unwrap()
        }
        fn state() -> Self::State { }
        fn execute(instance:&mut Self::Instance, _:&Self::State, exec_code:&[u8]) -> Option<u64> {
            let ptr = match instance.call(
                "alloc",
                &[Value::I32(exec_code.len() as i32)]
            ).expect("failed to execute alloc export")[0]  {
                Value::I32(ptr) => ptr,
                Value::I64(_) => panic!(),
                Value::F32(_) => panic!(),
                Value::F64(_) => panic!(),
            };

            {
                let memory = instance.context_mut().memory(0);

                for (byte, cell) in exec_code.iter().zip(memory.view()[ptr as usize..(ptr as usize + exec_code.len())].iter()){
                    cell.set(*byte);
                }
            }

            match instance.call(
                "call",
                &[Value::I32(ptr), Value::I32(exec_code.len() as i32)]
            ).expect("failed to execute call export")[0] {
                Value::I32(res) =>  if res == 0 {
                    Some(0)
                } else {
                    None
                },
                _ => panic!()
            }
        }

        fn reset(instance: &mut Self::Instance) {
            //todo: get real gas so we can check
            //assert_eq!(0,T::extract_gas(instance));
            instance.call(
                "reset",
                &[]
            ).expect("failed to execute reset export");
        }
    }

    fn oog_wasmer(_:&mut wasmer_runtime::Ctx){
        Calls.with(|f| {
            *f.borrow_mut() += 1;
        });
    }

    fn gas_wasmer(_:&mut wasmer_runtime::Ctx, amount:u32){
        Calls.with(|f| {
            *f.borrow_mut() += amount;
        });
    }

    fn external_imports() -> wasmer_runtime::ImportObject{imports!{
        "env" => {
            "gas" => func!(gas_wasmer),
        },
    } }

    const GAS_LIMIT_INTERNAL:i32 = 1000000000;
    fn internal_imports() -> wasmer_runtime::ImportObject { imports!{
        "env" => {
            "oog" => func!(oog_wasmer),
            "limit" => wasmer_runtime::Global::new(Value::I32(GAS_LIMIT_INTERNAL)),
        },
    } }

    fn default_imports() -> wasmer_runtime::ImportObject {
        internal_imports()
    }

    fn auto_detect_imports(module:&wasmer_runtime::Module)  -> wasmer_runtime::ImportObject {
        for (name,_) in  &module.info().exports {
            if name == "gas" {
                return internal_imports()
            }
        }
        external_imports()
    }

    fn extract_internal(instance:&wasmer_runtime::Instance) -> u32 {
        match instance.call("gas",&[]).unwrap()[0] {
            Value::I32(res) => (GAS_LIMIT_INTERNAL - res) as u32,
            _ => panic!()
        }
    }

    fn extract_external() -> u32 {
        Calls.with(|f| {
            *f.borrow_mut()
        })
    }

    fn auto_extract_gas(instance:&wasmer_runtime::Instance)  -> u32 {
        for (name,_) in  instance.exports() {
            if name == "gas" {
                return extract_internal(instance)
            }
        }
        return extract_external();
    }

    struct Wasmer;
    impl WasmerConfig for Wasmer{
        fn target_file() -> &'static str { "wasm/sanskrit_wasm_deploy.wasm" }
        fn imports(_module:&wasmer_runtime::Module) -> wasmer_runtime::ImportObject { imports!{} }
        fn extract_gas(instance: &wasmer_runtime::Instance) -> u32 { 0}
    }

    struct WasmerGasOrig;
    impl WasmerConfig for WasmerGasOrig{
        fn target_file() -> &'static str { "wasm/sanskrit_wasm_deploy_gas.wasm" }
        fn imports(module:&wasmer_runtime::Module) -> wasmer_runtime::ImportObject {auto_detect_imports(module)}
        fn extract_gas(instance: &wasmer_runtime::Instance) -> u32 { auto_extract_gas(instance) }

    }

    struct WasmerGasOpt;
    impl WasmerConfig for WasmerGasOpt {
        fn target_file() -> &'static str { "wasm/sanskrit_wasm_deploy_gas_opt.wasm" }
        fn imports(module:&wasmer_runtime::Module) -> wasmer_runtime::ImportObject {auto_detect_imports(module)}
        fn extract_gas(instance: &wasmer_runtime::Instance) -> u32 { auto_extract_gas(instance) }
    }

    struct WasmerGasMax;
    impl WasmerConfig for WasmerGasMax {
        fn target_file() -> &'static str { "wasm/sanskrit_wasm_deploy_gas_max.wasm" }
        fn imports(module:&wasmer_runtime::Module) -> wasmer_runtime::ImportObject {auto_detect_imports(module)}
        fn extract_gas(instance: &wasmer_runtime::Instance) -> u32 { auto_extract_gas(instance) }
    }

    struct WasmerGasSec;
    impl WasmerConfig for WasmerGasSec {
        fn target_file() -> &'static str { "wasm/sanskrit_wasm_deploy_gas_new.wasm" }
        fn imports(module:&wasmer_runtime::Module) -> wasmer_runtime::ImportObject {auto_detect_imports(module)}
        fn extract_gas(instance: &wasmer_runtime::Instance) -> u32 { auto_extract_gas(instance) }
    }

    //todo: use db
    fn parse_and_compile_deploy_bench<E:Engine>(id: &str, b: &mut Bencher) -> Result<()> {
        let id = Id(id.into());
        let folder = current_dir().unwrap().join("scripts");

        let mut comp = Compiler::new(&folder);
        comp.parse_module_tree(Id("system".into()));
        comp.parse_module_tree(id.clone());
        comp.compile_module_tree().unwrap();
        let res = comp.get_results();
        let module = E::load();
        let mut instance = E::instantiate(&module);

        b.iter(|| {
            let s = E::state();
            for (_, r) in &res {
                E::execute(&mut instance, &s,r).unwrap();
            }
            E::reset(&mut instance)
        });
        Ok(())
    }


    #[bench]
    fn build_system_native_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Native>("system", b).unwrap();
    }

    #[bench]
    fn build_system_wasmi_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmI>("system", b).unwrap();
    }

    #[bench]
    fn build_system_wasmer_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Wasmer>("system", b).unwrap();
    }

    #[bench]
    fn build_system_wasmer_gas_bench(b: &mut Bencher) {
        // Gas used: 1'204'244 // Block - non-accounted
        // Gas used: 1'927'772 // Block - accounted
        parse_and_compile_deploy_bench::<WasmerGasOrig>("system", b).unwrap();
    }

    #[bench]
    fn build_system_wasmer_gas_new_bench(b: &mut Bencher) {
        // Gas used: 542'917   // Sec - non-accounted
        // Gas used: 1'547'053 // Sec - accounted
        parse_and_compile_deploy_bench::<WasmerGasSec>("system", b).unwrap();
    }

    #[bench]
    fn build_system_wasmer_gas_opt_bench(b: &mut Bencher) {
        // Gas used: 542'917 // precharge - non-accounted
        // Gas used: 927'831 // precharge - accounted
        parse_and_compile_deploy_bench::<WasmerGasOpt>("system", b).unwrap();
    }

    #[bench]
    fn build_system_wasmer_gas_max_bench(b: &mut Bencher) {
        // Gas used: 629'390 // maxcharge - non-accounted
        // Gas used: 986'762 // maxcharge - accounted
        parse_and_compile_deploy_bench::<WasmerGasMax>("system", b).unwrap();
    }

    #[bench]
    fn build_diff_adts_native_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Native>("testSucAdt",b).unwrap();
    }

    #[bench]
    fn build_diff_adts_wasmi_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmI>("testSucAdt",b).unwrap();
    }

    #[bench]
    fn build_diff_adts_wasmer_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Wasmer>("testSucAdt",b).unwrap();
    }

    #[bench]
    fn build_diff_adts_wasmer_gas_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOrig>("testSucAdt",b).unwrap();
    }

    #[bench]
    fn build_diff_adts_wasmer_gas_new_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasSec>("testSucAdt", b).unwrap();
    }

    #[bench]
    fn build_diff_adts_wasmer_gas_opt_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOpt>("testSucAdt", b).unwrap();
    }

    #[bench]
    fn build_diff_adts_wasmer_gas_max_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasMax>("testSucAdt", b).unwrap();
    }

    #[bench]
    fn build_diff_adts_import_native_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Native>("testSucAdtImport",b).unwrap();
    }

    #[bench]
    fn build_diff_adts_import_wasmi_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmI>("testSucAdtImport",b).unwrap();
    }

    #[bench]
    fn build_diff_adts_import_wasmer_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Wasmer>("testSucAdtImport",b).unwrap();
    }

    #[bench]
    fn build_diff_adts_import_wasmer_gas_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOrig>("testSucAdtImport",b).unwrap();
    }

    #[bench]
    fn build_diff_adts_import_wasmer_gas_new_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasSec>("testSucAdtImport", b).unwrap();
    }

    #[bench]
    fn build_diff_adts_import_wasmer_gas_opt_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOpt>("testSucAdtImport", b).unwrap();
    }

    #[bench]
    fn build_diff_adts_import_wasmer_gas_max_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasMax>("testSucAdtImport", b).unwrap();
    }

    #[bench]
    fn build_create_open_adts_native_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Native>("testSucCreateOpenAdts",b).unwrap();
    }

    #[bench]
    fn build_create_open_adts_wasmi_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmI>("testSucCreateOpenAdts",b).unwrap();
    }

    #[bench]
    fn build_create_open_adts_wasmer_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Wasmer>("testSucCreateOpenAdts",b).unwrap();
    }

    #[bench]
    fn build_create_open_adts_wasmer_gas_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOrig>("testSucCreateOpenAdts",b).unwrap();
    }

    #[bench]
    fn build_create_open_adts_wasmer_gas_new_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasSec>("testSucCreateOpenAdts", b).unwrap();
    }

    #[bench]
    fn build_create_open_adts_wasmer_gas_opt_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOpt>("testSucCreateOpenAdts", b).unwrap();
    }

    #[bench]
    fn build_create_open_adts_wasmer_gas_max_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasMax>("testSucCreateOpenAdts", b).unwrap();
    }

    #[bench]
    fn build_create_closed_adts_native_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Native>("testSucCreateClosedAdts",b).unwrap();
    }

    #[bench]
    fn build_create_closed_adts_wasmi_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmI>("testSucCreateClosedAdts",b).unwrap();
    }

    #[bench]
    fn build_create_closed_adts_wasmer_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Wasmer>("testSucCreateClosedAdts",b).unwrap();
    }

    #[bench]
    fn build_create_closed_adts_wasmer_gas_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOrig>("testSucCreateClosedAdts",b).unwrap();
    }

    #[bench]
    fn build_create_closed_adts_wasmer_gas_new_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasSec>("testSucCreateClosedAdts", b).unwrap();
    }

    #[bench]
    fn build_create_closed_adts_wasmer_gas_opt_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOpt>("testSucCreateClosedAdts", b).unwrap();
    }

    #[bench]
    fn build_create_closed_adts_wasmer_gas_max_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasMax>("testSucCreateClosedAdts", b).unwrap();
    }

    #[bench]
    fn build_borrow_create_open_adts_native_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Native>("testSucBorrowCreateOpenAdts",b).unwrap();
    }

    #[bench]
    fn build_borrow_create_open_adts_wasmi_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmI>("testSucBorrowCreateOpenAdts",b).unwrap();
    }

    #[bench]
    fn build_borrow_create_open_adts_wasmer_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Wasmer>("testSucBorrowCreateOpenAdts",b).unwrap();
    }

    #[bench]
    fn build_borrow_create_open_adts_wasmer_gas_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOrig>("testSucBorrowCreateOpenAdts",b).unwrap();
    }

    #[bench]
    fn build_borrow_create_open_adts_wasmer_gas_new_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasSec>("testSucBorrowCreateOpenAdts", b).unwrap();
    }

    #[bench]
    fn build_borrow_create_open_adts_wasmer_gas_opt_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOpt>("testSucBorrowCreateOpenAdts", b).unwrap();
    }

    #[bench]
    fn build_borrow_create_open_adts_wasmer_gas_max_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasMax>("testSucBorrowCreateOpenAdts", b).unwrap();
    }

    #[bench]
    fn define_and_call_functions_native_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Native>("testSucFun",b).unwrap();
    }

    #[bench]
    fn define_and_call_functions_wasmi_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmI>("testSucFun",b).unwrap();
    }

    #[bench]
    fn define_and_call_functions_wasmer_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Wasmer>("testSucFun",b).unwrap();
    }

    #[bench]
    fn define_and_call_functions_wasmer_gas_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOrig>("testSucFun",b).unwrap();
    }

    #[bench]
    fn define_and_call_functions_wasmer_gas_new_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasSec>("testSucFun", b).unwrap();
    }

    #[bench]
    fn define_and_call_functions_wasmer_gas_opt_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOpt>("testSucFun", b).unwrap();
    }

    #[bench]
    fn define_and_call_functions_wasmer_gas_max_bench(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasMax>("testSucFun", b).unwrap();
    }

    #[bench]
    fn validate_succ_native_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Native>("testSucValidate",b).unwrap();
    }

    #[bench]
    fn validate_succ_wasmi_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmI>("testSucValidate",b).unwrap();
    }

    #[bench]
    fn validate_succ_wasmer_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Wasmer>("testSucValidate",b).unwrap();
    }

    #[bench]
    fn validate_succ_wasmer_gas_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOrig>("testSucValidate",b).unwrap();
    }

    #[bench]
    fn validate_succ_wasmer_gas_new_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasSec>("testSucValidate", b).unwrap();
    }

    #[bench]
    fn validate_succ_wasmer_gas_opt_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOpt>("testSucValidate", b).unwrap();
    }

    #[bench]
    fn validate_succ_wasmer_gas_max_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasMax>("testSucValidate", b).unwrap();
    }

    #[bench]
    fn check_succ_native_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Native>("testSucTypeCheck",b).unwrap();
    }

    #[bench]
    fn check_succ_wasmi_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmI>("testSucTypeCheck",b).unwrap();
    }

    #[bench]
    fn check_succ_wasmer_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Wasmer>("testSucTypeCheck",b).unwrap();
    }

    #[bench]
    fn check_succ_wasmer_bench_gas_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOrig>("testSucTypeCheck",b).unwrap();
    }

    #[bench]
    fn check_succ_wasmer_bench_gas_new_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasSec>("testSucTypeCheck", b).unwrap();
    }

    #[bench]
    fn check_succ_wasmer_bench_gas_opt_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOpt>("testSucTypeCheck", b).unwrap();
    }

    #[bench]
    fn check_succ_wasmer_bench_gas_max_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasMax>("testSucTypeCheck", b).unwrap();
    }

    #[bench]
    fn lin_succ_native_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Native>("testSucLinearity",b).unwrap();
    }

    #[bench]
    fn lin_succ_wasmi_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmI>("testSucLinearity",b).unwrap();
    }

    #[bench]
    fn lin_succ_wasmer_bench_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<Wasmer>("testSucLinearity",b).unwrap();
    }

    #[bench]
    fn lin_succ_wasmer_bench_gas_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOrig>("testSucLinearity",b).unwrap();
    }

    #[bench]
    fn lin_succ_wasmer_bench_gas_new_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasSec>("testSucLinearity", b).unwrap();
    }

    #[bench]
    fn lin_succ_wasmer_bench_gas_opt_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasOpt>("testSucLinearity", b).unwrap();
    }

    #[bench]
    fn lin_succ_wasmer_bench_gas_max_deploy(b: &mut Bencher) {
        parse_and_compile_deploy_bench::<WasmerGasMax>("testSucLinearity", b).unwrap();
    }

}