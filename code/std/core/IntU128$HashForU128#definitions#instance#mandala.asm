NAME: IntU128$HashForU128
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.IntU128.U128[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.IntU128.hash[]
		IMPL_TARGET: core.IntU128.HashForU128$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.IntU128.joinHash[]
		IMPL_TARGET: core.IntU128.HashForU128$joinHash[]
