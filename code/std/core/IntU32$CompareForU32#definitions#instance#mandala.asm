NAME: IntU32$CompareForU32
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Compare
	PARAMS
		PARAM: sys.IntU32.U32[]
IMPLEMENTS
	IMPLEMENT: lt
		GENERICS
		FUN_TARGET: sys.IntU32.lt[]
		IMPL_TARGET: core.IntU32.CompareForU32$lt[]
	IMPLEMENT: lte
		GENERICS
		FUN_TARGET: sys.IntU32.lte[]
		IMPL_TARGET: core.IntU32.CompareForU32$lte[]
	IMPLEMENT: gt
		GENERICS
		FUN_TARGET: sys.IntU32.gt[]
		IMPL_TARGET: core.IntU32.CompareForU32$gt[]
	IMPLEMENT: gte
		GENERICS
		FUN_TARGET: sys.IntU32.gte[]
		IMPL_TARGET: core.IntU32.CompareForU32$gte[]
