NAME: IntU16$ArithForU16
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Arith
	PARAMS
		PARAM: sys.IntU16.U16[]
IMPLEMENTS
	IMPLEMENT: add
		GENERICS
		FUN_TARGET: sys.IntU16.add[]
		IMPL_TARGET: core.IntU16.ArithForU16$add[]
	IMPLEMENT: sub
		GENERICS
		FUN_TARGET: sys.IntU16.sub[]
		IMPL_TARGET: core.IntU16.ArithForU16$sub[]
	IMPLEMENT: mul
		GENERICS
		FUN_TARGET: sys.IntU16.mul[]
		IMPL_TARGET: core.IntU16.ArithForU16$mul[]
	IMPLEMENT: div
		GENERICS
		FUN_TARGET: sys.IntU16.div[]
		IMPL_TARGET: core.IntU16.ArithForU16$div[]
