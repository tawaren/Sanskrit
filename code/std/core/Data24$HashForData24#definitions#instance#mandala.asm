NAME: Data24$HashForData24
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.Data.Data24[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.Data.hash24[]
		IMPL_TARGET: core.Data24.HashForData24$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.Data24.joinHash[]
		IMPL_TARGET: core.Data24.HashForData24$joinHash[]
