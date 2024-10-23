NAME: Bool
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Bool
		EXTERNAL: false
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
		CONSTRUCTORS
			CONSTRUCTOR: False
				FIELDS
			CONSTRUCTOR: True
				FIELDS
SIGNATURES
FUNCTIONS
	FUNCTION: not
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: b1
				TYPE: sys.Bool.Bool[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] #0 = switch(Copy) b1:
				case False()
					[2] #0 = pack(Move)#sys.Bool.Bool[]|True()
				case True()
					[3] #0 = pack(Move)#sys.Bool.Bool[]|False()
	FUNCTION: and
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: b1
				TYPE: sys.Bool.Bool[]
				CONSUME: false
			PARAM: b2
				TYPE: sys.Bool.Bool[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] #0 = switch(Copy) b1:
				case False()
					[2] #0 = pack(Move)#sys.Bool.Bool[]|False()
				case True()
					[3] #0 = fetch(Copy) b2
	FUNCTION: or
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: b1
				TYPE: sys.Bool.Bool[]
				CONSUME: false
			PARAM: b2
				TYPE: sys.Bool.Bool[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] #0 = switch(Copy) b1:
				case False()
					[2] #0 = fetch(Copy) b2
				case True()
					[3] #0 = pack(Move)#sys.Bool.Bool[]|True()
	FUNCTION: xor
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: b1
				TYPE: sys.Bool.Bool[]
				CONSUME: false
			PARAM: b2
				TYPE: sys.Bool.Bool[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] #0 = switch(Copy) b1:
				case False()
					[2] #0 = fetch(Copy) b2
				case True()
					[3] #0 = call#sys.Bool.not[](b2@_)
	FUNCTION: eq
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: b1
				TYPE: sys.Bool.Bool[]
				CONSUME: false
			PARAM: b2
				TYPE: sys.Bool.Bool[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] $0 = call#sys.Bool.xor[](b1@_, b2@_)
			[2] #0 = call#sys.Bool.not[]($0@_)
IMPLEMENTS
