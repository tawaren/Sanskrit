Module { byte_size: Some(81), meta: LargeVec([116, 101, 115, 116, 102, 97, 105, 108, 117, 110, 112, 97, 99, 107, 116, 121, 112, 101, 55]), data: [DataComponent { byte_size: Some(16), create_scope: Local, consume_scope: Local, inspect_scope: Local, top: false, provided_caps: CapSet(0), generics: [Physical(CapSet(0))], import: PublicImport { modules: [], types: [] }, body: Internal { constructors: [Case { fields: [TypeRef(0)] }, Case { fields: [TypeRef(0)] }] } }], sigs: [], data_sig_order: BitSerializedVec([true]), functions: [FunctionComponent { byte_size: Some(34), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(48))], import: PublicImport { modules: [], types: [Data { link: DataLink { module: ModRef(0), offset: 0 }, applies: [TypeRef(0)] }] }, params: [Param { consumes: false, typ: TypeRef(1) }], returns: [TypeRef(0)] }, body: Internal { byte_size: Some(16), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(1))] }, code: Exp(LargeVec([Unpack(ValueRef(0), PermRef(0)), Move(ValueRef(0))])) } }], implements: [], fun_impl_order: BitSerializedVec([true]) }