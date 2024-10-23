NAME: IntU64$BitOpsForU64
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: BitOps
	PARAMS
		PARAM: sys.IntU64.U64[]
IMPLEMENTS
	IMPLEMENT: and
		GENERICS
		FUN_TARGET: sys.IntU64.and[]
		IMPL_TARGET: core.IntU64.BitOpsForU64$and[]
	IMPLEMENT: or
		GENERICS
		FUN_TARGET: sys.IntU64.or[]
		IMPL_TARGET: core.IntU64.BitOpsForU64$or[]
	IMPLEMENT: xor
		GENERICS
		FUN_TARGET: sys.IntU64.xor[]
		IMPL_TARGET: core.IntU64.BitOpsForU64$xor[]
	IMPLEMENT: not
		GENERICS
		FUN_TARGET: sys.IntU64.not[]
		IMPL_TARGET: core.IntU64.BitOpsForU64$not[]
