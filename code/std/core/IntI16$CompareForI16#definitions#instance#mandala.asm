NAME: IntI16$CompareForI16
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Compare
	PARAMS
		PARAM: sys.IntI16.I16[]
IMPLEMENTS
	IMPLEMENT: lt
		GENERICS
		FUN_TARGET: sys.IntI16.lt[]
		IMPL_TARGET: core.IntI16.CompareForI16$lt[]
	IMPLEMENT: lte
		GENERICS
		FUN_TARGET: sys.IntI16.lte[]
		IMPL_TARGET: core.IntI16.CompareForI16$lte[]
	IMPLEMENT: gt
		GENERICS
		FUN_TARGET: sys.IntI16.gt[]
		IMPL_TARGET: core.IntI16.CompareForI16$gt[]
	IMPLEMENT: gte
		GENERICS
		FUN_TARGET: sys.IntI16.gte[]
		IMPL_TARGET: core.IntI16.CompareForI16$gte[]
