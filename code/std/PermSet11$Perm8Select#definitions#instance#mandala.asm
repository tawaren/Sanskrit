NAME: PermSet11$Perm8Select
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: PermAssociation
	PARAMS
		PARAM: P8
		PARAM: std.PermSet11.PermSet11[P1,P2,P3,P4,P5,P6,P7,P8,P9,P10,P11]
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
			GENERIC: P8
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P9
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P10
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P11
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.PermSet11.perm8[G,P1,P2,P3,P4,P5,P6,P7,P8,P9,P10,P11]
		IMPL_TARGET: std.PermSet11.Perm8Select$perm[P1,P2,P3,P4,P5,P6,P7,P8,P9,P10,P11,G]
