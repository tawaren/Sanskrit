Module { byte_size: Some(111), meta: LargeVec([116, 101, 115, 116, 102, 97, 105, 108, 102, 117, 110, 99, 97, 108, 108, 51]), data: [], sigs: [], data_sig_order: BitSerializedVec([]), functions: [FunctionComponent { byte_size: Some(25), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(48))], import: PublicImport { modules: [], types: [] }, params: [Param { consumes: true, typ: TypeRef(0) }], returns: [TypeRef(0)] }, body: Internal { byte_size: Some(12), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [] }, code: Exp(LargeVec([Move(ValueRef(0)), Move(ValueRef(0))])) } }, FunctionComponent { byte_size: Some(59), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(48))], import: PublicImport { modules: [Remote([50, 211, 51, 19, 133, 122, 33, 145, 191, 32, 241, 31, 70, 78, 4, 129, 152, 56, 197, 160])], types: [Data { link: DataLink { module: ModRef(1), offset: 0 }, applies: [] }] }, params: [Param { consumes: true, typ: TypeRef(0) }], returns: [TypeRef(0)] }, body: Internal { byte_size: Some(22), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [Function { link: FuncLink { module: ModRef(0), offset: 0 }, applies: [TypeRef(1)] }], permissions: [Callable(PermSet(8), CallRef(0))] }, code: Exp(LargeVec([Invoke(PermRef(0), [ValueRef(0)]), Move(ValueRef(0))])) } }], implements: [], fun_impl_order: BitSerializedVec([true, true]) }