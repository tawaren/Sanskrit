NAME: PermSet12$Perm8Select
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: PermAssociation
	PARAMS
		PARAM: P8
		PARAM: std.PermSet12.PermSet12[P1,P2,P3,P4,P5,P6,P7,P8,P9,P10,P11,P12]
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
			GENERIC: P12
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.PermSet12.perm8[G,P1,P2,P3,P4,P5,P6,P7,P8,P9,P10,P11,P12]
		IMPL_TARGET: std.PermSet12.Perm8Select$perm[P1,P2,P3,P4,P5,P6,P7,P8,P9,P10,P11,P12,G]
