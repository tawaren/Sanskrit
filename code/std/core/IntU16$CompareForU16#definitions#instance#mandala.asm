NAME: IntU16$CompareForU16
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Compare
	PARAMS
		PARAM: sys.IntU16.U16[]
IMPLEMENTS
	IMPLEMENT: lt
		GENERICS
		FUN_TARGET: sys.IntU16.lt[]
		IMPL_TARGET: core.IntU16.CompareForU16$lt[]
	IMPLEMENT: lte
		GENERICS
		FUN_TARGET: sys.IntU16.lte[]
		IMPL_TARGET: core.IntU16.CompareForU16$lte[]
	IMPLEMENT: gt
		GENERICS
		FUN_TARGET: sys.IntU16.gt[]
		IMPL_TARGET: core.IntU16.CompareForU16$gt[]
	IMPLEMENT: gte
		GENERICS
		FUN_TARGET: sys.IntU16.gte[]
		IMPL_TARGET: core.IntU16.CompareForU16$gte[]
