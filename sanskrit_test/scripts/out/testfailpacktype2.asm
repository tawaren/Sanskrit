Module { byte_size: Some(125), meta: LargeVec([116, 101, 115, 116, 102, 97, 105, 108, 112, 97, 99, 107, 116, 121, 112, 101, 50]), data: [DataComponent { byte_size: Some(36), create_scope: Local, consume_scope: Local, inspect_scope: Local, top: false, provided_caps: CapSet(0), generics: [], import: PublicImport { modules: [Remote([50, 211, 51, 19, 133, 122, 33, 145, 191, 32, 241, 31, 70, 78, 4, 129, 152, 56, 197, 160])], types: [Data { link: DataLink { module: ModRef(1), offset: 0 }, applies: [] }] }, body: Internal { constructors: [Case { fields: [TypeRef(0)] }] } }], sigs: [], data_sig_order: BitSerializedVec([true]), functions: [FunctionComponent { byte_size: Some(60), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(48))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Data { link: DataLink { module: ModRef(0), offset: 0 }, applies: [] }, Data { link: DataLink { module: ModRef(1), offset: 1 }, applies: [TypeRef(0)] }] }, params: [Param { consumes: true, typ: TypeRef(0) }], returns: [TypeRef(2)] }, body: Internal { byte_size: Some(18), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(1), TypeRef(1))] }, code: Exp(LargeVec([Pack(PermRef(0), Tag(0), [ValueRef(0)]), Move(ValueRef(0))])) } }], implements: [], fun_impl_order: BitSerializedVec([true]) }