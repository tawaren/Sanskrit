NAME: IntI8$HashForI8
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.IntI8.I8[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.IntI8.hash[]
		IMPL_TARGET: core.IntI8.HashForI8$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.IntI8.joinHash[]
		IMPL_TARGET: core.IntI8.HashForI8$joinHash[]
