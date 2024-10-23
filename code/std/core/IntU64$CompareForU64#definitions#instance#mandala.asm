NAME: IntU64$CompareForU64
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Compare
	PARAMS
		PARAM: sys.IntU64.U64[]
IMPLEMENTS
	IMPLEMENT: lt
		GENERICS
		FUN_TARGET: sys.IntU64.lt[]
		IMPL_TARGET: core.IntU64.CompareForU64$lt[]
	IMPLEMENT: lte
		GENERICS
		FUN_TARGET: sys.IntU64.lte[]
		IMPL_TARGET: core.IntU64.CompareForU64$lte[]
	IMPLEMENT: gt
		GENERICS
		FUN_TARGET: sys.IntU64.gt[]
		IMPL_TARGET: core.IntU64.CompareForU64$gt[]
	IMPLEMENT: gte
		GENERICS
		FUN_TARGET: sys.IntU64.gte[]
		IMPL_TARGET: core.IntU64.CompareForU64$gte[]
