NAME: IntI64$EqForI64
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: sys.IntI64.I64[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.IntI64.eq[]
		IMPL_TARGET: core.IntI64.EqForI64$eq[]
