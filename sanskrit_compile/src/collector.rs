use sanskrit_core::resolver::Context;
use alloc::vec::Vec;
use alloc::collections::BTreeSet;
use sp1_zkvm::lib::unconstrained;
use sanskrit_core::model::*;
use sanskrit_core::model::resolved::*;
use sanskrit_core::loader::{Loader, StateManager, NoUnconstraine};
use sanskrit_core::model::linking::{FastModuleLink, ImplementedComponent};
use sp1_zkvm::io::{hint, hint_slice, read, read_vec};
use sp1_zkvm_col::DefaultIndexType;

const FUNCTION_LIMIT:usize = u16::MAX as usize;

//Todo: Run this in unconstrained
//      Hook would be even better in the future
//          Without hook code still in binary

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CollectComponent {
    Function(FastModuleLink,u8),
    Implement(FastModuleLink,u8)
}

pub type CollectResult = Vec<(bool, DefaultIndexType, u8)>;

trait ToCollectComponent {
    fn collect_component(module:FastModuleLink,offset:u8) -> CollectComponent;
}

impl ToCollectComponent for FunctionComponent {
    fn collect_component(module:FastModuleLink,offset:u8) -> CollectComponent {
        CollectComponent::Function(module,offset)
    }
}

impl ToCollectComponent for ImplementComponent {
    fn collect_component(module:FastModuleLink,offset:u8) -> CollectComponent {
        CollectComponent::Implement(module,offset)
    }
}

pub enum Action {
    //Bool marks implements
    PushDependencies(CollectComponent),
    Record(CollectComponent)
}

pub struct Collector {
    //depth first search stack
    stack:Vec<Action>,
    //all the embedded functions and implement (the later marked over bool)
    recorded_funs:BTreeSet<CollectComponent>,
    //the ordered components
    components:Vec<CollectComponent>,
}

impl Collector {

    //Todo: Make a entry wrapper using unconstrained
    //      Can we panic if not uncconstrained

    pub fn collect<S:StateManager+NoUnconstraine>(fun:&FunctionComponent, store:&Loader<S>) -> CollectResult {
        //Todo: Find the nested system call and eliminate it
        //      We need a version of loader that does not use unconstrained
        unconstrained!{
            let u_store = store.no_unconstrained();
            let res = Self::collect_inner(fun,&u_store);
            let out = res.iter().map(|r|match r {
                CollectComponent::Function(module, offset) => (false, u_store.get_link_index(module), *offset),
                CollectComponent::Implement(module, offset) => (true, u_store.get_link_index(module), *offset)
            }).collect::<CollectResult>();
            let mut u8Out = Vec::with_capacity(out.len() * (2+size_of::<DefaultIndexType>()));
            for (imp, module, offset) in out {
                u8Out.push(if imp {1u8} else {0u8});
                //Todo: make Source type good enough so we can make this dynamic
                u8Out.push((module & 255) as u8);
                u8Out.push(((module >> 8) & 255) as u8);
                u8Out.push(offset);
            }
            //I hate that this is so big we need some quicker way
            hint_slice(&u8Out);
        }
        let raw = read_vec();
        let len = raw.len() / (2+size_of::<DefaultIndexType>());
        let mut res_vec = Vec::with_capacity(len);
        for i in 0..len {
            let start = i * 4;
            let b = raw[start] != 0;
            //Todo: make Source type good enough so we can make this dynamic
            let m1 = raw[start+1] as DefaultIndexType;
            let m2 = raw[start+2] as DefaultIndexType;
            let module: DefaultIndexType = m1 + (m2 << 8);
            let offset = raw[start+3];
            res_vec.push((b,module,offset))
        }
        res_vec
    }

    fn collect_inner<S:StateManager>(fun:&FunctionComponent, store:&Loader<S>) -> Vec<CollectComponent>{
        let mut col = Collector {
            stack: Vec::new(),
            recorded_funs: BTreeSet::new(),
            components: Vec::new()
        };
        let ctx = Context::from_top_component(fun,store);
        col.collect_dependencies(&ctx);
        while !col.stack.is_empty() {
            match col.stack.pop().unwrap() {
                Action::PushDependencies(comp) => {
                    //check if we already have processed
                    // Note: as the dependencies form a DAG it is guaranteed that we process each function only once
                    if col.recorded_funs.contains(&comp) { continue; }
                    match comp {
                        CollectComponent::Function(module, offset) => {
                            col.collect_record_module_dependencies::<S,FunctionComponent>(module, offset, store);
                        },
                        CollectComponent::Implement(module, offset) => {
                            col.collect_record_module_dependencies::<S,ImplementComponent>(module, offset, store);
                        }
                    };
                },
                Action::Record(result) => {
                    if col.recorded_funs.contains(&result) { continue; }
                    col.recorded_funs.insert(result);
                    //ensure we do not go over the limit
                    assert!(col.components.len() < u16::MAX as usize);
                    assert!(col.components.len() < FUNCTION_LIMIT);
                    col.components.push(result)
                }
            }
        }
        col.components
    }

    fn collect_record_module_dependencies<S:StateManager, C:ImplementedComponent+ToCollectComponent>(&mut self, module:FastModuleLink, offset:u8, store:&Loader<S>) {
        let comp = store.borrow_component::<C>(&module, offset);
        match comp.get_body() {
            CallableImpl::Internal{..} => {
                //if the function does not have an impact omit it (no returns & no risk will not change anything)
                if comp.get_returns().is_empty() && !comp.is_transactional(){ return; }
                //get the targets context
                let new_ctx = Context::from_module_component(comp, &module, true, store);
                self.stack.push(Action::Record(C::collect_component(module,offset)));
                self.collect_dependencies(&new_ctx)
            },
            //Only internal implements are required in the function graph
            _ => {}
        }
    }

    fn collect_dependencies<S:StateManager>(&mut self, ctx:&Context<S>) {      //push the post processing
        for perm in ctx.list_perms() {
            match **perm {
                ResolvedPermission::FunSig{fun, signature, ..} => {
                    match *fun {
                        ResolvedCallable::Function{ref base, ..} => {
                            if !signature.returns.is_empty() || signature.transactional {
                                self.stack.push(Action::PushDependencies(CollectComponent::Function(base.module, base.offset)))
                            }
                        },
                        ResolvedCallable::Implement{ref base, ..} => {
                            self.stack.push(Action::PushDependencies(CollectComponent::Implement(base.module, base.offset)))
                        }
                    }
                },
                _ => {}
            }
        }
    }
}
