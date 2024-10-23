NAME: IntU8$HashForU8
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.IntU8.U8[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.IntU8.hash[]
		IMPL_TARGET: core.IntU8.HashForU8$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.IntU8.joinHash[]
		IMPL_TARGET: core.IntU8.HashForU8$joinHash[]
