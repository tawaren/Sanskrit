Module { byte_size: Some(1170), meta: LargeVec([116, 101, 115, 116, 115, 117, 99, 116, 121, 112, 101, 99, 104, 101, 99, 107]), data: [DataComponent { byte_size: Some(17), create_scope: Local, consume_scope: Local, inspect_scope: Local, top: false, provided_caps: CapSet(32), generics: [Physical(CapSet(0)), Physical(CapSet(0))], import: PublicImport { modules: [], types: [] }, body: Internal { constructors: [Case { fields: [TypeRef(0), TypeRef(1)] }] } }], sigs: [], data_sig_order: BitSerializedVec([true]), functions: [FunctionComponent { byte_size: Some(61), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(32))], import: PublicImport { modules: [Remote([50, 211, 51, 19, 133, 122, 33, 145, 191, 32, 241, 31, 70, 78, 4, 129, 152, 56, 197, 160])], types: [Data { link: DataLink { module: ModRef(1), offset: 0 }, applies: [] }] }, params: [Param { consumes: true, typ: TypeRef(0) }], returns: [TypeRef(0), TypeRef(1)] }, body: Internal { byte_size: Some(23), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(1), TypeRef(1))] }, code: Exp(LargeVec([Move(ValueRef(0)), Lit(LargeVec([1]), PermRef(0)), Return([ValueRef(1), ValueRef(0)])])) } }, FunctionComponent { byte_size: Some(52), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(1))], import: PublicImport { modules: [Remote([114, 32, 241, 132, 79, 189, 46, 224, 165, 231, 180, 28, 243, 44, 149, 65, 98, 138, 130, 183])], types: [Data { link: DataLink { module: ModRef(1), offset: 13 }, applies: [] }] }, params: [Param { consumes: true, typ: TypeRef(0) }, Param { consumes: true, typ: TypeRef(1) }], returns: [] }, body: Internal { byte_size: Some(14), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [] }, code: Exp(LargeVec([DiscardMany([ValueRef(1), ValueRef(0)]), Return([])])) } }, FunctionComponent { byte_size: Some(58), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(1))], import: PublicImport { modules: [Remote([114, 32, 241, 132, 79, 189, 46, 224, 165, 231, 180, 28, 243, 44, 149, 65, 98, 138, 130, 183])], types: [Data { link: DataLink { module: ModRef(1), offset: 13 }, applies: [] }] }, params: [Param { consumes: true, typ: TypeRef(0) }, Param { consumes: true, typ: TypeRef(1) }], returns: [] }, body: Internal { byte_size: Some(20), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [] }, code: Exp(LargeVec([Move(ValueRef(1)), Move(ValueRef(1)), DiscardMany([ValueRef(1), ValueRef(0)]), Return([])])) } }, FunctionComponent { byte_size: Some(58), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(50))], import: PublicImport { modules: [Remote([114, 32, 241, 132, 79, 189, 46, 224, 165, 231, 180, 28, 243, 44, 149, 65, 98, 138, 130, 183])], types: [Data { link: DataLink { module: ModRef(1), offset: 11 }, applies: [] }] }, params: [Param { consumes: false, typ: TypeRef(0) }, Param { consumes: false, typ: TypeRef(1) }], returns: [TypeRef(0), TypeRef(1)] }, body: Internal { byte_size: Some(18), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [] }, code: Exp(LargeVec([Copy(ValueRef(1)), Copy(ValueRef(1)), Return([ValueRef(1), ValueRef(0)])])) } }, FunctionComponent { byte_size: Some(51), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(48))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Data { link: DataLink { module: ModRef(1), offset: 1 }, applies: [TypeRef(0)] }] }, params: [Param { consumes: true, typ: TypeRef(1) }], returns: [TypeRef(0)] }, body: Internal { byte_size: Some(13), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(1))] }, code: Exp(LargeVec([Unpack(ValueRef(0), PermRef(0))])) } }, FunctionComponent { byte_size: Some(55), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(0))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Data { link: DataLink { module: ModRef(1), offset: 1 }, applies: [TypeRef(0)] }, Projection { typ: TypeRef(1) }, Projection { typ: TypeRef(0) }] }, params: [Param { consumes: true, typ: TypeRef(2) }], returns: [TypeRef(3)] }, body: Internal { byte_size: Some(13), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(2))] }, code: Exp(LargeVec([Unpack(ValueRef(0), PermRef(0))])) } }, FunctionComponent { byte_size: Some(53), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(0))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Projection { typ: TypeRef(0) }, Data { link: DataLink { module: ModRef(1), offset: 1 }, applies: [TypeRef(1)] }] }, params: [Param { consumes: true, typ: TypeRef(2) }], returns: [TypeRef(1)] }, body: Internal { byte_size: Some(13), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(2))] }, code: Exp(LargeVec([Unpack(ValueRef(0), PermRef(0))])) } }, FunctionComponent { byte_size: Some(64), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(0))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Projection { typ: TypeRef(0) }, Data { link: DataLink { module: ModRef(1), offset: 1 }, applies: [TypeRef(1)] }, Projection { typ: TypeRef(2) }, Projection { typ: TypeRef(1) }] }, params: [Param { consumes: false, typ: TypeRef(2) }], returns: [TypeRef(4)] }, body: Internal { byte_size: Some(20), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(3))] }, code: Exp(LargeVec([Project(TypeRef(3), ValueRef(0)), Unpack(ValueRef(0), PermRef(0)), Move(ValueRef(0))])) } }, FunctionComponent { byte_size: Some(65), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(0))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Data { link: DataLink { module: ModRef(1), offset: 1 }, applies: [TypeRef(0)] }, Projection { typ: TypeRef(1) }, Projection { typ: TypeRef(0) }, Projection { typ: TypeRef(3) }] }, params: [Param { consumes: true, typ: TypeRef(2) }], returns: [TypeRef(4)] }, body: Internal { byte_size: Some(21), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(2))] }, code: Exp(LargeVec([Field(ValueRef(0), PermRef(0), 0), Project(TypeRef(4), ValueRef(0)), Move(ValueRef(0))])) } }, FunctionComponent { byte_size: Some(56), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(0))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Data { link: DataLink { module: ModRef(1), offset: 1 }, applies: [TypeRef(0)] }, Projection { typ: TypeRef(1) }, Projection { typ: TypeRef(0) }] }, params: [Param { consumes: true, typ: TypeRef(2) }], returns: [TypeRef(3)] }, body: Internal { byte_size: Some(14), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(2))] }, code: Exp(LargeVec([Field(ValueRef(0), PermRef(0), 0)])) } }, FunctionComponent { byte_size: Some(25), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(0))], import: PublicImport { modules: [], types: [Projection { typ: TypeRef(0) }] }, params: [Param { consumes: false, typ: TypeRef(0) }], returns: [TypeRef(1)] }, body: Internal { byte_size: Some(10), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [] }, code: Exp(LargeVec([Project(TypeRef(1), ValueRef(0))])) } }, FunctionComponent { byte_size: Some(34), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(0))], import: PublicImport { modules: [], types: [Projection { typ: TypeRef(0) }, Projection { typ: TypeRef(1) }] }, params: [Param { consumes: false, typ: TypeRef(0) }], returns: [TypeRef(2)] }, body: Internal { byte_size: Some(17), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [] }, code: Exp(LargeVec([Project(TypeRef(1), ValueRef(0)), Project(TypeRef(2), ValueRef(0)), Move(ValueRef(0))])) } }, FunctionComponent { byte_size: Some(67), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(49))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Data { link: DataLink { module: ModRef(1), offset: 4 }, applies: [TypeRef(0)] }] }, params: [Param { consumes: true, typ: TypeRef(1) }, Param { consumes: true, typ: TypeRef(0) }], returns: [TypeRef(0)] }, body: Internal { byte_size: Some(27), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(1))] }, code: Exp(LargeVec([Switch(ValueRef(1), PermRef(0), [Exp(LargeVec([Discard(ValueRef(1)), Move(ValueRef(0))])), Exp(LargeVec([Move(ValueRef(0))]))])])) } }, FunctionComponent { byte_size: Some(71), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(1))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Data { link: DataLink { module: ModRef(1), offset: 4 }, applies: [TypeRef(0)] }, Projection { typ: TypeRef(1) }, Projection { typ: TypeRef(0) }] }, params: [Param { consumes: true, typ: TypeRef(2) }, Param { consumes: true, typ: TypeRef(3) }], returns: [TypeRef(3)] }, body: Internal { byte_size: Some(27), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(2))] }, code: Exp(LargeVec([Switch(ValueRef(1), PermRef(0), [Exp(LargeVec([Discard(ValueRef(1)), Move(ValueRef(0))])), Exp(LargeVec([Move(ValueRef(0))]))])])) } }, FunctionComponent { byte_size: Some(69), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(1))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Projection { typ: TypeRef(0) }, Data { link: DataLink { module: ModRef(1), offset: 4 }, applies: [TypeRef(1)] }] }, params: [Param { consumes: true, typ: TypeRef(2) }, Param { consumes: true, typ: TypeRef(1) }], returns: [TypeRef(1)] }, body: Internal { byte_size: Some(27), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(2), TypeRef(2))] }, code: Exp(LargeVec([Switch(ValueRef(1), PermRef(0), [Exp(LargeVec([Discard(ValueRef(1)), Move(ValueRef(0))])), Exp(LargeVec([Move(ValueRef(0))]))])])) } }, FunctionComponent { byte_size: Some(70), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(51))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Data { link: DataLink { module: ModRef(1), offset: 4 }, applies: [TypeRef(0)] }] }, params: [Param { consumes: false, typ: TypeRef(1) }, Param { consumes: true, typ: TypeRef(0) }], returns: [TypeRef(0)] }, body: Internal { byte_size: Some(30), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(4), TypeRef(1))] }, code: Exp(LargeVec([Inspect(ValueRef(1), PermRef(0), [Exp(LargeVec([Discard(ValueRef(1)), Copy(ValueRef(0)), Move(ValueRef(0))])), Exp(LargeVec([Move(ValueRef(0))]))])])) } }, FunctionComponent { byte_size: Some(76), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(51))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Data { link: DataLink { module: ModRef(1), offset: 4 }, applies: [TypeRef(0)] }] }, params: [Param { consumes: true, typ: TypeRef(1) }, Param { consumes: true, typ: TypeRef(0) }], returns: [TypeRef(0)] }, body: Internal { byte_size: Some(36), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(4), TypeRef(1))] }, code: Exp(LargeVec([Inspect(ValueRef(1), PermRef(0), [Exp(LargeVec([Discard(ValueRef(1)), Copy(ValueRef(0)), Move(ValueRef(0))])), Exp(LargeVec([Move(ValueRef(0))]))]), Discard(ValueRef(2)), Move(ValueRef(0))])) } }, FunctionComponent { byte_size: Some(53), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(48))], import: PublicImport { modules: [Remote([204, 154, 61, 238, 235, 17, 216, 48, 120, 65, 132, 94, 114, 52, 39, 227, 90, 210, 71, 163])], types: [Data { link: DataLink { module: ModRef(1), offset: 4 }, applies: [TypeRef(0)] }] }, params: [Param { consumes: true, typ: TypeRef(0) }], returns: [TypeRef(1)] }, body: Internal { byte_size: Some(15), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(1), TypeRef(1))] }, code: Exp(LargeVec([Pack(PermRef(0), Tag(0), [ValueRef(0)])])) } }, FunctionComponent { byte_size: Some(40), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(48)), Physical(CapSet(48))], import: PublicImport { modules: [], types: [Data { link: DataLink { module: ModRef(0), offset: 0 }, applies: [TypeRef(0), TypeRef(1)] }] }, params: [Param { consumes: true, typ: TypeRef(0) }, Param { consumes: true, typ: TypeRef(1) }], returns: [TypeRef(2)] }, body: Internal { byte_size: Some(17), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [], permissions: [Type(PermSet(1), TypeRef(2))] }, code: Exp(LargeVec([Pack(PermRef(0), Tag(0), [ValueRef(1), ValueRef(0)])])) } }, FunctionComponent { byte_size: Some(45), scope: Local, shared: FunSigShared { transactional: false, generics: [Physical(CapSet(48)), Physical(CapSet(48))], import: PublicImport { modules: [], types: [Data { link: DataLink { module: ModRef(0), offset: 0 }, applies: [TypeRef(0), TypeRef(1)] }] }, params: [Param { consumes: true, typ: TypeRef(0) }, Param { consumes: true, typ: TypeRef(1) }], returns: [TypeRef(2)] }, body: Internal { byte_size: Some(22), imports: BodyImport { public: PublicImport { modules: [], types: [] }, callables: [Function { link: FuncLink { module: ModRef(0), offset: 18 }, applies: [TypeRef(0), TypeRef(1)] }], permissions: [Callable(PermSet(8), CallRef(0))] }, code: Exp(LargeVec([Invoke(PermRef(0), [ValueRef(1), ValueRef(0)])])) } }], implements: [], fun_impl_order: BitSerializedVec([true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true]) }