NAME: IntI8$ArithForI8
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Arith
	PARAMS
		PARAM: sys.IntI8.I8[]
IMPLEMENTS
	IMPLEMENT: add
		GENERICS
		FUN_TARGET: sys.IntI8.add[]
		IMPL_TARGET: core.IntI8.ArithForI8$add[]
	IMPLEMENT: sub
		GENERICS
		FUN_TARGET: sys.IntI8.sub[]
		IMPL_TARGET: core.IntI8.ArithForI8$sub[]
	IMPLEMENT: mul
		GENERICS
		FUN_TARGET: sys.IntI8.mul[]
		IMPL_TARGET: core.IntI8.ArithForI8$mul[]
	IMPLEMENT: div
		GENERICS
		FUN_TARGET: sys.IntI8.div[]
		IMPL_TARGET: core.IntI8.ArithForI8$div[]
