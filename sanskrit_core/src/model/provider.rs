use alloc::borrow::ToOwned;
use sanskrit_common::encoding::{Parsable, Parser, Serializable, Serializer};
use sanskrit_common::errors::error;
use sanskrit_common::model::{Hash, ModuleLink};
use sanskrit_common::utils::store_hash;
use sp1_zkvm_col::arena::{UniqueArena, UniqueEmbeddableArena, URef};
use sp1_zkvm_col::map::HintMap;
use crate::loader::{StateManager, ResolvedCtrs};
use crate::model::linking::FastModuleLink;
use crate::model::{DataComponent, Module};
use crate::model::resolved::{ResolvedCallable, ResolvedComponent, ResolvedPermission, ResolvedSignature, ResolvedType};


pub struct DeDupHolder {
    pub link_dedup:UniqueArena<ModuleLink>,
    pub dedup_type:UniqueArena<ResolvedType>,
    pub dedup_data_type:UniqueEmbeddableArena<ResolvedComponent,ResolvedType>,
    pub dedup_call:UniqueArena<ResolvedCallable>,
    pub dedup_sig:UniqueArena<ResolvedSignature>,
    pub dedup_ctr:UniqueArena<ResolvedCtrs>,
    pub dedup_perm:UniqueArena<ResolvedPermission>
}

impl DeDupHolder {
    pub const fn new() -> Self {
        DeDupHolder {
            link_dedup:UniqueArena::new_unvalidated(),
            dedup_type:UniqueArena::new_unvalidated(),
            dedup_data_type:UniqueEmbeddableArena::new_unvalidated(),
            dedup_call:UniqueArena::new_unvalidated(),
            dedup_sig:UniqueArena::new_unvalidated(),
            dedup_ctr:UniqueArena::new_unvalidated(),
            dedup_perm:UniqueArena::new_unvalidated(),
        }
    }

    pub fn static_init(&self) {
        self.link_dedup.reserve_base_capacity();
        self.dedup_type.reserve_base_capacity();
        self.dedup_data_type.reserve_base_capacity();
        self.dedup_call.reserve_base_capacity();
        self.dedup_sig.reserve_base_capacity();
        self.dedup_ctr.reserve_base_capacity();
        self.dedup_perm.reserve_base_capacity();
    }

    pub fn validate(&self){
        self.link_dedup.validate();
        self.dedup_type.validate();
        self.dedup_data_type.validate();
        self.dedup_call.validate();
        self.dedup_sig.validate();
        self.dedup_ctr.validate();
        self.dedup_perm.validate();
    }
}

static mut DEDUP: DeDupHolder = DeDupHolder::new();
static mut EAGER_FAST_LINKS: HintMap<URef<'static, ModuleLink>, FastModuleLink> = HintMap::new_unvalidated();

//Todo: can we take in module link some or all time
pub fn parse_link(link:Hash) -> FastModuleLink {
    let mod_link = unsafe { DEDUP.link_dedup.alloc_unique(ModuleLink::new(link)) };
    return fast_link(mod_link)
}

pub fn fast_link(mod_link:URef<'static, ModuleLink>) -> FastModuleLink {
    //unsafe{dedup_count+=1}
    let res = unsafe {EAGER_FAST_LINKS.insert_if_missing(mod_link,|link|{
        //unsafe {dedup_miss_count+=1};
        unsafe {FastModuleLink::identity_leak(mod_link,None)}
    })}.to_owned();
    //unsafe {dedup_max = dedup_max.max(EAGER_FAST_LINKS.len())};
    res
}

impl Parsable for FastModuleLink {
    fn parse(p: &mut Parser) -> sanskrit_common::errors::Result<Self> {
        Ok(parse_link(Hash::parse(p)?))
    }
}

impl Serializable for FastModuleLink {
    fn serialize(&self, s: &mut Serializer) -> sanskrit_common::errors::Result<()> {
        self.get_module_link().serialize(s)
    }
}

#[derive(Copy, Clone)]
pub struct StaticProvider;

impl StaticProvider {
    pub fn new() -> Self {
        unsafe {DEDUP.static_init()}
        unsafe {EAGER_FAST_LINKS.reserve_default_capacity()}
        StaticProvider
    }

    pub fn add(&mut self, data:&[u8]) -> sanskrit_common::errors::Result<FastModuleLink> {
        let key = store_hash(&[data]);
        let fast_link = parse_link(key);
        if fast_link.get_cache().borrow().is_none() {
            let module: Module = Parser::parse_fully(data)?;
            let unique_module = unsafe { URef::identity_leak(module) };
            let _ = fast_link.get_cache().borrow_mut().insert(unique_module);
        }
        Ok(fast_link)
    }

    pub fn validate(self) {
        unsafe {DEDUP.validate()}
        unsafe {EAGER_FAST_LINKS.validate()}
    }
}


impl StateManager for StaticProvider {
    fn get_unique_module(&self, key: URef<'static, ModuleLink>) -> sanskrit_common::errors::Result<URef<'static, Module>> {
        //println!("cycle-tracker-report-start: fetch module");
        let fast_link = fast_link(key);
        let res = if fast_link.get_cache().borrow().is_some() {
            Ok(fast_link.get_cache().borrow().to_owned().unwrap())
        } else {
            error(||"Required module is missing")
        };
        //println!("cycle-tracker-report-end: fetch module");
        return res;
    }


    fn create_generic_type(&self, gen:ResolvedType) -> URef<'static,ResolvedType> {
        unsafe {URef::identity_leak(gen)}
    }

    fn sig_type_dedup(&self, sig:ResolvedType) -> URef<'static,ResolvedType> {
        unsafe {DEDUP.dedup_type.alloc_unique(sig)}
    }

    fn virtual_type_dedup(&self, virt:ResolvedType) -> URef<'static,ResolvedType> {
        unsafe {DEDUP.dedup_type.alloc_unique(virt)}
    }

    fn projection_type_dedup(&self, proj:ResolvedType) -> URef<'static,ResolvedType> {
        unsafe {DEDUP.dedup_type.alloc_unique(proj)}
    }

    fn data_type_dedup(&self, param:ResolvedComponent, extra:&DataComponent) -> URef<'static,ResolvedType> {
        unsafe {DEDUP.dedup_data_type.alloc_unique(param, extra)}
    }

    fn dedup_callable(&self, call:ResolvedCallable) -> URef<'static,ResolvedCallable> {
        unsafe {DEDUP.dedup_call.alloc_unique(call)}
    }

    fn dedup_permission(&self, perm:ResolvedPermission) -> URef<'static,ResolvedPermission> {
        unsafe {DEDUP.dedup_perm.alloc_unique(perm)}
    }

    fn dedup_signature(&self, sig:ResolvedSignature) -> URef<'static,ResolvedSignature> {
        unsafe {DEDUP.dedup_sig.alloc_unique(sig)}
    }

    fn dedup_ctr(&self, ctr:ResolvedCtrs) -> URef<'static,ResolvedCtrs> {
        unsafe {DEDUP.dedup_ctr.alloc_unique(ctr)}
    }
}