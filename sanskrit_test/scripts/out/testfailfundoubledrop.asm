Module { byte_size: Some(61), meta: LargeVec([116, 101, 115, 116, 102, 97, 105, 108, 102, 117, 110, 100, 111, 117, 98, 108, 101, 100, 114, 111, 112]), data: [], sigs: [], data_sig_order: BitSerializedVec([]), functions: [FunctionComponent { byte_size: Some(29), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(1))], import: PublicImport { modules: [], types: [] }, params: [Param { consumes: true, typ: TypeRef(0) }], returns: [] }, body: Internal { byte_size: Some(17), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [] }, code: Exp(LargeVec([Move(ValueRef(0)), Discard(ValueRef(0)), Discard(ValueRef(0)), Return([])])) } }], implements: [], fun_impl_order: BitSerializedVec([true]) }