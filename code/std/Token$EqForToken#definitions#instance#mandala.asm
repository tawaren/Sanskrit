NAME: Token$EqForToken
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: std.Token.Token[T]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Token.tokenEq[T]
		IMPL_TARGET: std.Token.EqForToken$eq[T]
