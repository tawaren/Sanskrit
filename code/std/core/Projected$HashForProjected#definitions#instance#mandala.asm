NAME: Projected$HashForProjected
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Hash
	PARAMS
		PARAM: projected(typeParam(0))
IMPLEMENTS
	IMPLEMENT: hash
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: core.Projected.projectedHash[T]
		IMPL_TARGET: core.Projected.HashForProjected$hash[T]
	IMPLEMENT: joinHash
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: core.Projected.projectedJoinHash[T]
		IMPL_TARGET: core.Projected.HashForProjected$joinHash[T]
