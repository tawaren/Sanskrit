NAME: Bool$HashForBool
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: sys.Bool.Bool[]
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
		FUN_TARGET: core.Bool.hash[]
		IMPL_TARGET: core.Bool.HashForBool$hash[]
	IMPLEMENT: joinHash
		GENERICS
		FUN_TARGET: core.Bool.joinHash[]
		IMPL_TARGET: core.Bool.HashForBool$joinHash[]
