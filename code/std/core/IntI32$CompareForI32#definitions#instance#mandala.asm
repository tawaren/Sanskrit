NAME: IntI32$CompareForI32
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Compare
	PARAMS
		PARAM: sys.IntI32.I32[]
IMPLEMENTS
	IMPLEMENT: lt
		GENERICS
		FUN_TARGET: sys.IntI32.lt[]
		IMPL_TARGET: core.IntI32.CompareForI32$lt[]
	IMPLEMENT: lte
		GENERICS
		FUN_TARGET: sys.IntI32.lte[]
		IMPL_TARGET: core.IntI32.CompareForI32$lte[]
	IMPLEMENT: gt
		GENERICS
		FUN_TARGET: sys.IntI32.gt[]
		IMPL_TARGET: core.IntI32.CompareForI32$gt[]
	IMPLEMENT: gte
		GENERICS
		FUN_TARGET: sys.IntI32.gte[]
		IMPL_TARGET: core.IntI32.CompareForI32$gte[]
