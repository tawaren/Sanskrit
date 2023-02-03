pub(crate) mod benchs {
    use sanskrit_deploy::deploy_module;
    use sanskrit_memory_store::BTreeMapStore;
    use sanskrit_common::errors::*;
    use sanskrit_common::store::{Store, StorageClass, store_hash};
    use crate::bench::validate::generators::generate;
    use std::time::Instant;

    fn shared() -> bool {
        false
    }

    fn overfill() -> bool {
        true
    }

    fn body() -> bool {
        true
    }

    fn est(load:usize, store:usize) -> usize {
        //load * 25 + store * 60 + 7500
        load * 20 + store * 75 + 4000
    }

    fn generate_and_deploy(n:u8, iter:usize) -> Result<()>{
        let data = generate(n, shared(), overfill(), body());
        let start = Instant::now();
        for _ in 0..iter {
            let s = BTreeMapStore::new();
            for d in data.clone() {
                deploy_module(&s, d, false, true).unwrap();
            }
        }
        let elapsed = start.elapsed().as_nanos();
        let bytes:usize = data.iter().map(|d|d.len()).sum();
        println!("ALL: param: {}, store:{}, tot: {}ns, per iter: {}ns, per byte {}ns", n, bytes, elapsed, elapsed/(iter as u128), (elapsed/(iter as u128)/(bytes as u128)));
        
        Ok(())
    }

    fn generate_and_deploy_first_bench(n:u8, iter:usize) -> Result<()>{
        let data = generate(n, shared(), overfill(), body());
        let start = Instant::now();
        for _ in 0..iter {
            let s = BTreeMapStore::new();
            deploy_module(&s, data[0].clone(), false, true).unwrap();
        }
        let elapsed = start.elapsed().as_nanos();
        let est = est(0, data[0].len());
        let dif = ((elapsed as f64)/(iter as f64))/(est as f64);
        println!("FIRST: param: {}, store:{}, tot: {}ns, iter: {}ns, est:{}ns, off:{}, per store byte {}ns", n, data[0].len(), elapsed, elapsed/(iter as u128), est, dif, (elapsed/(iter as u128)/(data[0].len() as u128)));
        Ok(())
    }

    fn generate_and_deploy_last_bench(n:u8, iter:usize) -> Result<()>{
        let data = generate(n, shared(), overfill(), body());
        let s = BTreeMapStore::new();
        let size = data.len();
        let last_hash = store_hash(&[&data[size-1]]);
        for d in &data[0..(size-1)] {
            deploy_module(&s, d.clone(), false, true).unwrap();
        }
        let start = Instant::now();
        for _ in 0..iter {
            deploy_module(&s, data[size - 1].clone(), false, true).unwrap();
            s.delete(StorageClass::Module, &last_hash).unwrap();
            s.commit(StorageClass::Module);
        }
        let elapsed = start.elapsed().as_nanos();
        let tot_bytes:usize = data.iter().map(|d|d.len()).sum();
        let bytes = tot_bytes - data[size - 1].len();
        let est = est(bytes,data[size - 1].len());
        let dif = ((elapsed as f64)/(iter as f64))/(est as f64);
        println!("LAST: param: {}, store:{}, load:{}, tot: {}ns, iter: {}ns, est:{}ns, off:{}, per store byte {}ns, per load byte {}ns, per byte {}ns", n, data[size - 1].len(), bytes, elapsed, elapsed/(iter as u128), est, dif, (elapsed/(iter as u128)/(data[size - 1].len() as u128)), (elapsed/(iter as u128)/(bytes as u128)), (elapsed/(iter as u128)/((bytes + data[size - 1].len())as u128)));
        Ok(())
    }

    
    pub fn bench_01_bench(iter:usize) {
        generate_and_deploy(1,iter).unwrap();
    }

    
    pub fn bench_01_first_bench(iter:usize) {
        generate_and_deploy_first_bench(1, iter).unwrap();
    }

    
    pub fn bench_01_last_bench(iter:usize) {
        generate_and_deploy_last_bench(1, iter).unwrap();
    }

    
    pub fn bench_02_bench(iter:usize) {
        generate_and_deploy(2, iter).unwrap();
    }

    
    pub fn bench_02_first_bench(iter:usize) {
        generate_and_deploy_first_bench(2, iter).unwrap();
    }

    
    pub fn bench_02_last_bench(iter:usize) {
        generate_and_deploy_last_bench(2, iter).unwrap();
    }

    
    pub fn bench_04_bench(iter:usize) {
        generate_and_deploy(4,iter).unwrap();
    }

    
    pub fn bench_04_first_bench(iter:usize) {
        generate_and_deploy_first_bench(4, iter).unwrap();
    }

    
    pub fn bench_04_last_bench(iter:usize) {
        generate_and_deploy_last_bench(4, iter).unwrap();
    }

    
    pub fn bench_08_bench(iter:usize) {
        generate_and_deploy(8, iter).unwrap();
    }

    
    pub fn bench_08_first_bench(iter:usize) {
        generate_and_deploy_first_bench(8, iter).unwrap();
    }

    
    pub fn bench_08_last_bench(iter:usize) {
        generate_and_deploy_last_bench(8, iter).unwrap();
    }

    
    pub fn bench_10_bench(iter:usize) {
        generate_and_deploy(16,iter).unwrap();
    }

    
    pub fn bench_10_first_bench(iter:usize) {
        generate_and_deploy_first_bench(16, iter).unwrap();
    }

    
    pub fn bench_10_last_bench(iter:usize) {
        generate_and_deploy_last_bench(16, iter).unwrap();
    }

    
    pub fn bench_20_bench(iter:usize) {
        generate_and_deploy(32, iter).unwrap();
    }

    
    pub fn bench_20_first_bench(iter:usize) {
        generate_and_deploy_first_bench(32, iter).unwrap();
    }

    
    pub fn bench_20_last_bench(iter:usize) {
        generate_and_deploy_last_bench(32, iter).unwrap();
    }

    pub fn bench_40_bench(iter:usize) {
        generate_and_deploy(64, iter).unwrap();
    }


    pub fn bench_40_first_bench(iter:usize) {
        generate_and_deploy_first_bench(64, iter).unwrap();
    }


    pub fn bench_40_last_bench(iter:usize) {
        generate_and_deploy_last_bench(64, iter).unwrap();
    }
}