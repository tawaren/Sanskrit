Module { byte_size: Some(83), meta: LargeVec([116, 101, 115, 116, 102, 97, 105, 108, 102, 114, 101, 101, 117, 110, 99, 111, 110, 115, 117, 109, 101, 100, 50]), data: [DataComponent { byte_size: Some(14), create_scope: Local, consume_scope: Local, inspect_scope: Local, top: false, provided_caps: CapSet(0), generics: [Physical(CapSet(0))], import: PublicImport { modules: [], types: [] }, body: Internal { constructors: [Case { fields: [TypeRef(0)] }] } }], sigs: [], data_sig_order: BitSerializedVec([true]), functions: [FunctionComponent { byte_size: Some(34), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(0))], import: PublicImport { modules: [], types: [Data { link: DataLink { module: ModRef(0), offset: 0 }, applies: [TypeRef(0)] }] }, params: [Param { consumes: true, typ: TypeRef(0) }], returns: [] }, body: Internal { byte_size: Some(17), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(1), TypeRef(1))] }, code: Exp(LargeVec([Pack(PermRef(0), Tag(0), [ValueRef(0)]), Return([])])) } }], implements: [], fun_impl_order: BitSerializedVec([true]) }