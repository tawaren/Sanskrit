echo f | xcopy /y /f "../../target/wasm32-unknown-unknown/release/sanskrit_wasm_deploy_compile.wasm" "sanskrit_wasm_deploy_compile.wasm"
wasm-gas sanskrit_wasm_deploy_compile.wasm sanskrit_wasm_deploy_compile_gas.wasm -v -g -e -m Maxcharge -c gas_cost.toml
