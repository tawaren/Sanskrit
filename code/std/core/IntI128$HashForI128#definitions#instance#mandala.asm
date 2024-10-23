NAME: IntI128$HashForI128
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.IntI128.I128[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.IntI128.hash[]
		IMPL_TARGET: core.IntI128.HashForI128$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.IntI128.joinHash[]
		IMPL_TARGET: core.IntI128.HashForI128$joinHash[]
