NAME: PermSet3$Perm1Select
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: PermAssociation
	PARAMS
		PARAM: P1
		PARAM: std.PermSet3.PermSet3[P1,P2,P3]
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
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.PermSet3.perm1[G,P1,P2,P3]
		IMPL_TARGET: std.PermSet3.Perm1Select$perm[P1,P2,P3,G]
