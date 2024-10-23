NAME: IntU16$BitOpsForU16
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: BitOps
	PARAMS
		PARAM: sys.IntU16.U16[]
IMPLEMENTS
	IMPLEMENT: and
		GENERICS
		FUN_TARGET: sys.IntU16.and[]
		IMPL_TARGET: core.IntU16.BitOpsForU16$and[]
	IMPLEMENT: or
		GENERICS
		FUN_TARGET: sys.IntU16.or[]
		IMPL_TARGET: core.IntU16.BitOpsForU16$or[]
	IMPLEMENT: xor
		GENERICS
		FUN_TARGET: sys.IntU16.xor[]
		IMPL_TARGET: core.IntU16.BitOpsForU16$xor[]
	IMPLEMENT: not
		GENERICS
		FUN_TARGET: sys.IntU16.not[]
		IMPL_TARGET: core.IntU16.BitOpsForU16$not[]
