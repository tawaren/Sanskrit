NAME: IntU32$HashForU32
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.IntU32.U32[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.IntU32.hash[]
		IMPL_TARGET: core.IntU32.HashForU32$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.IntU32.joinHash[]
		IMPL_TARGET: core.IntU32.HashForU32$joinHash[]
