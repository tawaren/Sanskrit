NAME: IntU8$EqForU8
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: sys.IntU8.U8[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.IntU8.eq[]
		IMPL_TARGET: core.IntU8.EqForU8$eq[]
