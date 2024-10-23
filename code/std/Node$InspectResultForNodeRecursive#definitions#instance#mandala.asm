NAME: Node$InspectResultForNodeRecursive
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: NodeUtils
	PARAMS
		PARAM: E
		PARAM: std.Node.Node[N]
IMPLEMENTS
	IMPLEMENT: single
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: N
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Node.singleRec[E,N]
		IMPL_TARGET: std.Node.InspectResultForNodeRecursive$single[E,N]
	IMPLEMENT: inspection
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: N
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Node.inspectionRec[E,N]
		IMPL_TARGET: std.Node.InspectResultForNodeRecursive$inspection[E,N]
	IMPLEMENT: insertion
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: N
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Node.insertionRec[E,N]
		IMPL_TARGET: std.Node.InspectResultForNodeRecursive$insertion[E,N]
