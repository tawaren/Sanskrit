NAME: IntI128$EqForI128
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: sys.IntI128.I128[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.IntI128.eq[]
		IMPL_TARGET: core.IntI128.EqForI128$eq[]
