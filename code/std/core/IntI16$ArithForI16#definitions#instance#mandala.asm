NAME: IntI16$ArithForI16
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Arith
	PARAMS
		PARAM: sys.IntI16.I16[]
IMPLEMENTS
	IMPLEMENT: add
		GENERICS
		FUN_TARGET: sys.IntI16.add[]
		IMPL_TARGET: core.IntI16.ArithForI16$add[]
	IMPLEMENT: sub
		GENERICS
		FUN_TARGET: sys.IntI16.sub[]
		IMPL_TARGET: core.IntI16.ArithForI16$sub[]
	IMPLEMENT: mul
		GENERICS
		FUN_TARGET: sys.IntI16.mul[]
		IMPL_TARGET: core.IntI16.ArithForI16$mul[]
	IMPLEMENT: div
		GENERICS
		FUN_TARGET: sys.IntI16.div[]
		IMPL_TARGET: core.IntI16.ArithForI16$div[]
