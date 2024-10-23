NAME: Arith
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, class, signatures)
DATA TYPES
SIGNATURES
	SIGNATURE: add
		ACCESS
			Define: Guarded(Set(T))
			Call: Global
		TRANSACTIONAL: true
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
				TYPE: T
	SIGNATURE: sub
		ACCESS
			Define: Guarded(Set(T))
			Call: Global
		TRANSACTIONAL: true
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
				TYPE: T
	SIGNATURE: mul
		ACCESS
			Define: Guarded(Set(T))
			Call: Global
		TRANSACTIONAL: true
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
				TYPE: T
	SIGNATURE: div
		ACCESS
			Define: Guarded(Set(T))
			Call: Global
		TRANSACTIONAL: true
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
				TYPE: T
FUNCTIONS
IMPLEMENTS
