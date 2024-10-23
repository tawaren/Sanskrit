NAME: Data1$EqForData1
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: sys.Data.Data1[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.Data.eq1[]
		IMPL_TARGET: core.Data1.EqForData1$eq[]
