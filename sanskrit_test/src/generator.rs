use sanskrit_core::model::{Module, BitSerializedVec, FunctionComponent, FunSigShared, Accessibility, PublicImport, CallableImpl, BodyImport, Capability, Param, TypeRef, Exp, Generic, TypeImport, DataLink, ModRef, CallableImport, FuncLink, PermissionImport, Permission, CallRef, PermRef, OpCode, DataComponent, DataImpl};
use sanskrit_common::encoding::Serializer;
use sanskrit_common::model::{LargeVec, ValueRef, Hash, ModuleLink};
use sanskrit_core::model::bitsets::{CapSet, BitSet, PermSet};
use sanskrit_common::store::store_hash;

pub fn generate(n:u8, reuse:bool, overfill:bool, body:bool) -> Vec<Vec<u8>> {
    let mut modules = Vec::with_capacity((n as usize)+1);
    let mut hashes = Vec::with_capacity(n as usize);
    if !reuse {
        for no in 0..n {
            let module_data = generate_function_module(n, no, reuse, overfill);
            hashes.push(store_hash(&[&module_data]));
            modules.push(module_data)
        }
    } else {
        let moidule_data = generate_function_module(n,0, reuse, overfill);
        hashes.push(store_hash(&[&moidule_data]));
        modules.push(moidule_data)
    }


    modules.push(generate_master_module(n, hashes, reuse, body));
    modules
}

fn generate_function_module(n:u8, no:u8, reuse:bool, overfill:bool) -> Vec<u8> {
    let mut functions = Vec::with_capacity(n as usize);
    if overfill {
        for _ in 0..255 {
            functions.push(generate_import_function(n))
        }
    } else {
        if reuse {
            functions.push(generate_import_function(n))
        } else {
            for _ in 0..n {
                functions.push(generate_import_function(n))
            }
        }
    }

    let module = Module {
        byte_size: None,
        meta: LargeVec( Serializer::serialize_fully(&no, 1).unwrap()),
        data: vec![],
        sigs: vec![],
        data_sig_order: BitSerializedVec(vec![]),
        functions,
        implements: vec![],
        fun_impl_order: BitSerializedVec(vec![true;if reuse {1}  else {n as usize}])
    };

    return Serializer::serialize_fully(&module, u16::max_value() as usize).unwrap()
}

fn generate_master_module(n:u8, imports:Vec<Hash>, reuse:bool, body:bool) ->  Vec<u8> {
    let mut functions = Vec::with_capacity(n as usize);
    let mut data = Vec::with_capacity(n as usize);

    for no in 0..n {
        data.push(generate_main_data(n));
        functions.push(generate_main_function(n, no, imports.clone(), reuse, body))
    }


    let module = Module {
        byte_size: None,
        meta: LargeVec( vec![]),
        data,
        sigs: vec![],
        data_sig_order: BitSerializedVec(vec![true;n as usize]),
        functions,
        implements: vec![],
        fun_impl_order: BitSerializedVec(vec![true;n as usize])
    };

    return Serializer::serialize_fully(&module, u16::max_value() as usize).unwrap()

}

fn generate_import_function(n:u8) -> FunctionComponent {
    let mut generics =  Vec::with_capacity(n as usize);
    let mut params =  Vec::with_capacity(n as usize);
    let mut returns = Vec::with_capacity(n as usize);
    let mut codes = Vec::with_capacity(n as usize);

    for offset in 0..n {
        generics.push(Generic::Physical(CapSet::from_entry(Capability::Unbound)));
        params.push(Param{consumes: true,typ: TypeRef(offset)});
        returns.push(TypeRef(offset));
        codes.push(OpCode::Move(ValueRef((n-1) as u16)))
    }

    FunctionComponent {
        byte_size: None,
        shared: FunSigShared {
            generics,
            import: empty_import(),
            transactional: false,
            params,
            returns
        },
        scope: Accessibility::Global,
        body: CallableImpl::Internal {
            byte_size: None,
            imports: empty_body_import(),
            code: Exp(LargeVec(codes)),
        }
    }
}


fn generate_main_function(n:u8, no:u8, imports:Vec<Hash>, reuse:bool, body:bool) -> FunctionComponent {
    let mut generics =  Vec::with_capacity(n as usize);
    let mut params =  Vec::with_capacity(n as usize);
    let mut returns = Vec::with_capacity(n as usize);
    let mut codes = Vec::with_capacity(n as usize);
    let mut value_fetch = Vec::with_capacity(n as usize);

    if body {
        for offset in 0..n {
            value_fetch.push(ValueRef((n - offset - 1) as u16))
        }
    }

    for offset in 0..n {
        generics.push(Generic::Physical(CapSet::from_entry(Capability::Unbound)));
        if body {
            params.push(Param{consumes: true,typ: TypeRef(n+offset)});
            returns.push(TypeRef(n+offset));
            if reuse {
                codes.push(OpCode::Invoke(PermRef(0), value_fetch.clone()))
            } else {
                codes.push(OpCode::Invoke(PermRef(offset), value_fetch.clone()))
            }
        };
    }

    FunctionComponent {
        byte_size: None,
        shared: FunSigShared {
            generics,
            import: full_import(n),
            transactional: false,
            params,
            returns
        },
        scope: Accessibility::Global,
        body: CallableImpl::Internal {
            byte_size: None,
            imports: full_body_import(n, no, imports, reuse),
            code: Exp(LargeVec(codes)),
        }
    }
}

fn generate_main_data(n:u8) -> DataComponent {
    let mut generics =  Vec::with_capacity(n as usize);
    for _ in 0..n {
        generics.push(Generic::Physical(CapSet::empty()));
    }

    DataComponent {
        byte_size: None,
        create_scope: Accessibility::Global,
        consume_scope: Accessibility::Global,
        inspect_scope: Accessibility::Global,
        provided_caps: CapSet::all(),
        top:false,
        generics,
        import: empty_import(),
        body: DataImpl::Internal {
            constructors: vec![]
        }
    }
}

fn empty_import() -> PublicImport {
    PublicImport {
        modules: vec![],
        types: vec![]
    }
}

fn empty_body_import() -> BodyImport {
    BodyImport {
        public: empty_import(),
        callables: vec![],
        permissions: vec![]
    }
}

fn full_import(n:u8) -> PublicImport {
    let type_applies = (0..n).map(TypeRef).collect::<Vec<_>>();
    let mut types = Vec::with_capacity(n as usize);

    for offset in 0..n {
        types.push(TypeImport::Data {
            link: DataLink {
                module: ModRef(0),
                offset
            },
            applies: type_applies.clone()
        });
    }

    PublicImport {
        modules: vec![],
        types
    }
}

fn full_body_import(n:u8, no:u8, modules:Vec<Hash>, reuse:bool) -> BodyImport {
    let n16 = n as u16;
    let modules = modules.into_iter().map(ModuleLink::Remote).collect();
    let call_applies = (n16..(2*n16)).map(|n|n as u8).map(TypeRef).collect::<Vec<_>>();
    let mut callables = Vec::with_capacity(n as usize);
    let mut permissions = Vec::with_capacity(n as usize);

    if reuse {
        callables.push(CallableImport::Function {
            link: FuncLink {
                module: ModRef(1),
                offset: 0
            },
            applies: call_applies.clone()
        });

        permissions.push(PermissionImport::Callable(
            PermSet::from_entry(Permission::Call),
            CallRef(0)
        ));
    } else {
        for offset in 0..n {
            callables.push(CallableImport::Function {
                link: FuncLink {
                    module: ModRef(1+offset),
                    offset: no
                },
                applies: call_applies.clone()
            });

            permissions.push(PermissionImport::Callable(
                PermSet::from_entry(Permission::Call),
                CallRef(offset)
            ));
        }
    }



    BodyImport {
        public: PublicImport {
            modules,
            types: vec![]
        },
        callables,
        permissions
    }
}