NAME: IntI64$HashForI64
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.IntI64.I64[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.IntI64.hash[]
		IMPL_TARGET: core.IntI64.HashForI64$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.IntI64.joinHash[]
		IMPL_TARGET: core.IntI64.HashForI64$joinHash[]
