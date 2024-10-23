NAME: Data4$HashForData4
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.Data.Data4[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.Data.hash4[]
		IMPL_TARGET: core.Data4.HashForData4$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.Data4.joinHash[]
		IMPL_TARGET: core.Data4.HashForData4$joinHash[]
