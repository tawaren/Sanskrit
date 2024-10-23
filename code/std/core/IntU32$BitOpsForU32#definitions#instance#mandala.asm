NAME: IntU32$BitOpsForU32
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: BitOps
	PARAMS
		PARAM: sys.IntU32.U32[]
IMPLEMENTS
	IMPLEMENT: and
		GENERICS
		FUN_TARGET: sys.IntU32.and[]
		IMPL_TARGET: core.IntU32.BitOpsForU32$and[]
	IMPLEMENT: or
		GENERICS
		FUN_TARGET: sys.IntU32.or[]
		IMPL_TARGET: core.IntU32.BitOpsForU32$or[]
	IMPLEMENT: xor
		GENERICS
		FUN_TARGET: sys.IntU32.xor[]
		IMPL_TARGET: core.IntU32.BitOpsForU32$xor[]
	IMPLEMENT: not
		GENERICS
		FUN_TARGET: sys.IntU32.not[]
		IMPL_TARGET: core.IntU32.BitOpsForU32$not[]
