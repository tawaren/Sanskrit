NAME: PermSet4
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: PermSet4
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
			GENERIC: P3
				PHANTOM: true
				CAPABILITIES: 
			GENERIC: P4
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
			GENERIC: P3
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P4
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[G,std.PermSet4.PermSet4[P1,P2,P3,P4]]
				CONSUME: false
		RETURNS
			RETURN: $1
				TYPE: std.Capability.Perm[G,P1]
		CODE
			[1] $1 = call#std.Capability.createPerm[G,P1,std.PermSet4.PermSet4[P1,P2,P3,P4]](cap@_)
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
			GENERIC: P3
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P4
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[G,std.PermSet4.PermSet4[P1,P2,P3,P4]]
				CONSUME: false
		RETURNS
			RETURN: $1
				TYPE: std.Capability.Perm[G,P2]
		CODE
			[1] $1 = call#std.Capability.createPerm[G,P2,std.PermSet4.PermSet4[P1,P2,P3,P4]](cap@_)
	FUNCTION: perm3
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
			GENERIC: P3
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P4
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[G,std.PermSet4.PermSet4[P1,P2,P3,P4]]
				CONSUME: false
		RETURNS
			RETURN: $1
				TYPE: std.Capability.Perm[G,P3]
		CODE
			[1] $1 = call#std.Capability.createPerm[G,P3,std.PermSet4.PermSet4[P1,P2,P3,P4]](cap@_)
	FUNCTION: perm4
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
			GENERIC: P3
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P4
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[G,std.PermSet4.PermSet4[P1,P2,P3,P4]]
				CONSUME: false
		RETURNS
			RETURN: $1
				TYPE: std.Capability.Perm[G,P4]
		CODE
			[1] $1 = call#std.Capability.createPerm[G,P4,std.PermSet4.PermSet4[P1,P2,P3,P4]](cap@_)
IMPLEMENTS
	IMPLEMENT: Perm1Select$perm
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.PermAssociation.perm[P1,std.PermSet4.PermSet4[P1,P2,P3,P4],G]
		ACCESS
			Call: Global
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
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: perm
				TYPE: std.PermAssociation.perm[P1,std.PermSet4.PermSet4[P1,P2,P3,P4],G]
		BINDINGS
			PARAMS
				BINDING: cap
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.PermSet4.perm1[G,P1,P2,P3,P4](cap@_)
	IMPLEMENT: Perm2Select$perm
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.PermAssociation.perm[P2,std.PermSet4.PermSet4[P1,P2,P3,P4],G]
		ACCESS
			Call: Global
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
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: perm
				TYPE: std.PermAssociation.perm[P2,std.PermSet4.PermSet4[P1,P2,P3,P4],G]
		BINDINGS
			PARAMS
				BINDING: cap
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.PermSet4.perm2[G,P1,P2,P3,P4](cap@_)
	IMPLEMENT: Perm3Select$perm
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.PermAssociation.perm[P3,std.PermSet4.PermSet4[P1,P2,P3,P4],G]
		ACCESS
			Call: Global
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
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: perm
				TYPE: std.PermAssociation.perm[P3,std.PermSet4.PermSet4[P1,P2,P3,P4],G]
		BINDINGS
			PARAMS
				BINDING: cap
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.PermSet4.perm3[G,P1,P2,P3,P4](cap@_)
	IMPLEMENT: Perm4Select$perm
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.PermAssociation.perm[P4,std.PermSet4.PermSet4[P1,P2,P3,P4],G]
		ACCESS
			Call: Global
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
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: perm
				TYPE: std.PermAssociation.perm[P4,std.PermSet4.PermSet4[P1,P2,P3,P4],G]
		BINDINGS
			PARAMS
				BINDING: cap
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.PermSet4.perm4[G,P1,P2,P3,P4](cap@_)
