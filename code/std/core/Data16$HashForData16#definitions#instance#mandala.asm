NAME: Data16$HashForData16
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.Data.Data16[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.Data.hash16[]
		IMPL_TARGET: core.Data16.HashForData16$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.Data16.joinHash[]
		IMPL_TARGET: core.Data16.HashForData16$joinHash[]
