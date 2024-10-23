NAME: IntU64$ArithForU64
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Arith
	PARAMS
		PARAM: sys.IntU64.U64[]
IMPLEMENTS
	IMPLEMENT: add
		GENERICS
		FUN_TARGET: sys.IntU64.add[]
		IMPL_TARGET: core.IntU64.ArithForU64$add[]
	IMPLEMENT: sub
		GENERICS
		FUN_TARGET: sys.IntU64.sub[]
		IMPL_TARGET: core.IntU64.ArithForU64$sub[]
	IMPLEMENT: mul
		GENERICS
		FUN_TARGET: sys.IntU64.mul[]
		IMPL_TARGET: core.IntU64.ArithForU64$mul[]
	IMPLEMENT: div
		GENERICS
		FUN_TARGET: sys.IntU64.div[]
		IMPL_TARGET: core.IntU64.ArithForU64$div[]
