NAME: NodeUtils
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, class, signatures)
DATA TYPES
SIGNATURES
	SIGNATURE: single
		ACCESS
			Call: Global
			Define: Global
		TRANSACTIONAL: false
		CAPABILITIES:  Drop
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: N
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: head
				TYPE: E
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: N
	SIGNATURE: inspection
		ACCESS
			Call: Global
			Define: Global
		TRANSACTIONAL: false
		CAPABILITIES:  Drop
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: N
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: lst
				TYPE: N
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.NodeUtils.InspectResult[E,N]
	SIGNATURE: insertion
		ACCESS
			Call: Global
			Define: Global
		TRANSACTIONAL: false
		CAPABILITIES:  Drop
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: N
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: head
				TYPE: E
				CONSUME: true
			PARAM: tail
				TYPE: N
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.NodeUtils.InjectResult[N]
FUNCTIONS
IMPLEMENTS
