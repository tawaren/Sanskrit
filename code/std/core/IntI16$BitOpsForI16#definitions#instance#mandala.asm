NAME: IntI16$BitOpsForI16
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: BitOps
	PARAMS
		PARAM: sys.IntI16.I16[]
IMPLEMENTS
	IMPLEMENT: and
		GENERICS
		FUN_TARGET: sys.IntI16.and[]
		IMPL_TARGET: core.IntI16.BitOpsForI16$and[]
	IMPLEMENT: or
		GENERICS
		FUN_TARGET: sys.IntI16.or[]
		IMPL_TARGET: core.IntI16.BitOpsForI16$or[]
	IMPLEMENT: xor
		GENERICS
		FUN_TARGET: sys.IntI16.xor[]
		IMPL_TARGET: core.IntI16.BitOpsForI16$xor[]
	IMPLEMENT: not
		GENERICS
		FUN_TARGET: sys.IntI16.not[]
		IMPL_TARGET: core.IntI16.BitOpsForI16$not[]
