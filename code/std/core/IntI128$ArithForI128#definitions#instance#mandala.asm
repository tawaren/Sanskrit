NAME: IntI128$ArithForI128
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Arith
	PARAMS
		PARAM: sys.IntI128.I128[]
IMPLEMENTS
	IMPLEMENT: add
		GENERICS
		FUN_TARGET: sys.IntI128.add[]
		IMPL_TARGET: core.IntI128.ArithForI128$add[]
	IMPLEMENT: sub
		GENERICS
		FUN_TARGET: sys.IntI128.sub[]
		IMPL_TARGET: core.IntI128.ArithForI128$sub[]
	IMPLEMENT: mul
		GENERICS
		FUN_TARGET: sys.IntI128.mul[]
		IMPL_TARGET: core.IntI128.ArithForI128$mul[]
	IMPLEMENT: div
		GENERICS
		FUN_TARGET: sys.IntI128.div[]
		IMPL_TARGET: core.IntI128.ArithForI128$div[]
