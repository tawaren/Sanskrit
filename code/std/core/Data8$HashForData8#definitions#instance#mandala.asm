NAME: Data8$HashForData8
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.Data.Data8[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.Data.hash8[]
		IMPL_TARGET: core.Data8.HashForData8$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.Data8.joinHash[]
		IMPL_TARGET: core.Data8.HashForData8$joinHash[]
