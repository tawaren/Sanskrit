Module { byte_size: Some(80), meta: LargeVec([116, 101, 115, 116, 102, 97, 105, 108, 102, 117, 110, 112, 104, 97, 110, 116, 111, 109, 97, 112, 112, 108, 121]), data: [], sigs: [], data_sig_order: BitSerializedVec([]), functions: [FunctionComponent { byte_size: Some(18), scope: Global, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(0))], import: PublicImport { modules: [], types: [] }, params: [], returns: [] }, body: Internal { byte_size: Some(8), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [] }, code: Exp(LargeVec([Return([])])) } }, FunctionComponent { byte_size: Some(28), scope: Global, shared: FunSigShared { transactional: false, generics: [Phantom], import: PublicImport { modules: [], types: [] }, params: [], returns: [] }, body: Internal { byte_size: Some(19), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [Function { link: FuncLink { module: ModRef(0), offset: 0 }, applies: [TypeRef(0)] }], permissions: [Callable(PermSet(8), CallRef(0))] }, code: Exp(LargeVec([Invoke(PermRef(0), []), Return([])])) } }], implements: [], fun_impl_order: BitSerializedVec([true, true]) }