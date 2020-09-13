use sanskrit_core::resolver::Context;
use alloc::vec::Vec;
use alloc::collections::BTreeSet;
use sanskrit_core::model::*;
use sanskrit_core::model::resolved::*;
use sanskrit_common::errors::*;
use sanskrit_common::model::*;
use sanskrit_common::store::*;
use sanskrit_core::utils::Crc;
use sanskrit_core::loader::{Loader, FetchCache};
use limiter::Limiter;

pub enum Action {
    PushDependencies(Crc<ModuleLink>,u8),
    Record(FetchCache<FunctionComponent>)
}

pub struct Collector {
    //depth first search sack
    stack:Vec<Action>,
    //all the embedded functions and where to find them at runtime
    recorded_funs:BTreeSet<(Crc<ModuleLink>,u8)>,
    //the ordered functions
    functions:Vec<FetchCache<FunctionComponent>>,

}

impl Collector {
    pub fn collect<S:Store>(fun:&FunctionComponent, store:&Loader<S>, limiter:&Limiter) -> Result<Vec<FetchCache<FunctionComponent>>>{
        let mut col = Collector {
            stack: Vec::new(),
            recorded_funs: BTreeSet::new(),
            functions: Vec::new()
        };
        let ctx = Context::from_top_component(fun,store)?;
        col.collect_fun_dependencies(&ctx,store)?;
        while !col.stack.is_empty() {
            match col.stack.pop().unwrap() {
                Action::PushDependencies(module, offset) => {
                    //check if we already have processed
                    // Note: as the dependencies form a DAG it is guaranteed that we process each function only once
                    let key = (module, offset);
                    if col.recorded_funs.contains(&key) { continue; }
                    col.collect_record_module_fun_dependencies(key.0,key.1,store)?;
                },
                Action::Record(fun_cache) => {
                    let key = (fun_cache.module().clone(), fun_cache.offset());
                    if col.recorded_funs.contains(&key) { continue; }
                    col.recorded_funs.insert(key.clone());
                    //ensure we do not go over the limit
                    if col.functions.len() >= u16::max_value() as usize {return error(||"Number of functions out of range")}
                    if col.functions.len() >= limiter.max_functions {return error(||"Size limit exceeded")}
                    col.functions.push(fun_cache)
                }
            }
        }
        limiter.produced_functions.set(col.functions.len());
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
                self.stack.push(Action::Record(fun_cache));
                self.collect_fun_dependencies(&new_ctx,store)
            },
            //Only internal functions are required in the function graph
             _ => Ok(())
        }

    }

    fn collect_fun_dependencies<S:Store>(&mut self, ctx:&Context<S>, store:&Loader<S>) -> Result<()> {      //push the post processing
        for perm in ctx.list_perms() {
            match **perm {
                ResolvedPermission::FunSig{ref fun, ref signature, ..} => {
                    match **fun {
                        ResolvedCallable::Function{ref module, offset, ..}
                        | ResolvedCallable::Implement{ref module, offset, ..} => {
                            if !signature.returns.is_empty() || signature.transactional{
                                self.stack.push(Action::PushDependencies(module.clone(), offset))
                            }
                        }
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }

}
