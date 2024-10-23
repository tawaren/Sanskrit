NAME: Id$EqForPublicModuleId
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: projected(sys.Ids.PrivateModuleId[])
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.Ids.eqModuleId[]
		IMPL_TARGET: core.Id.EqForPublicModuleId$eq[]
