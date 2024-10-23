NAME: IntI128$CompareForI128
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Compare
	PARAMS
		PARAM: sys.IntI128.I128[]
IMPLEMENTS
	IMPLEMENT: lt
		GENERICS
		FUN_TARGET: sys.IntI128.lt[]
		IMPL_TARGET: core.IntI128.CompareForI128$lt[]
	IMPLEMENT: lte
		GENERICS
		FUN_TARGET: sys.IntI128.lte[]
		IMPL_TARGET: core.IntI128.CompareForI128$lte[]
	IMPLEMENT: gt
		GENERICS
		FUN_TARGET: sys.IntI128.gt[]
		IMPL_TARGET: core.IntI128.CompareForI128$gt[]
	IMPLEMENT: gte
		GENERICS
		FUN_TARGET: sys.IntI128.gte[]
		IMPL_TARGET: core.IntI128.CompareForI128$gte[]
