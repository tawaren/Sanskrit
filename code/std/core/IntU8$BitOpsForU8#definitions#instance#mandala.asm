NAME: IntU8$BitOpsForU8
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: BitOps
	PARAMS
		PARAM: sys.IntU8.U8[]
IMPLEMENTS
	IMPLEMENT: and
		GENERICS
		FUN_TARGET: sys.IntU8.and[]
		IMPL_TARGET: core.IntU8.BitOpsForU8$and[]
	IMPLEMENT: or
		GENERICS
		FUN_TARGET: sys.IntU8.or[]
		IMPL_TARGET: core.IntU8.BitOpsForU8$or[]
	IMPLEMENT: xor
		GENERICS
		FUN_TARGET: sys.IntU8.xor[]
		IMPL_TARGET: core.IntU8.BitOpsForU8$xor[]
	IMPLEMENT: not
		GENERICS
		FUN_TARGET: sys.IntU8.not[]
		IMPL_TARGET: core.IntU8.BitOpsForU8$not[]
