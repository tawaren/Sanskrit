NAME: Data1$HashForData1
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.Data.Data1[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.Data.hash1[]
		IMPL_TARGET: core.Data1.HashForData1$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.Data1.joinHash[]
		IMPL_TARGET: core.Data1.HashForData1$joinHash[]
