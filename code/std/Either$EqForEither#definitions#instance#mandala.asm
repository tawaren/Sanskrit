NAME: Either$EqForEither
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: std.Either.Either[L,R]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
			GENERIC: L
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: R
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Either.eq[L,R]
		IMPL_TARGET: std.Either.EqForEither$eq[L,R]
