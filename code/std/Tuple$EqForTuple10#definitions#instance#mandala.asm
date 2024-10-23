NAME: Tuple$EqForTuple10
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: std.Tuple.Tuple10[T1,T2,T3,T4,T5,T6,T7,T8,T9,T10]
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
			GENERIC: T9
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: T10
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Tuple.eq10[T1,T2,T3,T4,T5,T6,T7,T8,T9,T10]
		IMPL_TARGET: std.Tuple.EqForTuple10$eq[T1,T2,T3,T4,T5,T6,T7,T8,T9,T10]
