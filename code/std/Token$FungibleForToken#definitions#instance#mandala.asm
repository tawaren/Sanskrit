NAME: Token$FungibleForToken
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Fungible
	PARAMS
		PARAM: std.Token.Token[T]
IMPLEMENTS
	IMPLEMENT: split
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Token.split[T]
		IMPL_TARGET: std.Token.FungibleForToken$split[T]
	IMPLEMENT: merge
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Token.merge[T]
		IMPL_TARGET: std.Token.FungibleForToken$merge[T]
	IMPLEMENT: amount
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Token.amount[T]
		IMPL_TARGET: std.Token.FungibleForToken$amount[T]
