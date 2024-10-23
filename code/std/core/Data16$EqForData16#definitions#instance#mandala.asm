NAME: Data16$EqForData16
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: sys.Data.Data16[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.Data.eq16[]
		IMPL_TARGET: core.Data16.EqForData16$eq[]
