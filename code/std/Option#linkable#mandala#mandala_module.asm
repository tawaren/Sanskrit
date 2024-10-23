NAME: Option
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Option
		EXTERNAL: false
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Some
				FIELDS
					FIELD: inner
						TYPE: T
			CONSTRUCTOR: None
				FIELDS
SIGNATURES
FUNCTIONS
	FUNCTION: isNone
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: op1
				TYPE: std.Option.Option[T]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] #0 = inspect op1:
				case Some($1)
					[2] #0 = pack(Move)#sys.Bool.Bool[]|False()
				case None()
					[3] #0 = pack(Move)#sys.Bool.Bool[]|True()
	FUNCTION: isSome
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: op1
				TYPE: std.Option.Option[T]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] #0 = inspect op1:
				case Some($1)
					[2] #0 = pack(Move)#sys.Bool.Bool[]|True()
				case None()
					[3] #0 = pack(Move)#sys.Bool.Bool[]|False()
	FUNCTION: optionEq
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: op1
				TYPE: std.Option.Option[T]
				CONSUME: false
			PARAM: op2
				TYPE: std.Option.Option[T]
				CONSUME: false
			PARAM: eqFun
				TYPE: core.Equal.eq[T]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] #0 = inspect op1:
				case Some(v1)
					[2] #0 = inspect op2:
						case Some(v2)
							[3] #0 = sig call#eqFun(v1@_, v2@_)
						case None()
							[4] #0 = pack(Move)#sys.Bool.Bool[]|False()
							discard eqFun@_
							[5] #0  = return (#0)
				case None()
					[6] #0 = call#std.Option.isNone[T](op2@_)
					discard eqFun@_
					[7] #0  = return (#0)
IMPLEMENTS
	IMPLEMENT: EqForOption$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[std.Option.Option[T]]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: eqFun
				TYPE: core.Equal.eq[T]
				CONSUME: true
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[std.Option.Option[T]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Option.optionEq[T](op1@_, op2@_, eqFun@_)
