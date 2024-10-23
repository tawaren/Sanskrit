NAME: IntI64$ArithForI64
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Arith
	PARAMS
		PARAM: sys.IntI64.I64[]
IMPLEMENTS
	IMPLEMENT: add
		GENERICS
		FUN_TARGET: sys.IntI64.add[]
		IMPL_TARGET: core.IntI64.ArithForI64$add[]
	IMPLEMENT: sub
		GENERICS
		FUN_TARGET: sys.IntI64.sub[]
		IMPL_TARGET: core.IntI64.ArithForI64$sub[]
	IMPLEMENT: mul
		GENERICS
		FUN_TARGET: sys.IntI64.mul[]
		IMPL_TARGET: core.IntI64.ArithForI64$mul[]
	IMPLEMENT: div
		GENERICS
		FUN_TARGET: sys.IntI64.div[]
		IMPL_TARGET: core.IntI64.ArithForI64$div[]
