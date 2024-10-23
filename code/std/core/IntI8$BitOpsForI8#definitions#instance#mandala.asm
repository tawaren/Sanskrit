NAME: IntI8$BitOpsForI8
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: BitOps
	PARAMS
		PARAM: sys.IntI8.I8[]
IMPLEMENTS
	IMPLEMENT: and
		GENERICS
		FUN_TARGET: sys.IntI8.and[]
		IMPL_TARGET: core.IntI8.BitOpsForI8$and[]
	IMPLEMENT: or
		GENERICS
		FUN_TARGET: sys.IntI8.or[]
		IMPL_TARGET: core.IntI8.BitOpsForI8$or[]
	IMPLEMENT: xor
		GENERICS
		FUN_TARGET: sys.IntI8.xor[]
		IMPL_TARGET: core.IntI8.BitOpsForI8$xor[]
	IMPLEMENT: not
		GENERICS
		FUN_TARGET: sys.IntI8.not[]
		IMPL_TARGET: core.IntI8.BitOpsForI8$not[]
