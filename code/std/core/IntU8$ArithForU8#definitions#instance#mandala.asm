NAME: IntU8$ArithForU8
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Arith
	PARAMS
		PARAM: sys.IntU8.U8[]
IMPLEMENTS
	IMPLEMENT: add
		GENERICS
		FUN_TARGET: sys.IntU8.add[]
		IMPL_TARGET: core.IntU8.ArithForU8$add[]
	IMPLEMENT: sub
		GENERICS
		FUN_TARGET: sys.IntU8.sub[]
		IMPL_TARGET: core.IntU8.ArithForU8$sub[]
	IMPLEMENT: mul
		GENERICS
		FUN_TARGET: sys.IntU8.mul[]
		IMPL_TARGET: core.IntU8.ArithForU8$mul[]
	IMPLEMENT: div
		GENERICS
		FUN_TARGET: sys.IntU8.div[]
		IMPL_TARGET: core.IntU8.ArithForU8$div[]
