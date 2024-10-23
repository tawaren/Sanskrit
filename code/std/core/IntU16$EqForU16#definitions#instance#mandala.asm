NAME: IntU16$EqForU16
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: sys.IntU16.U16[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.IntU16.eq[]
		IMPL_TARGET: core.IntU16.EqForU16$eq[]
