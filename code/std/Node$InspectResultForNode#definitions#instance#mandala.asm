NAME: Node$InspectResultForNode
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: NodeUtils
	PARAMS
		PARAM: E
		PARAM: std.Node.Node[E]
IMPLEMENTS
	IMPLEMENT: single
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Node.single[E]
		IMPL_TARGET: std.Node.InspectResultForNode$single[E]
	IMPLEMENT: inspection
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Node.inspection[E]
		IMPL_TARGET: std.Node.InspectResultForNode$inspection[E]
	IMPLEMENT: insertion
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Node.insertion[E]
		IMPL_TARGET: std.Node.InspectResultForNode$insertion[E]
