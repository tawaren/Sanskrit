NAME: Data24$EqForData24
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: sys.Data.Data24[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.Data.eq24[]
		IMPL_TARGET: core.Data24.EqForData24$eq[]
