NAME: IntI32$BitOpsForI32
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: BitOps
	PARAMS
		PARAM: sys.IntI32.I32[]
IMPLEMENTS
	IMPLEMENT: and
		GENERICS
		FUN_TARGET: sys.IntI32.and[]
		IMPL_TARGET: core.IntI32.BitOpsForI32$and[]
	IMPLEMENT: or
		GENERICS
		FUN_TARGET: sys.IntI32.or[]
		IMPL_TARGET: core.IntI32.BitOpsForI32$or[]
	IMPLEMENT: xor
		GENERICS
		FUN_TARGET: sys.IntI32.xor[]
		IMPL_TARGET: core.IntI32.BitOpsForI32$xor[]
	IMPLEMENT: not
		GENERICS
		FUN_TARGET: sys.IntI32.not[]
		IMPL_TARGET: core.IntI32.BitOpsForI32$not[]
