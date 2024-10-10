use sanskrit_core::resolver::Context;
use alloc::vec::Vec;
use alloc::collections::BTreeSet;
use sanskrit_core::model::*;
use sanskrit_core::model::resolved::*;
use sanskrit_common::errors::*;
use sanskrit_common::model::*;
use sanskrit_common::store::*;
use sanskrit_common::utils::Crc;
use sanskrit_core::loader::{Loader, FetchCache};

const FUNCTION_LIMIT:usize = u16::MAX as usize;

pub enum CollectResult {
    Function(FetchCache<FunctionComponent>),
    Implement(FetchCache<ImplementComponent>)
}

pub enum Action {
    //Bool marks implements
    PushDependencies(Crc<ModuleLink>,u8, bool),
    Record(CollectResult)
}

pub struct Collector {
    //depth first search stack
    stack:Vec<Action>,
    //all the embedded functions and implement (the later marked over bool)
    recorded_funs:BTreeSet<(Crc<ModuleLink>,u8, bool)>,
    //the ordered functions
    functions:Vec<CollectResult>,
}

impl Collector {
    pub fn collect<S:Store>(fun:&FunctionComponent, store:&Loader<S>) -> Result<Vec<CollectResult>>{
        let mut col = Collector {
            stack: Vec::new(),
            recorded_funs: BTreeSet::new(),
            functions: Vec::new()
        };
        let ctx = Context::from_top_component(fun,store)?;
        col.collect_dependencies(&ctx)?;
        while !col.stack.is_empty() {
            match col.stack.pop().unwrap() {
                Action::PushDependencies(module, offset, is_implement) => {
                    //check if we already have processed
                    // Note: as the dependencies form a DAG it is guaranteed that we process each function only once
                    let key = (module, offset, is_implement);
                    if col.recorded_funs.contains(&key) { continue; }
                    if is_implement {
                        col.collect_record_module_impl_dependencies(key.0,key.1,store)?;
                    } else {
                        col.collect_record_module_fun_dependencies(key.0,key.1,store)?;
                    }
                },
                Action::Record(result) => {
                    let key = match &result {
                        CollectResult::Function(fun_cache) => (fun_cache.module().clone(), fun_cache.offset(), false),
                        CollectResult::Implement(impl_cache) => (impl_cache.module().clone(), impl_cache.offset(), true)
                    };

                    if col.recorded_funs.contains(&key) { continue; }
                    col.recorded_funs.insert(key.clone());
                    //ensure we do not go over the limit
                    if col.functions.len() >= u16::MAX as usize {return error(||"Number of functions out of range")}
                    if col.functions.len() >= FUNCTION_LIMIT {return error(||"Size limit exceeded")}
                    col.functions.push(result)
                }
            }
        }
        Ok(col.functions)
    }

    fn collect_record_module_fun_dependencies<S:Store>(&mut self, module:Crc<ModuleLink>, offset:u8, store:&Loader<S>) -> Result<()> {
        let fun_cache = store.get_component::<FunctionComponent>(&module, offset)?;
        let fun_comp = fun_cache.retrieve();
        match fun_comp.body {
            CallableImpl::Internal{..} => {
                //if the function does not have an impact omit it (no returns & no risk will not change anything)
                if fun_comp.shared.returns.is_empty() && !fun_comp.shared.transactional{ return Ok(()); }
                //get the targets context
                let new_ctx = Context::from_module_component(fun_comp, &module, true, store)?;
                self.stack.push(Action::Record(CollectResult::Function(fun_cache)));
                self.collect_dependencies(&new_ctx)
            },
            //Only internal implements are required in the function graph
             _ => Ok(())
        }
    }

    fn collect_record_module_impl_dependencies<S:Store>(&mut self, module:Crc<ModuleLink>, offset:u8, store:&Loader<S>) -> Result<()> {
        let impl_cache = store.get_component::<ImplementComponent>(&module, offset)?;
        let impl_comp = impl_cache.retrieve();
        match impl_comp.body {
            CallableImpl::Internal{..} => {
                //get the targets context
                let new_ctx = Context::from_module_component(impl_comp, &module, true, store)?;
                self.stack.push(Action::Record(CollectResult::Implement(impl_cache)));
                self.collect_dependencies(&new_ctx)
            },
            //Only internal functions are required in the function graph
            _ => Ok(())
        }
    }

    fn collect_dependencies<S:Store>(&mut self, ctx:&Context<S>) -> Result<()> {      //push the post processing
        for perm in ctx.list_perms() {
            match **perm {
                ResolvedPermission::FunSig{ref fun, ref signature, ..} => {
                    match **fun {
                        ResolvedCallable::Function{ref module, offset, ..} => {
                            if !signature.returns.is_empty() || signature.transactional{
                                self.stack.push(Action::PushDependencies(module.clone(), offset, false))
                            }
                        },
                        ResolvedCallable::Implement{ref module, offset, ..} => {
                            self.stack.push(Action::PushDependencies(module.clone(), offset, true))
                        }
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }

}
