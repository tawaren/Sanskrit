Module { byte_size: Some(101), meta: LargeVec([116, 101, 115, 116, 102, 97, 105, 108, 115, 119, 105, 116, 99, 104, 100, 111, 117, 98, 108, 101, 100, 114, 111, 112, 50]), data: [DataComponent { byte_size: Some(15), create_scope: Local, consume_scope: Local, inspect_scope: Local, top: false, provided_caps: CapSet(0), generics: [Physical(CapSet(0))], import: PublicImport { modules: [], types: [] }, body: Internal { constructors: [Case { fields: [TypeRef(0)] }, Case { fields: [] }] } }], sigs: [], data_sig_order: BitSerializedVec([true]), functions: [FunctionComponent { byte_size: Some(49), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(3))], import: PublicImport { modules: [], types: [Data { link: DataLink { module: ModRef(0), offset: 0 }, applies: [TypeRef(0)] }] }, params: [Param { consumes: true, typ: TypeRef(1) }, Param { consumes: false, typ: TypeRef(0) }], returns: [] }, body: Internal { byte_size: Some(30), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(1))] }, code: Exp(LargeVec([Switch(ValueRef(1), PermRef(0), [Exp(LargeVec([Discard(ValueRef(0)), Discard(ValueRef(0)), Return([])])), Exp(LargeVec([Return([])]))]), Return([])])) } }], implements: [], fun_impl_order: BitSerializedVec([true]) }