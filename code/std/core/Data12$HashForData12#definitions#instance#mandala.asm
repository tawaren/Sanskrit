NAME: Data12$HashForData12
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.Data.Data12[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.Data.hash12[]
		IMPL_TARGET: core.Data12.HashForData12$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.Data12.joinHash[]
		IMPL_TARGET: core.Data12.HashForData12$joinHash[]
