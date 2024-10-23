NAME: Option$EqForOption
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: std.Option.Option[T]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Option.optionEq[T]
		IMPL_TARGET: std.Option.EqForOption$eq[T]
