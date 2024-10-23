NAME: IntU32$ArithForU32
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Arith
	PARAMS
		PARAM: sys.IntU32.U32[]
IMPLEMENTS
	IMPLEMENT: add
		GENERICS
		FUN_TARGET: sys.IntU32.add[]
		IMPL_TARGET: core.IntU32.ArithForU32$add[]
	IMPLEMENT: sub
		GENERICS
		FUN_TARGET: sys.IntU32.sub[]
		IMPL_TARGET: core.IntU32.ArithForU32$sub[]
	IMPLEMENT: mul
		GENERICS
		FUN_TARGET: sys.IntU32.mul[]
		IMPL_TARGET: core.IntU32.ArithForU32$mul[]
	IMPLEMENT: div
		GENERICS
		FUN_TARGET: sys.IntU32.div[]
		IMPL_TARGET: core.IntU32.ArithForU32$div[]
