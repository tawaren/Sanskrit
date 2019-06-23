#[cfg(test)]

use super::*;

#[cfg(test)]
mod sig {
    use super::*;
    use ed25519_dalek::*;
    use sha2::{Sha512};
    use rand::rngs::OsRng;


    #[bench]
    fn bench_32(b: &mut Bencher){
        let mut csprng: OsRng = OsRng::new().unwrap();
        let kp = Keypair::generate::<Sha512, _>(&mut csprng);
        let data = [0;32];
        let sig = kp.sign::<Sha512>(&data);

        b.iter(||{
            for _ in 0..1000 {
                kp.verify::<Sha512>(&data, &sig).unwrap();
            }
        })

    }


    #[bench]
    fn bench_64(b: &mut Bencher){
        let mut csprng: OsRng = OsRng::new().unwrap();
        let kp = Keypair::generate::<Sha512, _>(&mut csprng);
        let data = [0;64];
        let sig = kp.sign::<Sha512>(&data);

        b.iter(||{
            for _ in 0..1000 {
                kp.verify::<Sha512>(&data, &sig).unwrap();
            }
        })

    }

    #[bench]
    fn bench_128(b: &mut Bencher){
        let mut csprng: OsRng = OsRng::new().unwrap();
        let kp = Keypair::generate::<Sha512, _>(&mut csprng);
        let data = [0;128];
        let sig = kp.sign::<Sha512>(&data);

        b.iter(||{
            for _ in 0..1000 {
                kp.verify::<Sha512>(&data, &sig).unwrap();
            }
        })

    }

    #[bench]
    fn bench_256(b: &mut Bencher){
        let mut csprng: OsRng = OsRng::new().unwrap();
        let kp = Keypair::generate::<Sha512, _>(&mut csprng);
        let data = [0;256];
        let sig = kp.sign::<Sha512>(&data);

        b.iter(||{
            for _ in 0..1000 {
                kp.verify::<Sha512>(&data, &sig).unwrap();
            }
        })

    }

    #[bench]
    fn bench_512(b: &mut Bencher){
        let mut csprng: OsRng = OsRng::new().unwrap();
        let kp = Keypair::generate::<Sha512, _>(&mut csprng);
        let data = [0;512];
        let sig = kp.sign::<Sha512>(&data);

        b.iter(||{
            for _ in 0..1000 {
                kp.verify::<Sha512>(&data, &sig).unwrap();
            }
        })

    }

    #[bench]
    fn bench_1024(b: &mut Bencher){
        let mut csprng: OsRng = OsRng::new().unwrap();
        let kp = Keypair::generate::<Sha512, _>(&mut csprng);
        let data = [0;1024];
        let sig = kp.sign::<Sha512>(&data);

        b.iter(||{
            for _ in 0..1000 {
                kp.verify::<Sha512>(&data, &sig).unwrap();
            }
        })

    }
}