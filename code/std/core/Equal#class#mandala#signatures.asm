NAME: Equal
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, class, signatures)
DATA TYPES
SIGNATURES
	SIGNATURE: eq
		ACCESS
			Define: Guarded(Set(T))
			Call: Global
		TRANSACTIONAL: false
		CAPABILITIES:  Drop
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
FUNCTIONS
IMPLEMENTS
