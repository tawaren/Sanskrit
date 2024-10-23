NAME: IntI32$ArithForI32
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Arith
	PARAMS
		PARAM: sys.IntI32.I32[]
IMPLEMENTS
	IMPLEMENT: add
		GENERICS
		FUN_TARGET: sys.IntI32.add[]
		IMPL_TARGET: core.IntI32.ArithForI32$add[]
	IMPLEMENT: sub
		GENERICS
		FUN_TARGET: sys.IntI32.sub[]
		IMPL_TARGET: core.IntI32.ArithForI32$sub[]
	IMPLEMENT: mul
		GENERICS
		FUN_TARGET: sys.IntI32.mul[]
		IMPL_TARGET: core.IntI32.ArithForI32$mul[]
	IMPLEMENT: div
		GENERICS
		FUN_TARGET: sys.IntI32.div[]
		IMPL_TARGET: core.IntI32.ArithForI32$div[]
