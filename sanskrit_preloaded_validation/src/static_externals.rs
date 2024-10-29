use sanskrit_common::model::Hash;
use sanskrit_default_externals::{_unsafe, data, eddsa, External, ids, iX, StaticExternalsProvider, uX};
use sp1_zkvm_col::{IdSelect, Seekable};

pub struct ExternalsV1;
pub struct ExtVec([(Hash, &'static dyn External);14]);

pub const EXT_VEC:ExtVec = ExtVec([
    ([188, 27, 240, 163, 24, 149, 173, 164, 202, 74, 116, 120, 186, 229, 143, 151, 37, 168, 27, 170],iX::EXT_I8),
    ([213, 207, 203, 128, 136, 108, 226, 166, 246, 196, 177, 236, 29, 53, 12, 97, 222, 87, 156, 51],iX::EXT_I16),
    ([159, 224, 239, 241, 186, 79, 57, 194, 146, 49, 109, 48, 201, 177, 203, 100, 89, 202, 235, 153],iX::EXT_I32),
    ([114, 109, 10, 154, 39, 113, 152, 131, 54, 204, 166, 238, 32, 131, 112, 55, 20, 85, 194, 97],iX::EXT_I64),
    ([8, 147, 227, 217, 48, 27, 216, 45, 241, 120, 72, 118, 69, 235, 111, 120, 110, 143, 100, 110],iX::EXT_I128),
    ([164, 10, 193, 1, 247, 225, 76, 251, 138, 10, 1, 185, 81, 182, 255, 77, 101, 192, 8, 224],uX::EXT_U8),
    ([36, 250, 207, 5, 164, 68, 203, 205, 71, 165, 232, 204, 125, 37, 100, 202, 85, 126, 167, 79],uX::EXT_U16),
    ([138, 236, 174, 129, 68, 121, 17, 117, 245, 67, 13, 101, 202, 51, 217, 240, 242, 180, 45, 225],uX::EXT_U32),
    ([65, 4, 195, 79, 237, 231, 99, 203, 184, 227, 245, 45, 116, 182, 218, 235, 39, 25, 240, 113],uX::EXT_U64),
    ([222, 144, 255, 97, 140, 73, 237, 173, 201, 149, 133, 214, 226, 91, 24, 231, 74, 165, 232, 250],uX::EXT_U128),
    ([62, 170, 107, 16, 10, 116, 57, 13, 18, 6, 103, 225, 153, 136, 95, 176, 184, 170, 84, 166],data::EXT_DATA),
    ([30, 0, 236, 118, 93, 188, 252, 209, 159, 180, 77, 117, 89, 151, 206, 18, 95, 42, 219, 119],ids::EXT_IDS),
    ([232, 171, 42, 13, 153, 47, 8, 240, 11, 16, 199, 117, 117, 144, 60, 179, 57, 25, 71, 212],eddsa::EXT_ECDSA),
    ([183, 52, 79, 151, 104, 198, 58, 254, 156, 108, 125, 137, 239, 125, 96, 35, 254, 64, 229, 108],_unsafe::EXT_UNSAFE),
]);

impl Seekable<Hash> for ExtVec {
    type I = (Hash, &'static dyn External);
    fn deref(inner: &Self::I) -> &Hash {&inner.0}
    fn with_store<R, F: FnOnce(&[Self::I]) -> R>(&self, f: F) -> R { f(&self.0) }
}

impl StaticExternalsProvider for ExternalsV1 {
    fn get(hash: &Hash) -> Option<&'static dyn External> {
        let idx = EXT_VEC.unconstrained_seek::<Hash,IdSelect>(hash);
        assert!(&EXT_VEC.0[idx].0 == hash);
        Some(EXT_VEC.0[idx].1)
    }
}