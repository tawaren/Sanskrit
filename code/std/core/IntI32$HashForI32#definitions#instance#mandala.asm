NAME: IntI32$HashForI32
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.IntI32.I32[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.IntI32.hash[]
		IMPL_TARGET: core.IntI32.HashForI32$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.IntI32.joinHash[]
		IMPL_TARGET: core.IntI32.HashForI32$joinHash[]
