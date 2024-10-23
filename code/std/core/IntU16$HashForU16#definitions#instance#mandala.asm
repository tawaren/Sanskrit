NAME: IntU16$HashForU16
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.IntU16.U16[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.IntU16.hash[]
		IMPL_TARGET: core.IntU16.HashForU16$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.IntU16.joinHash[]
		IMPL_TARGET: core.IntU16.HashForU16$joinHash[]
