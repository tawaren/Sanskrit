NAME: Data28$HashForData28
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.Data.Data28[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: sys.Data.hash28[]
		IMPL_TARGET: core.Data28.HashForData28$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.Data28.joinHash[]
		IMPL_TARGET: core.Data28.HashForData28$joinHash[]
