NAME: Either
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Either
		EXTERNAL: false
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
			GENERIC: L
				PHANTOM: false
				CAPABILITIES: 
			GENERIC: R
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Left
				FIELDS
					FIELD: left
						TYPE: L
			CONSTRUCTOR: Right
				FIELDS
					FIELD: right
						TYPE: R
SIGNATURES
FUNCTIONS
	FUNCTION: isLeft
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: L
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: R
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: op1
				TYPE: std.Either.Either[L,R]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] #0 = inspect op1:
				case Left($1)
					[2] #0 = pack(Move)#sys.Bool.Bool[]|True()
				case Right($2)
					[3] #0 = pack(Move)#sys.Bool.Bool[]|False()
	FUNCTION: isRight
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: L
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: R
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: op1
				TYPE: std.Either.Either[L,R]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] #0 = inspect op1:
				case Left($1)
					[2] #0 = pack(Move)#sys.Bool.Bool[]|False()
				case Right($2)
					[3] #0 = pack(Move)#sys.Bool.Bool[]|True()
	FUNCTION: eq
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: L
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: R
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: op1
				TYPE: std.Either.Either[L,R]
				CONSUME: false
			PARAM: op2
				TYPE: std.Either.Either[L,R]
				CONSUME: false
			PARAM: eqLeft
				TYPE: core.Equal.eq[L]
				CONSUME: true
			PARAM: eqRight
				TYPE: core.Equal.eq[R]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] #0 = inspect op1:
				case Left(l1)
					[2] $1 = inspect op2:
						case Left(l2)
							[3] $1 = sig call#eqLeft(l1@_, l2@_)
						case Right($5)
							[4] $1 = pack(Move)#sys.Bool.Bool[]|False()
							discard eqLeft@_
							[5] $1  = return ($1)
					[6] $1#5 = fetch(Copy) $1
					[7] #0  = return ($1#5@6)
					discard eqRight@_
					[8] #0  = return (#0)
				case Right(r1)
					[9] $6 = inspect op2:
						case Left($8)
							[10] $6 = pack(Move)#sys.Bool.Bool[]|False()
							discard eqRight@_
							[11] $6  = return ($6)
						case Right(r2)
							[12] $6 = sig call#eqRight(r1@_, r2@_)
					[13] $6#6 = fetch(Copy) $6
					[14] #0  = return ($6#6@13)
					discard eqLeft@_
					[15] #0  = return (#0)
IMPLEMENTS
	IMPLEMENT: EqForEither$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[std.Either.Either[L,R]]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: L
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: R
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: eqLeft
				TYPE: core.Equal.eq[L]
				CONSUME: true
			PARAM: eqRight
				TYPE: core.Equal.eq[R]
				CONSUME: true
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[std.Either.Either[L,R]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Either.eq[L,R](op1@_, op2@_, eqLeft@_, eqRight@_)
