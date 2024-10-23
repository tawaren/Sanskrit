NAME: Projected$EqForProjected
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: projected(typeParam(0))
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: core.Projected.projectedEq[T]
		IMPL_TARGET: core.Projected.EqForProjected$eq[T]
