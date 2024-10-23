NAME: Hash
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, class, signatures)
DATA TYPES
SIGNATURES
	SIGNATURE: hash
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
			PARAM: op
				TYPE: T
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	SIGNATURE: joinHash
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
				TYPE: sys.Data.Data20[]
FUNCTIONS
IMPLEMENTS
