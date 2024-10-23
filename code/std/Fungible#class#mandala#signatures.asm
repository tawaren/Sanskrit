NAME: Fungible
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, class, signatures)
DATA TYPES
SIGNATURES
	SIGNATURE: split
		ACCESS
			Call: Global
			Define: Global
		TRANSACTIONAL: true
		CAPABILITIES:  Drop
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
	SIGNATURE: merge
		ACCESS
			Call: Global
			Define: Global
		TRANSACTIONAL: true
		CAPABILITIES:  Drop
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
	SIGNATURE: amount
		ACCESS
			Call: Global
			Define: Global
		TRANSACTIONAL: false
		CAPABILITIES:  Drop
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
FUNCTIONS
IMPLEMENTS
