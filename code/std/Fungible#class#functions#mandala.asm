NAME: Fungible
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, class, functions)
DATA TYPES
SIGNATURES
FUNCTIONS
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
			PARAM: item
				TYPE: T
				CONSUME: true
			PARAM: split
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: reminder
				TYPE: T
			RETURN: extracted
				TYPE: T
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
			PARAM: item1
				TYPE: T
				CONSUME: true
			PARAM: item2
				TYPE: T
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: T
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
			PARAM: item
				TYPE: T
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.IntU128.U128[]
IMPLEMENTS
