NAME: IntI8$EqForI8
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: sys.IntI8.I8[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.IntI8.eq[]
		IMPL_TARGET: core.IntI8.EqForI8$eq[]
