Module { byte_size: Some(97), meta: LargeVec([116, 101, 115, 116, 102, 97, 105, 108, 99, 97, 112, 117, 110, 112, 97, 99, 107]), data: [DataComponent { byte_size: Some(14), create_scope: Local, consume_scope: Local, inspect_scope: Local, top: false, provided_caps: CapSet(0), generics: [Physical(CapSet(0))], import: PublicImport { modules: [], types: [] }, body: Internal { constructors: [Case { fields: [TypeRef(0)] }] } }], sigs: [], data_sig_order: BitSerializedVec([true]), functions: [FunctionComponent { byte_size: Some(54), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(48))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Data { link: DataLink { module: ModRef(1), offset: 2 }, applies: [TypeRef(0)] }] }, params: [Param { consumes: false, typ: TypeRef(1) }], returns: [TypeRef(0)] }, body: Internal { byte_size: Some(16), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(1))] }, code: Exp(LargeVec([Unpack(ValueRef(0), PermRef(0)), Move(ValueRef(0))])) } }], implements: [], fun_impl_order: BitSerializedVec([true]) }