NAME: PermSet7$Perm6Select
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: PermAssociation
	PARAMS
		PARAM: P6
		PARAM: std.PermSet7.PermSet7[P1,P2,P3,P4,P5,P6,P7]
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
			GENERIC: P5
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P6
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P7
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.PermSet7.perm6[G,P1,P2,P3,P4,P5,P6,P7]
		IMPL_TARGET: std.PermSet7.Perm6Select$perm[P1,P2,P3,P4,P5,P6,P7,G]