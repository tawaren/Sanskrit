NAME: PermSet1$Perm1Select
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: PermAssociation
	PARAMS
		PARAM: P
		PARAM: std.PermSet1.PermSet1[P]
IMPLEMENTS
	IMPLEMENT: perm
		GENERICS
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.PermSet1.perm1[G,P]
		IMPL_TARGET: std.PermSet1.Perm1Select$perm[P,G]
