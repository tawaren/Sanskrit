//! The compiler does two things:
//!     1: it generates descriptors (function & adt) which then can be uncalled at runtime
//!     2: it does some optimisations to function code, mainly removing unnecessary information and opcodes
//!
//! The results are stored and then can be invoked from the script interpreter at runtime.
//! If a function is invoked in that way it is executed over another interpreter
//! (See runtime for the two interpreters)
//!

use sanskrit_runtime::model::*;
use alloc::prelude::*;
use sanskrit_common::store::*;
use sanskrit_common::model::*;
use sanskrit_core::model::*;
use sanskrit_core::loader::StorageCache;
use sanskrit_core::resolver::Context;
use sanskrit_common::capabilities::CapSet;
use sanskrit_core::model::linking::Ref;
use compacting::Compactor;
use sanskrit_common::errors::*;
use sanskrit_core::model::resolved::ResolvedType;
use sanskrit_core::model::resolved::ResolvedApply;
use sanskrit_common::encoding::NoCustomAlloc;
use sanskrit_common::arena::*;
use sanskrit_common::encoding::ParserAllocator;

pub trait ComponentProcessor {
    fn process_adt(&mut self, offset:u8, a_desc:&AdtDescriptor) -> Result<()>;
    fn process_fun(&mut self, offset:u8, f_desc:&FunctionDescriptor) -> Result<()>;
}

//Entry point that compiles all types and public functions of a module
pub fn compile<'b, S:Store, P:ComponentProcessor>(module_hash:&Hash, store:&S, proc:&mut P) -> Result<()>{
    //load the module
    let module:Module = store.parsed_get(StorageClass::Module,module_hash, &NoCustomAlloc())?;
    let resolver = StorageCache::new_complete(store);

    let heap = Heap::new(10000,0,1.0);
    let mut alloc = heap.new_arena(10000)?;
    //generate descriptors for all adts
    for (offset,adt) in  module.adts.iter().enumerate() {
        //Prepare the context
        let context = Context::from_store_adt(adt, *module_hash, &resolver)?;
        //call the generator
        let adt = generate_adt_descriptor(adt,module_hash.clone(), offset as u8, &context, &alloc)?;
        proc.process_adt(offset as u8,&adt)?;
        alloc = alloc.reuse();
    }

    //generate descriptors for all functions
    for (offset,fun) in  module.functions.iter().enumerate() {
        if fun.visibility != Visibility::Private {
            //Prepare the context
            let context = Context::from_store_func(fun, *module_hash, &resolver)?;
            //call the generator
            proc.process_fun(offset as u8,&generate_function_descriptor(fun,module_hash, offset as u8, &context, &alloc)?)?;
            alloc = alloc.reuse()
        }
    }
    Ok(())
}

//generates a adt descriptor
fn generate_adt_descriptor<'b, 'h, S:Store>(adt:&AdtComponent, module:Hash, offset:u8, ctx:&Context<S>, alloc:&'b HeapArena<'h>) -> Result<AdtDescriptor<'b>> {
    //Collect infos about the generics
    let generics = alloc.iter_alloc_slice(adt.generics.iter().map(|g|match *g {
        Generic::Phantom => TypeTypeParam(true,CapSet::empty()),
        Generic::Physical(caps) => TypeTypeParam(false, caps),
    }))?;

    //collect the constructors and build type builders for their fields
    let mut constructors = alloc.slice_builder(adt.constructors.len())?;
    for ctr in &adt.constructors {
        //collect builders for each field
        let mut fields =  alloc.slice_builder(ctr.fields.len())?;
        for field in &ctr.fields {
            //build the builder
            fields.push( resolved_type_to_builder(&*field.fetch(ctx)?, alloc)?);
        }
        constructors.push(fields.finish());
    }

    //pack it all together in an adt descriptor
    Ok(AdtDescriptor {
        generics,
        constructors: constructors.finish(),
        base_caps: adt.provided_caps,
        id: AdtId::Custom(module,offset)
    })
}

//generates a function descriptor
fn generate_function_descriptor<'b,'h, S:Store>(fun:&FunctionComponent, module:&Hash, offset:u8, ctx:&Context<S>, alloc:&'b HeapArena<'h>) -> Result<FunctionDescriptor<'b>> {
    //collect the generics including protection information
    let generics = alloc.iter_alloc_slice(fun.generics.iter().enumerate().map(|(i,g)|{
        //get phantoms and caps
        let (is_phantom,caps) = match *g {
            Generic::Phantom => (true,CapSet::empty()),
            Generic::Physical(caps) => (false,caps),
        };
        //get protection info
        let is_protected = match fun.visibility {
            Visibility::Private => false,
            Visibility::Protected(ref guards) => guards.contains(&GenRef(i as u8)),
            Visibility::Public => false,
        };
        //pack the generic info
        FunTypeParam{is_protected, is_phantom, caps}
    }))?;

    //collect the params type builder
    let mut params = alloc.slice_builder(fun.params.len())?;
    for p in &fun.params{
        //build the builder
        let builder = alloc.alloc(resolved_type_to_builder(&*p.typ.fetch(ctx)?, alloc)?)?;
        params.push(Param(p.consumes, builder));
    }
    let params = params.finish();

    //collect the returns type builder
    let mut returns =  alloc.slice_builder(fun.returns.len())?;
    for r in &fun.returns{
        //build the builder
        let ret = alloc.alloc(resolved_type_to_builder(&*r.typ.fetch(ctx)?, alloc)?)?;
        returns.push(Return(ret, alloc.copy_alloc_slice(&r.borrows)?));
    }
    let returns = returns.finish();

    //Prepare the compactor to optimize the body
    let mut compactor = Compactor::new(alloc);
    //start the compaction process
    let (pos,args) = compactor.emit_func(fun,module,offset,ctx)?;
    assert_eq!(args as usize, returns.len());
    assert_eq!(pos, 0);

    //get all the compiled functions
    let functions = compactor.extract_functions()?;
    if functions.len() > u16::max_value() as usize {
        return size_limit_exceeded_error();
    }

    //pack it all together in an adt descriptor
    Ok(FunctionDescriptor{
        generics,
        params,
        returns,
        functions,
    })
}



//todo: find better pos
//Helper to generate a type builder from a type
pub fn resolved_type_to_builder<'b,'h>(typ:&ResolvedType, alloc:&'b HeapArena<'h>) -> Result<TypeBuilder<'b>> {
    //build an adt type
    fn build_type<'b, 'h>(caps:CapSet, kind:TypeKind, applies:&Vec<ResolvedApply>, alloc:&'b HeapArena<'h>) -> Result<TypeBuilder<'b>> {
        //builders for thy applies
        let mut builders = alloc.slice_builder(applies.len())?;
        for ResolvedApply{is_phantom,typ} in applies {
            //recursively process each apply
            let r_typ = resolved_type_to_builder(&*typ, alloc)?;
            //record it
            builders.push((*is_phantom,alloc.alloc(r_typ)?));
        }
        //put it together
        let builder = TypeBuilder::Dynamic(caps,kind, builders.finish());
        Ok(builder)
    }

    Ok(match *typ {
        //generics ned to fetch the type at runtime
        ResolvedType::Generic { offset, .. } => TypeBuilder::Ref(TypeInputRef(offset)),
        //Import & Natives can use the build_type
        ResolvedType::Import { base_caps, ref module, offset, ref applies, .. } => build_type(base_caps,TypeKind::Custom { module:module.to_hash(), offset },applies, alloc)?,
        ResolvedType::Native { base_caps, typ, ref applies , ..} => build_type(base_caps, TypeKind::Native { typ }, applies, alloc)?,
    })
}

