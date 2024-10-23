NAME: Bool$EqForBool
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: sys.Bool.Bool[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.Bool.eq[]
		IMPL_TARGET: core.Bool.EqForBool$eq[]
