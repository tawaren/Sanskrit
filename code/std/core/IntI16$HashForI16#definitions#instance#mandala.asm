NAME: IntI16$HashForI16
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.IntI16.I16[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.IntI16.hash[]
		IMPL_TARGET: core.IntI16.HashForI16$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.IntI16.joinHash[]
		IMPL_TARGET: core.IntI16.HashForI16$joinHash[]
