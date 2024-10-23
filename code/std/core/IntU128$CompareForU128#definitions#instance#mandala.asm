NAME: IntU128$CompareForU128
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Compare
	PARAMS
		PARAM: sys.IntU128.U128[]
IMPLEMENTS
	IMPLEMENT: lt
		GENERICS
		FUN_TARGET: sys.IntU128.lt[]
		IMPL_TARGET: core.IntU128.CompareForU128$lt[]
	IMPLEMENT: lte
		GENERICS
		FUN_TARGET: sys.IntU128.lte[]
		IMPL_TARGET: core.IntU128.CompareForU128$lte[]
	IMPLEMENT: gt
		GENERICS
		FUN_TARGET: sys.IntU128.gt[]
		IMPL_TARGET: core.IntU128.CompareForU128$gt[]
	IMPLEMENT: gte
		GENERICS
		FUN_TARGET: sys.IntU128.gte[]
		IMPL_TARGET: core.IntU128.CompareForU128$gte[]
