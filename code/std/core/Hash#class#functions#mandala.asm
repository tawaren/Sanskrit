NAME: Hash
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, class, functions)
DATA TYPES
SIGNATURES
FUNCTIONS
	FUNCTION: hash
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: op
				TYPE: T
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	FUNCTION: joinHash
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
				TYPE: sys.Data.Data20[]
IMPLEMENTS
