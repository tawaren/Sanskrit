NAME: PermSet4$Perm3Select
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: PermAssociation
	PARAMS
		PARAM: P3
		PARAM: std.PermSet4.PermSet4[P1,P2,P3,P4]
IMPLEMENTS
	IMPLEMENT: perm
		GENERICS
			GENERIC: P1
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P2
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P3
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P4
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.PermSet4.perm3[G,P1,P2,P3,P4]
		IMPL_TARGET: std.PermSet4.Perm3Select$perm[P1,P2,P3,P4,G]
