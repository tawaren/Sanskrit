NAME: IntI64$CompareForI64
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Compare
	PARAMS
		PARAM: sys.IntI64.I64[]
IMPLEMENTS
	IMPLEMENT: lt
		GENERICS
		FUN_TARGET: sys.IntI64.lt[]
		IMPL_TARGET: core.IntI64.CompareForI64$lt[]
	IMPLEMENT: lte
		GENERICS
		FUN_TARGET: sys.IntI64.lte[]
		IMPL_TARGET: core.IntI64.CompareForI64$lte[]
	IMPLEMENT: gt
		GENERICS
		FUN_TARGET: sys.IntI64.gt[]
		IMPL_TARGET: core.IntI64.CompareForI64$gt[]
	IMPLEMENT: gte
		GENERICS
		FUN_TARGET: sys.IntI64.gte[]
		IMPL_TARGET: core.IntI64.CompareForI64$gte[]
