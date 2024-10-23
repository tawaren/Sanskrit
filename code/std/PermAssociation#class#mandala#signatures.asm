NAME: PermAssociation
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, class, signatures)
DATA TYPES
SIGNATURES
	SIGNATURE: perm
		ACCESS
			Call: Global
			Define: Global
		TRANSACTIONAL: false
		CAPABILITIES:  Drop
		GENERICS
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: PS
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[G,PS]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: std.Capability.Perm[G,P]
FUNCTIONS
IMPLEMENTS
