NAME: IntI128$BitOpsForI128
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: BitOps
	PARAMS
		PARAM: sys.IntI128.I128[]
IMPLEMENTS
	IMPLEMENT: and
		GENERICS
		FUN_TARGET: sys.IntI128.and[]
		IMPL_TARGET: core.IntI128.BitOpsForI128$and[]
	IMPLEMENT: or
		GENERICS
		FUN_TARGET: sys.IntI128.or[]
		IMPL_TARGET: core.IntI128.BitOpsForI128$or[]
	IMPLEMENT: xor
		GENERICS
		FUN_TARGET: sys.IntI128.xor[]
		IMPL_TARGET: core.IntI128.BitOpsForI128$xor[]
	IMPLEMENT: not
		GENERICS
		FUN_TARGET: sys.IntI128.not[]
		IMPL_TARGET: core.IntI128.BitOpsForI128$not[]
