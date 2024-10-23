NAME: Id$EqForPrivateId
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: sys.Ids.PrivateId[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.Ids.eqPrivateId[]
		IMPL_TARGET: core.Id.EqForPrivateId$eq[]
