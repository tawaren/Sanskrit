NAME: Compare
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, class, functions)
DATA TYPES
SIGNATURES
FUNCTIONS
	FUNCTION: lt
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
				TYPE: T
				CONSUME: false
			PARAM: op2
				TYPE: T
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
	FUNCTION: lte
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
				TYPE: T
				CONSUME: false
			PARAM: op2
				TYPE: T
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
	FUNCTION: gt
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
				TYPE: T
				CONSUME: false
			PARAM: op2
				TYPE: T
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
	FUNCTION: gte
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
				TYPE: T
				CONSUME: false
			PARAM: op2
				TYPE: T
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
IMPLEMENTS
