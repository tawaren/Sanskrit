NAME: IntU8$CompareForU8
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Compare
	PARAMS
		PARAM: sys.IntU8.U8[]
IMPLEMENTS
	IMPLEMENT: lt
		GENERICS
		FUN_TARGET: sys.IntU8.lt[]
		IMPL_TARGET: core.IntU8.CompareForU8$lt[]
	IMPLEMENT: lte
		GENERICS
		FUN_TARGET: sys.IntU8.lte[]
		IMPL_TARGET: core.IntU8.CompareForU8$lte[]
	IMPLEMENT: gt
		GENERICS
		FUN_TARGET: sys.IntU8.gt[]
		IMPL_TARGET: core.IntU8.CompareForU8$gt[]
	IMPLEMENT: gte
		GENERICS
		FUN_TARGET: sys.IntU8.gte[]
		IMPL_TARGET: core.IntU8.CompareForU8$gte[]
