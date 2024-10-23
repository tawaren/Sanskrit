NAME: Id$EqForPrivateModuleId
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: sys.Ids.PrivateModuleId[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.Ids.eqPrivateModuleId[]
		IMPL_TARGET: core.Id.EqForPrivateModuleId$eq[]
