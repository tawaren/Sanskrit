NAME: Bool
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
SIGNATURES
FUNCTIONS
	FUNCTION: hash
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: op1
				TYPE: sys.Bool.Bool[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
		CODE
			[1] #0 = switch(Copy) op1:
				case False()
					[2] $2 = lit 0x00
					[3] #0 = call#sys.IntU8.hash[]($2@_)
				case True()
					[4] $1 = lit 0x01
					[5] #0 = call#sys.IntU8.hash[]($1@_)
	FUNCTION: joinHash
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: op1
				TYPE: sys.Bool.Bool[]
				CONSUME: false
			PARAM: op2
				TYPE: sys.Bool.Bool[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
		CODE
			[1] $0 = call#core.Bool.hash[](op1@_)
			[2] $2 = call#core.Bool.hash[](op2@_)
			[3] #0 = call#sys.Data.joinHash[]($0@_, $2@_)
IMPLEMENTS
	IMPLEMENT: HashForBool$hash
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Hash.hash[sys.Bool.Bool[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: hash
				TYPE: core.Hash.hash[sys.Bool.Bool[]]
		BINDINGS
			PARAMS
				BINDING: op
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#core.Bool.hash[](op@_)
	IMPLEMENT: HashForBool$joinHash
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Hash.joinHash[sys.Bool.Bool[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: joinHash
				TYPE: core.Hash.joinHash[sys.Bool.Bool[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#core.Bool.joinHash[](op1@_, op2@_)
	IMPLEMENT: EqForBool$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[sys.Bool.Bool[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[sys.Bool.Bool[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.Bool.eq[](op1@_, op2@_)
	IMPLEMENT: BitOpsForBool$and
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.and[sys.Bool.Bool[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: and
				TYPE: core.BitOps.and[sys.Bool.Bool[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.Bool.and[](op1@_, op2@_)
	IMPLEMENT: BitOpsForBool$or
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.or[sys.Bool.Bool[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: or
				TYPE: core.BitOps.or[sys.Bool.Bool[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.Bool.or[](op1@_, op2@_)
	IMPLEMENT: BitOpsForBool$xor
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.xor[sys.Bool.Bool[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: xor
				TYPE: core.BitOps.xor[sys.Bool.Bool[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.Bool.xor[](op1@_, op2@_)
	IMPLEMENT: BitOpsForBool$not
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.not[sys.Bool.Bool[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: not
				TYPE: core.BitOps.not[sys.Bool.Bool[]]
		BINDINGS
			PARAMS
				BINDING: op
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.Bool.not[](op@_)
