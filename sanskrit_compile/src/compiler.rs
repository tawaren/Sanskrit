//! The compiler does two things:
//!     1: it generates transaction descriptors which then can be called at runtime
//!     2: it does some optimisations to function sys, mainly removing unnecessary information and opcodes
//!
//! The results are stored and then can be invoked from runtime.
//!

use alloc::boxed::Box;
use alloc::vec::Vec;
use sanskrit_chain_code::model::*;
use sanskrit_common::model::*;
use sanskrit_core::model::Exp as SExp;
use sanskrit_core::model::*;
use sanskrit_core::loader::Loader;
use sanskrit_core::resolver::Context;
use sanskrit_core::model::linking::Ref;
use sanskrit_core::model::bitsets::*;
use crate::compacting::Compactor;
use sanskrit_core::model::resolved::ResolvedType;
use sanskrit_common::utils::Crc;
use crate::externals::CompilationExternals;

//Entry point that compiles all types and public functions of a module
pub fn compile_transaction<S:Store, CE:CompilationExternals>(transaction_hash:&Hash, store:&CachedStore<Module,S>) -> Result<TransactionDescriptor>{

    //load the module
    let fun:FunctionComponent = store.parsed_get(StorageClass::Transaction, transaction_hash)?;
    let resolver = Loader::new_transaction(store);

    //generate descriptors for all internal functions
    if fun.scope != Accessibility::Global {
        return error(||"Transactions must have the public call permission")
    } else {
        match fun.body {
            CallableImpl::External => error(||"External functions can not be used as transactions"),
            CallableImpl::Internal { ref code, .. } => {
                //Prepare the context
                let context = Context::from_top_component(&fun, &resolver)?;
                //call the generator
                generate_transaction_descriptor::<_,CE>(&fun, code, &context)
            },
        }
    }

}

//generates a function descriptor
fn generate_transaction_descriptor<S:Store,CE:CompilationExternals>(fun:&FunctionComponent, code:&SExp, ctx:&Context<S>) -> Result<TransactionDescriptor> {
    
    //collect the params type builder
    let mut params = Vec::with_capacity(fun.shared.params.len());
    for p in &fun.shared.params{
        let typ = p.typ.fetch(ctx)?;
        //build the type & desc
        let r_typ = Box::new(resolved_to_runtime_type(&*typ)?);
        let desc = Box::new(resolved_to_value_descriptor::<_,CE>(&*typ, ctx)?);
        params.push(TxTParam{
            primitive: typ.get_caps().contains(Capability::Primitive),
            copy:typ.get_caps().contains(Capability::Copy),
            drop:typ.get_caps().contains(Capability::Drop),
            consumes:p.consumes,
            typ:r_typ,
            desc
        });
    }

    //collect the returns type builder
    let mut returns =  Vec::with_capacity(fun.shared.returns.len());
    for r in &fun.shared.returns{
        let typ = r.fetch(ctx)?;

        //build the typ
        let r_typ = Box::new(resolved_to_runtime_type(&*typ)?);
        let desc = Box::new(resolved_to_value_descriptor::<_,CE>(&*typ, ctx)?);
        returns.push(TxTReturn{
            primitive: typ.get_caps().contains(Capability::Primitive),
            copy: typ.get_caps().contains(Capability::Copy),
            drop: typ.get_caps().contains(Capability::Drop),
            typ:r_typ,
            desc
        });
    }
    //do the compaction process
    let functions = Compactor::compact::<_,CE>(fun, code,  &ctx.store)?;

    if functions.len() > u16::MAX as usize {
        return error(||"Number of functions out of range")
    }

    let desc = TransactionDescriptor {
        byte_size: None,
        params,
        returns,
        functions
    };

    //pack it all together in a function descriptor
    Ok(desc)
}


pub fn resolved_to_runtime_type(typ:&ResolvedType) -> Result<RuntimeType> {
    //build an adt type
    fn build_type(module:Hash, offset:u8, applies:&[Crc<ResolvedType>]) -> Result<RuntimeType> {
        //builders for thy applies
        let mut builders = Vec::with_capacity(applies.len());
        for typ in applies {
            //recursively process each apply
            let r_typ = resolved_to_runtime_type(&*typ)?;
            //record it
            builders.push(r_typ);
        }
        Ok(RuntimeType::Custom {
            module,
            offset,
            applies: builders
        })
    }
    
    Ok(match *typ {
        //transactions have no generics
        ResolvedType::Generic { .. } => unreachable!() ,
        //transactions can not take or return sigs
        //it is unreachable as transaction params and returns are limited to top types or primitives
        // Sig itself is neither & top wrappers require persist which sig has not
        // If in the future a top type witch does allow a inner Sig is introduced this needs implementation (which is impossible without changing the runtime completely)
        ResolvedType::Sig {..} => unreachable!(),
        ResolvedType::Projection { depth, ref un_projected } => {
            let inner = resolved_to_runtime_type(&**un_projected)?;
            RuntimeType::Projection {
                depth,
                typ:Box::new(inner)
            }
        },
        ResolvedType::Virtual(hash) => RuntimeType::Virtual { id:hash },
        ResolvedType::Lit { ref module, offset, ref applies, .. }
        | ResolvedType::Data { ref module, offset, ref applies, .. } => build_type(module.to_hash(), offset, applies)?,
    })
}



pub fn resolved_to_value_descriptor<S:Store, CE:CompilationExternals>(typ:&ResolvedType, ctx:&Context<S>) -> Result<ValueSchema> {
    //build an adt type
    fn build_adt_checker<S:Store, CE:CompilationExternals>(module:Crc<ModuleLink>, offset:u8, applies:&[Crc<ResolvedType>], ctx:&Context<S>) -> Result<ValueSchema> {
         //Get the cache
        let adt_cache = ctx.store.get_component::<DataComponent>(&module, offset)?;
        //Get the adt
        let adt = adt_cache.retrieve();

        Ok(match adt.body {
            DataImpl::External(size) => CE::get_literal_checker(&*module, offset, size)?,
            DataImpl::Internal { ref constructors} => {
                //get its context with the applies as substitutions
                let context = adt_cache.substituted_context(&applies,ctx.store)?;
                //handle special case
                if constructors.len() == 1 && constructors[0].fields.len() == 1 {
                    //Wrapper Optimization
                    let f_typ = constructors[0].fields[0].typ.fetch(&context)?;
                    resolved_to_value_descriptor::<_,CE>(&f_typ, &context)?
                } else {
                    let mut index_mod = None;

                    //normal case
                    let mut casees = Vec::with_capacity(constructors.len());
                    //build the ctrs by retriving their fields
                    for case in constructors {
                        let mut fields = Vec::with_capacity(case.fields.len());
                        for field in &case.fields {
                            let field_typ = field.typ.fetch(&context)?;
                            if !field.indexed.is_empty() && index_mod.is_none() {
                                index_mod = Some(Box::new((module.to_hash(),offset)))
                            }
                            let index = field.indexed.clone();
                            fields.push((index,Box::new(resolved_to_value_descriptor::<_,CE>(&field_typ, &context)?)))
                        }
                        casees.push(fields);
                    }
                    ValueSchema::Adt(index_mod, casees)
                }
            }
        })
    }

    Ok(match *typ {
        //transactions have no generics
        ResolvedType::Generic {  .. } => unreachable!(),
        //Virtuals never have instances of them
        ResolvedType::Virtual(_) => unreachable!(),
        //sigs are never primitives
        ResolvedType::Sig {..} => unreachable!(),
        //images have the same repr as the inner
        ResolvedType::Projection { ref un_projected, .. } => resolved_to_value_descriptor::<_,CE>(&**un_projected, ctx)?,
        ResolvedType::Lit { ref module, offset, ref applies, .. }
        | ResolvedType::Data { ref module, offset, ref applies, .. } => build_adt_checker::<_,CE>(module.clone(), offset, applies, ctx)?,
    })
}
