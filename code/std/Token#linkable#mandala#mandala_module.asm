NAME: Token
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Token
		EXTERNAL: false
		ACCESS
			Inspect: Global
			Create: Local
			Consume: Local
		CAPABILITIES:  Value Unbound Persist
		GENERICS
			GENERIC: T
				PHANTOM: true
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Token
				FIELDS
					FIELD: amount
						TYPE: sys.IntU128.U128[]
SIGNATURES
FUNCTIONS
	FUNCTION: mint
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Guarded(Set(T))
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: amount
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: std.Token.Token[T]
		CODE
			[1] res = pack(Copy)#std.Token.Token[T]|Token(amount)
	FUNCTION: burn
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Guarded(Set(T))
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: $0
				TYPE: std.Token.Token[T]
				CONSUME: true
		RETURNS
		CODE
			[1] $1 = unpack(Move) $0
			[2]   = return ()
	FUNCTION: zero
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: res
				TYPE: std.Token.Token[T]
		CODE
			[1] $0 = lit 0x00000000000000000000000000000000
			[2] res = pack(Copy)#std.Token.Token[T]|Token($0)
	FUNCTION: split
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: tok
				TYPE: std.Token.Token[T]
				CONSUME: true
			PARAM: split
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: reminder
				TYPE: std.Token.Token[T]
			RETURN: extracted
				TYPE: std.Token.Token[T]
		CODE
			[1] $1 = field#amount(Copy) tok
			[2] $0 = call#sys.IntU128.gte[]($1@_, split@_)
			[3] reminder, extracted = switch(Move) $0:
				case False()
					[4] reminder, extracted = rollback(tok@_):(std.Token.Token[T], std.Token.Token[T])
				case True()
					[5] $6 = field#amount(Move) tok
					[6] $5 = call#sys.IntU128.sub[]($6@_, split@_)
					[7] $4 = pack(Move)#std.Token.Token[T]|Token($5@_)
					[8] $9 = pack(Copy)#std.Token.Token[T]|Token(split)
					[9] reminder, extracted  = return ($4@_, $9@_)
	FUNCTION: merge
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: tok1
				TYPE: std.Token.Token[T]
				CONSUME: true
			PARAM: tok2
				TYPE: std.Token.Token[T]
				CONSUME: true
		RETURNS
			RETURN: res
				TYPE: std.Token.Token[T]
		CODE
			[1] $1 = field#amount(Move) tok1
			[2] $2 = field#amount(Move) tok2
			[3] $0 = call#sys.IntU128.add[]($1@_, $2@_)
			[4] res = pack(Move)#std.Token.Token[T]|Token($0@_)
	FUNCTION: amount
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: $0
				TYPE: std.Token.Token[T]
				CONSUME: false
		RETURNS
			RETURN: $1
				TYPE: sys.IntU128.U128[]
		CODE
			[1] amount = inspect $0
			[2] $1 = fetch(Copy) amount
	FUNCTION: tokenEq
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: tok1
				TYPE: std.Token.Token[T]
				CONSUME: false
			PARAM: tok2
				TYPE: std.Token.Token[T]
				CONSUME: false
		RETURNS
			RETURN: $4
				TYPE: sys.Bool.Bool[]
		CODE
			[1] $0 = field#amount(Copy) tok1
			[2] $1 = field#amount(Copy) tok2
			[3] $4 = call#sys.IntU128.eq[]($0@_, $1@_)
IMPLEMENTS
	IMPLEMENT: FungibleForToken$split
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: std.Fungible.split[std.Token.Token[T]]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: split
				TYPE: std.Fungible.split[std.Token.Token[T]]
		BINDINGS
			PARAMS
				BINDING: item
				BINDING: split
			RETURNS
				BINDING: reminder
				BINDING: extracted
		CODE
			[param] reminder, extracted = call#std.Token.split[T](item@_, split@_)
	IMPLEMENT: FungibleForToken$merge
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: std.Fungible.merge[std.Token.Token[T]]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: merge
				TYPE: std.Fungible.merge[std.Token.Token[T]]
		BINDINGS
			PARAMS
				BINDING: item1
				BINDING: item2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Token.merge[T](item1@_, item2@_)
	IMPLEMENT: FungibleForToken$amount
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.Fungible.amount[std.Token.Token[T]]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: amount
				TYPE: std.Fungible.amount[std.Token.Token[T]]
		BINDINGS
			PARAMS
				BINDING: item
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Token.amount[T](item@_)
	IMPLEMENT: EqForToken$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[std.Token.Token[T]]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[std.Token.Token[T]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Token.tokenEq[T](op1@_, op2@_)
