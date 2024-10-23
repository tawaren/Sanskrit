NAME: PermSet10$Perm7Select
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: PermAssociation
	PARAMS
		PARAM: P7
		PARAM: std.PermSet10.PermSet10[P1,P2,P3,P4,P5,P6,P7,P8,P9,P10]
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
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.PermSet10.perm7[G,P1,P2,P3,P4,P5,P6,P7,P8,P9,P10]
		IMPL_TARGET: std.PermSet10.Perm7Select$perm[P1,P2,P3,P4,P5,P6,P7,P8,P9,P10,G]
