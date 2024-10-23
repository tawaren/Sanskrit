NAME: IntU64$HashForU64
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.IntU64.U64[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.IntU64.hash[]
		IMPL_TARGET: core.IntU64.HashForU64$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.IntU64.joinHash[]
		IMPL_TARGET: core.IntU64.HashForU64$joinHash[]
