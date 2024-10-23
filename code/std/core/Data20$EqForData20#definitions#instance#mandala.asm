NAME: Data20$EqForData20
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: sys.Data.Data20[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.Data.eq20[]
		IMPL_TARGET: core.Data20.EqForData20$eq[]
