NAME: Data20$HashForData20
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.Data.Data20[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.Data.hash20[]
		IMPL_TARGET: core.Data20.HashForData20$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.Data20.joinHash[]
		IMPL_TARGET: core.Data20.HashForData20$joinHash[]
