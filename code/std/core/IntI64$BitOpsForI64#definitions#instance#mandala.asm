NAME: IntI64$BitOpsForI64
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: BitOps
	PARAMS
		PARAM: sys.IntI64.I64[]
IMPLEMENTS
	IMPLEMENT: and
		GENERICS
		FUN_TARGET: sys.IntI64.and[]
		IMPL_TARGET: core.IntI64.BitOpsForI64$and[]
	IMPLEMENT: or
		GENERICS
		FUN_TARGET: sys.IntI64.or[]
		IMPL_TARGET: core.IntI64.BitOpsForI64$or[]
	IMPLEMENT: xor
		GENERICS
		FUN_TARGET: sys.IntI64.xor[]
		IMPL_TARGET: core.IntI64.BitOpsForI64$xor[]
	IMPLEMENT: not
		GENERICS
		FUN_TARGET: sys.IntI64.not[]
		IMPL_TARGET: core.IntI64.BitOpsForI64$not[]
