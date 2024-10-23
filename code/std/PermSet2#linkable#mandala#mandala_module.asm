NAME: PermSet2
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: PermSet2
		EXTERNAL: false
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
			GENERIC: P1
				PHANTOM: true
				CAPABILITIES: 
			GENERIC: P2
				PHANTOM: true
				CAPABILITIES: 
SIGNATURES
FUNCTIONS
	FUNCTION: perm1
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P1
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P2
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[G,std.PermSet2.PermSet2[P1,P2]]
				CONSUME: false
		RETURNS
			RETURN: $1
				TYPE: std.Capability.Perm[G,P1]
		CODE
			[1] $1 = call#std.Capability.createPerm[G,P1,std.PermSet2.PermSet2[P1,P2]](cap@_)
	FUNCTION: perm2
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P1
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P2
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[G,std.PermSet2.PermSet2[P1,P2]]
				CONSUME: false
		RETURNS
			RETURN: $1
				TYPE: std.Capability.Perm[G,P2]
		CODE
			[1] $1 = call#std.Capability.createPerm[G,P2,std.PermSet2.PermSet2[P1,P2]](cap@_)
IMPLEMENTS
	IMPLEMENT: Perm1Select$perm
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.PermAssociation.perm[P1,std.PermSet2.PermSet2[P1,P2],G]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: P1
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P2
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: perm
				TYPE: std.PermAssociation.perm[P1,std.PermSet2.PermSet2[P1,P2],G]
		BINDINGS
			PARAMS
				BINDING: cap
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.PermSet2.perm1[G,P1,P2](cap@_)
	IMPLEMENT: Perm2Select$perm
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.PermAssociation.perm[P2,std.PermSet2.PermSet2[P1,P2],G]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: P1
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P2
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: perm
				TYPE: std.PermAssociation.perm[P2,std.PermSet2.PermSet2[P1,P2],G]
		BINDINGS
			PARAMS
				BINDING: cap
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.PermSet2.perm2[G,P1,P2](cap@_)
