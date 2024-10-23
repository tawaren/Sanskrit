NAME: Capability
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Cap
		EXTERNAL: false
		ACCESS
			Inspect: Global
			Consume: Global
			Create: Guarded(Set(G))
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
			GENERIC: G
				PHANTOM: true
				CAPABILITIES: 
			GENERIC: PD
				PHANTOM: true
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Cap
				FIELDS
					FIELD: id
						TYPE: projected(sys.Ids.PrivateId[])
	DATA TYPE: Perm
		EXTERNAL: false
		ACCESS
			Inspect: Global
			Consume: Global
			Create: Local
		CAPABILITIES:  Value Unbound Copy Drop
		GENERICS
			GENERIC: G
				PHANTOM: true
				CAPABILITIES: 
			GENERIC: P
				PHANTOM: true
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Perm
				FIELDS
					FIELD: id
						TYPE: projected(sys.Ids.PrivateId[])
	DATA TYPE: Handle
		EXTERNAL: false
		ACCESS
			Inspect: Global
			Consume: Global
			Create: Local
		CAPABILITIES:  Value Unbound Copy Drop
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Persist
			GENERIC: PD
				PHANTOM: true
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Handle
				FIELDS
					FIELD: entry
						TYPE: sys.Sys.Entry[G]
SIGNATURES
FUNCTIONS
	FUNCTION: capId
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: PD
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[G,PD]
				CONSUME: false
		RETURNS
			RETURN: $1
				TYPE: projected(sys.Ids.PrivateId[])
		CODE
			[1] $1 = field#id(Copy) cap
	FUNCTION: permId
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: perm
				TYPE: std.Capability.Perm[G,P]
				CONSUME: false
		RETURNS
			RETURN: $1
				TYPE: projected(sys.Ids.PrivateId[])
		CODE
			[1] $1 = field#id(Copy) perm
	FUNCTION: handleId
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: handle
				TYPE: std.Capability.Handle[G,P]
				CONSUME: false
		RETURNS
			RETURN: $3
				TYPE: projected(sys.Ids.PrivateId[])
		CODE
			[1] $3 = inspect handle:
				case Handle(entry)
					[2] $2 = call#sys.Sys.entryId[G](entry@_)
	FUNCTION: createCap
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Guarded(Set(G))
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: id
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
		RETURNS
			RETURN: $1
				TYPE: std.Capability.Cap[G,P]
		CODE
			[1] $1 = pack(Copy)#std.Capability.Cap[G,P]|Cap(id)
	FUNCTION: createEntryCap
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Guarded(Set(G))
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: entry
				TYPE: sys.Sys.Entry[G]
				CONSUME: false
		RETURNS
			RETURN: $2
				TYPE: std.Capability.Cap[G,P]
		CODE
			[1] $0 = call#sys.Sys.entryId[G](entry@_)
			[2] $2 = pack(Move)#std.Capability.Cap[G,P]|Cap($0@_)
	FUNCTION: createPerm
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Guarded(Set(PD))
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: PD
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[G,PD]
				CONSUME: false
		RETURNS
			RETURN: $2
				TYPE: std.Capability.Perm[G,P]
		CODE
			[1] $0 = call#std.Capability.capId[G,PD](cap@_)
			[2] $2 = pack(Move)#std.Capability.Perm[G,P]|Perm($0@_)
	FUNCTION: toPerm
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[G,P]
				CONSUME: false
		RETURNS
			RETURN: $2
				TYPE: std.Capability.Perm[G,P]
		CODE
			[1] $0 = call#std.Capability.capId[G,P](cap@_)
			[2] $2 = pack(Move)#std.Capability.Perm[G,P]|Perm($0@_)
	FUNCTION: createHandle
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: entry
				TYPE: sys.Sys.Entry[G]
				CONSUME: true
			PARAM: perm
				TYPE: std.Capability.Perm[G,P]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: std.Capability.Handle[G,P]
		CODE
			[1] $1 = call#sys.Sys.entryId[G](entry@_)
			[2] $2 = call#std.Capability.permId[G,P](perm@_)
			[3] $0 = call#sys.Ids.eqId[]($1@_, $2@_)
			[4] #0 = switch(Move) $0:
				case False()
					[5] #0 = rollback(entry@_):(std.Capability.Handle[G,P])
				case True()
					[6] #0 = pack(Move)#std.Capability.Handle[G,P]|Handle(entry@_)
	FUNCTION: checkPerm
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
			GENERIC: P1
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: P2
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: handle
				TYPE: std.Capability.Handle[G,P1]
				CONSUME: false
			PARAM: perm
				TYPE: std.Capability.Perm[G,P2]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] $0 = call#std.Capability.handleId[G,P1](handle@_)
			[2] $1 = call#std.Capability.permId[G,P2](perm@_)
			[3] #0 = call#sys.Ids.eqId[]($0@_, $1@_)
	FUNCTION: extractHandle
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Guarded(Set(G))
		GENERICS
			GENERIC: G
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: $0
				TYPE: std.Capability.Handle[G,P]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: sys.Sys.Entry[G]
			RETURN: #1
				TYPE: std.Capability.Perm[G,P]
		CODE
			[1] entry = unpack(Move) $0
			[2] $3 = call#sys.Sys.entryId[G](entry@_)
			[3] $2 = pack(Move)#std.Capability.Perm[G,P]|Perm($3@_)
			[4] $2#4 = fetch(Copy) $2
			[5] #0, #1  = return (entry@_, $2#4@4)
IMPLEMENTS
