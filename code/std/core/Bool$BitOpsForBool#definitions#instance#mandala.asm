NAME: Bool$BitOpsForBool
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: BitOps
	PARAMS
		PARAM: sys.Bool.Bool[]
IMPLEMENTS
	IMPLEMENT: and
		GENERICS
		FUN_TARGET: sys.Bool.and[]
		IMPL_TARGET: core.Bool.BitOpsForBool$and[]
	IMPLEMENT: or
		GENERICS
		FUN_TARGET: sys.Bool.or[]
		IMPL_TARGET: core.Bool.BitOpsForBool$or[]
	IMPLEMENT: xor
		GENERICS
		FUN_TARGET: sys.Bool.xor[]
		IMPL_TARGET: core.Bool.BitOpsForBool$xor[]
	IMPLEMENT: not
		GENERICS
		FUN_TARGET: sys.Bool.not[]
		IMPL_TARGET: core.Bool.BitOpsForBool$not[]
