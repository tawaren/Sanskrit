Module { byte_size: Some(117), meta: LargeVec([116, 101, 115, 116, 102, 97, 105, 108, 115, 119, 105, 116, 99, 104, 100, 111, 117, 98, 108, 101, 114, 101, 116, 117, 114, 110, 50]), data: [DataComponent { byte_size: Some(15), create_scope: Local, consume_scope: Local, inspect_scope: Local, top: false, provided_caps: CapSet(0), generics: [Physical(CapSet(0))], import: PublicImport { modules: [], types: [] }, body: Internal { constructors: [Case { fields: [TypeRef(0)] }, Case { fields: [] }] } }], sigs: [], data_sig_order: BitSerializedVec([true]), functions: [FunctionComponent { byte_size: Some(63), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(51))], import: PublicImport { modules: [], types: [Data { link: DataLink { module: ModRef(0), offset: 0 }, applies: [TypeRef(0)] }] }, params: [Param { consumes: true, typ: TypeRef(1) }, Param { consumes: false, typ: TypeRef(0) }], returns: [TypeRef(0), TypeRef(0)] }, body: Internal { byte_size: Some(42), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(1))] }, code: Exp(LargeVec([Switch(ValueRef(1), PermRef(0), [Exp(LargeVec([Return([ValueRef(0), ValueRef(0)])])), Exp(LargeVec([Move(ValueRef(0)), Copy(ValueRef(0)), Return([ValueRef(1), ValueRef(0)])]))]), Return([ValueRef(1), ValueRef(0)])])) } }], implements: [], fun_impl_order: BitSerializedVec([true]) }