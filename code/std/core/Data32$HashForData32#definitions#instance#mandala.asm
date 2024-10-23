NAME: Data32$HashForData32
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.Data.Data32[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.Data.hash32[]
		IMPL_TARGET: core.Data32.HashForData32$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.Data32.joinHash[]
		IMPL_TARGET: core.Data32.HashForData32$joinHash[]
