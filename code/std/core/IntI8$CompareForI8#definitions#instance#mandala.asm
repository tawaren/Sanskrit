NAME: IntI8$CompareForI8
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Compare
	PARAMS
		PARAM: sys.IntI8.I8[]
IMPLEMENTS
	IMPLEMENT: lt
		GENERICS
		FUN_TARGET: sys.IntI8.lt[]
		IMPL_TARGET: core.IntI8.CompareForI8$lt[]
	IMPLEMENT: lte
		GENERICS
		FUN_TARGET: sys.IntI8.lte[]
		IMPL_TARGET: core.IntI8.CompareForI8$lte[]
	IMPLEMENT: gt
		GENERICS
		FUN_TARGET: sys.IntI8.gt[]
		IMPL_TARGET: core.IntI8.CompareForI8$gt[]
	IMPLEMENT: gte
		GENERICS
		FUN_TARGET: sys.IntI8.gte[]
		IMPL_TARGET: core.IntI8.CompareForI8$gte[]
