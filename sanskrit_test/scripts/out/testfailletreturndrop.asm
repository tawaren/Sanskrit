Module { byte_size: Some(66), meta: LargeVec([116, 101, 115, 116, 102, 97, 105, 108, 108, 101, 116, 114, 101, 116, 117, 114, 110, 100, 114, 111, 112]), data: [], sigs: [], data_sig_order: BitSerializedVec([]), functions: [FunctionComponent { byte_size: Some(34), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(49))], import: PublicImport { modules: [], types: [] }, params: [Param { consumes: true, typ: TypeRef(0) }], returns: [TypeRef(0)] }, body: Internal { byte_size: Some(21), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [] }, code: Exp(LargeVec([Let(Exp(LargeVec([Move(ValueRef(0)), Discard(ValueRef(0)), Move(ValueRef(0))]))), Move(ValueRef(0))])) } }], implements: [], fun_impl_order: BitSerializedVec([true]) }