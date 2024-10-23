NAME: PermSet1
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: PermSet1
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
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[G,std.PermSet1.PermSet1[P1]]
				CONSUME: false
		RETURNS
			RETURN: $1
				TYPE: std.Capability.Perm[G,P1]
		CODE
			[1] $1 = call#std.Capability.createPerm[G,P1,std.PermSet1.PermSet1[P1]](cap@_)
IMPLEMENTS
	IMPLEMENT: Perm1Select$perm
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.PermAssociation.perm[P,std.PermSet1.PermSet1[P],G]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: perm
				TYPE: std.PermAssociation.perm[P,std.PermSet1.PermSet1[P],G]
		BINDINGS
			PARAMS
				BINDING: cap
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.PermSet1.perm1[G,P](cap@_)
