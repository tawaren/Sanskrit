NAME: Tuple$EqForTuple8
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: std.Tuple.Tuple8[T1,T2,T3,T4,T5,T6,T7,T8]
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
			GENERIC: T5
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: T6
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: T7
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: T8
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Tuple.eq8[T1,T2,T3,T4,T5,T6,T7,T8]
		IMPL_TARGET: std.Tuple.EqForTuple8$eq[T1,T2,T3,T4,T5,T6,T7,T8]
