NAME: Data2$HashForData2
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.Data.Data2[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.Data.hash2[]
		IMPL_TARGET: core.Data2.HashForData2$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.Data2.joinHash[]
		IMPL_TARGET: core.Data2.HashForData2$joinHash[]
