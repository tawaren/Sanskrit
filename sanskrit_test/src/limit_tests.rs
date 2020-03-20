#[cfg(test)]
mod tests {
    use test::Bencher;
    use generator::generate;
    use sanskrit_deploy::deploy_module;
    use sanskrit_memory_store::BTreeMapStore;
    use sanskrit_common::errors::*;
    use sanskrit_common::store::{Store, StorageClass, store_hash};
    use sanskrit_core::model::Module;
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

    fn print_accounting(accounting:Accounting) {
        println!("  Stored {} Bytes", usize::max_value() - accounting.store_byte_budget.get());
        println!("  Loaded {} Bytes", usize::max_value() - accounting.load_byte_budget.get());
        println!("  Processed {} Bytes", usize::max_value() - accounting.process_byte_budget.get());
        println!("  Processed {} Stack Items", usize::max_value() - accounting.stack_elem_budget.get());
    }

    fn shared() -> bool {
        true
    }

    fn overfill() -> bool {
        false
    }

    fn body() -> bool {
        false
    }

    fn generate_and_deploy(n:u8) -> Result<()>{
        let data = generate(n, shared(), overfill(), body());
        let s = BTreeMapStore::new();
        println!("Start Test");
        for d in data {
            let accounting = max_accounting();
            deploy_module(&s, &accounting, d, false).unwrap();
            println!("Deployed Module");
            print_accounting(accounting);
        }
        println!("End Test");


        Ok(())
    }

    fn generate_and_deploy_first_bench(n:u8, b: &mut Bencher) -> Result<()>{
        let data = generate(n, shared(), overfill(), body());
        b.iter(|| {
            let s = BTreeMapStore::new();
            let accounting = max_accounting();
            deploy_module(&s, &accounting, data[0].clone(), false).unwrap();
        });
        Ok(())
    }


    fn generate_and_deploy_last_bench(n:u8, b: &mut Bencher) -> Result<()>{
        let data = generate(n, shared(), overfill(), body());
        let s = BTreeMapStore::new();
        let accounting = max_accounting();
        let size = data.len();
        let last_hash = store_hash(&[&data[size-1]]);
        for d in &data[0..(size-1)] {
            deploy_module(&s, &accounting, d.clone(), false).unwrap();
        }
        b.iter(|| {
            deploy_module(&s, &accounting, data[size-1].clone(), false).unwrap();
            s.delete(StorageClass::Module, &last_hash)
        });
        Ok(())
    }

    #[test]
    fn test_01() {
        generate_and_deploy(1).unwrap();
    }

    /*#[bench]
    fn test_01_first_bench(b: &mut Bencher) {
        generate_and_deploy_first_bench(1, b).unwrap();
    }*/

    #[bench]
    fn test_01_last_bench(b: &mut Bencher) {
        generate_and_deploy_last_bench(1, b).unwrap();
    }

    #[test]
    fn test_02() {
        generate_and_deploy(2).unwrap();
    }

    /*#[bench]
    fn test_02_first_bench(b: &mut Bencher) {
        generate_and_deploy_first_bench(2, b).unwrap();
    }*/

    #[bench]
    fn test_02_last_bench(b: &mut Bencher) {
        generate_and_deploy_last_bench(2, b).unwrap();
    }

    #[test]
    fn test_04() {
        generate_and_deploy(4).unwrap();
    }

    /*#[bench]
    fn test_04_first_bench(b: &mut Bencher) {
        generate_and_deploy_first_bench(4, b).unwrap();
    }*/

    #[bench]
    fn test_04_last_bench(b: &mut Bencher) {
        generate_and_deploy_last_bench(4, b).unwrap();
    }

    #[test]
    fn test_08() {
        generate_and_deploy(8).unwrap();
    }

    /*#[bench]
    fn test_08_first_bench(b: &mut Bencher) {
        generate_and_deploy_first_bench(8, b).unwrap();
    }*/

    #[bench]
    fn test_08_last_bench(b: &mut Bencher) {
        generate_and_deploy_last_bench(8, b).unwrap();
    }

    #[test]
    fn test_10() {
        generate_and_deploy(16).unwrap();
    }

    /*#[bench]
    fn test_10_first_bench(b: &mut Bencher) {
        generate_and_deploy_first_bench(16, b).unwrap();
    }*/

    #[bench]
    fn test_10_last_bench(b: &mut Bencher) {
        generate_and_deploy_last_bench(16, b).unwrap();
    }

    #[test]
    fn test_20() {
        generate_and_deploy(32).unwrap();
    }

    /*#[bench]
    fn test_20_first_bench(b: &mut Bencher) {
        generate_and_deploy_first_bench(32, b).unwrap();
    }*/

    #[bench]
    fn test_20_last_bench(b: &mut Bencher) {
        generate_and_deploy_last_bench(32, b).unwrap();
    }

    #[test]
    fn test_40() {
        generate_and_deploy(64).unwrap();
    }

    /*#[bench]
    fn test_40_first_bench(b: &mut Bencher) {
        generate_and_deploy_first_bench(64, b).unwrap();
    }*/

    #[bench]
    fn test_40_last_bench(b: &mut Bencher) {
        generate_and_deploy_last_bench(64, b).unwrap();
    }

    #[test]
    fn test_80() {
        generate_and_deploy(128).unwrap();
    }

    /*#[bench]
    fn test_80_first_bench(b: &mut Bencher) {
        generate_and_deploy_first_bench(128, b).unwrap();
    }*/

    #[bench]
    fn test_80_last_bench(b: &mut Bencher) {
        generate_and_deploy_last_bench(128, b).unwrap();
    }

}