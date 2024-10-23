NAME: Id$EqForPublicId
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: projected(sys.Ids.PrivateId[])
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: sys.Ids.eqId[]
		IMPL_TARGET: core.Id.EqForPublicId$eq[]
