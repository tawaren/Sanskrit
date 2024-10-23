NAME: NodeUtils
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, class, functions)
DATA TYPES
SIGNATURES
FUNCTIONS
	FUNCTION: single
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
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
	FUNCTION: inspection
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
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
	FUNCTION: insertion
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
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
IMPLEMENTS
