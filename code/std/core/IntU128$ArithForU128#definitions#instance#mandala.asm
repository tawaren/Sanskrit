NAME: IntU128$ArithForU128
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Arith
	PARAMS
		PARAM: sys.IntU128.U128[]
IMPLEMENTS
	IMPLEMENT: add
		GENERICS
		FUN_TARGET: sys.IntU128.add[]
		IMPL_TARGET: core.IntU128.ArithForU128$add[]
	IMPLEMENT: sub
		GENERICS
		FUN_TARGET: sys.IntU128.sub[]
		IMPL_TARGET: core.IntU128.ArithForU128$sub[]
	IMPLEMENT: mul
		GENERICS
		FUN_TARGET: sys.IntU128.mul[]
		IMPL_TARGET: core.IntU128.ArithForU128$mul[]
	IMPLEMENT: div
		GENERICS
		FUN_TARGET: sys.IntU128.div[]
		IMPL_TARGET: core.IntU128.ArithForU128$div[]
