NAME: Tuple$EqForTuple4
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: std.Tuple.Tuple4[T1,T2,T3,T4]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
			GENERIC: T1
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: T2
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: T3
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: T4
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Tuple.eq4[T1,T2,T3,T4]
		IMPL_TARGET: std.Tuple.EqForTuple4$eq[T1,T2,T3,T4]
