NAME: Permission$ToPerm
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: PermAssociation
	PARAMS
		PARAM: P
		PARAM: P
IMPLEMENTS
	IMPLEMENT: perm
		GENERICS
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Capability.toPerm[G,P]
		IMPL_TARGET: std.Permission.ToPerm$perm[P,G]
