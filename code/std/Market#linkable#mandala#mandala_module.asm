NAME: Market
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Offer
		EXTERNAL: false
		ACCESS
			Create: Global
			Inspect: Global
			Consume: Local
		CAPABILITIES:  Value Unbound Persist
		GENERICS
			GENERIC: O
				PHANTOM: false
				CAPABILITIES: 
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value
		CONSTRUCTORS
			CONSTRUCTOR: Offer
				FIELDS
					FIELD: offer
						TYPE: O
					FIELD: owner
						TYPE: std.Subject.Subject[]
					FIELD: pay
						TYPE: projected(typeParam(1))
SIGNATURES
FUNCTIONS
	FUNCTION: offer
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: O
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: offer
				TYPE: O
				CONSUME: true
			PARAM: owner
				TYPE: std.Subject.Subject[]
				CONSUME: false
			PARAM: id
				TYPE: sys.Ids.PrivateId[]
				CONSUME: false
			PARAM: pay
				TYPE: projected(typeParam(1))
				CONSUME: false
		RETURNS
			RETURN: $5
				TYPE: sys.Sys.Entry[std.Market.Offer[O,P]]
		CODE
			[1] owner#13 = fetch(Copy) owner
			[2] pay#14 = fetch(Copy) pay
			[3] $1 = pack(Move)#std.Market.Offer[O,P]|Offer(offer@_, owner#13@_, pay#14@_)
			[4] id#15 = fetch(Copy) id
			[5] $5 = pack(Move)#sys.Sys.Entry[std.Market.Offer[O,P]]|Entry(id#15@_, $1@_)
	FUNCTION: request
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: O
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: entry
				TYPE: sys.Sys.Entry[std.Market.Offer[O,P]]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(typeParam(1))
		CODE
			[1] #0 = inspect entry:
				case Entry($1, val)
					[2] #0 = field#pay(Copy) val
	FUNCTION: owner
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: O
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: entry
				TYPE: sys.Sys.Entry[std.Market.Offer[O,P]]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: std.Subject.Subject[]
		CODE
			[1] #0 = inspect entry:
				case Entry($1, val)
					[2] #0 = field#owner(Copy) val
	FUNCTION: accept
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: O
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: entry
				TYPE: sys.Sys.Entry[std.Market.Offer[O,P]]
				CONSUME: true
			PARAM: pay
				TYPE: P
				CONSUME: true
			PARAM: eq
				TYPE: core.Equal.eq[P]
				CONSUME: true
		RETURNS
			RETURN: o
				TYPE: O
			RETURN: res
				TYPE: sys.Sys.Entry[std.SubjectLock.Locked[P]]
		CODE
			[1] $0 = let:
				[2] $0 = fetch(Move) entry
			[3] loc, $1 = unpack(Move) $0
			[4] off, owner, p = unpack(Move) $1
			[5] $3 = project pay
			[6] $2 = call#core.Projected.projectedEq[P]($3@_, p@_, eq@_)
			[7] o, res = switch(Move) $2:
				case False()
					[8] o, res = rollback(off@_, pay@_):(O, sys.Sys.Entry[std.SubjectLock.Locked[P]])
				case True()
					[9] $7 = call#std.SubjectLock.lockEntry[P](owner@_, loc@_, pay@_)
					[10] o, res  = return (off@_, $7@_)
	FUNCTION: withdraw
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: O
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
			GENERIC: P
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: entry
				TYPE: sys.Sys.Entry[std.Market.Offer[O,P]]
				CONSUME: true
			PARAM: auth
				TYPE: std.Capability.Perm[std.Subject.Subject[],std.Subject.Authorize[]]
				CONSUME: false
		RETURNS
			RETURN: id
				TYPE: sys.Ids.PrivateId[]
			RETURN: val
				TYPE: O
		CODE
			[1] $0 = let:
				[2] $0 = fetch(Move) entry
			[3] loc, $1 = unpack(Move) $0
			[4] off, owner, $2 = unpack(Move) $1
			[5] $3 = call#std.Subject.checkAuthorization[](owner@_, auth@_)
			[6] id, val = switch(Move) $3:
				case False()
					[7] id, val = rollback(off@_):(sys.Ids.PrivateId[], O)
				case True()
					[8] loc#16 = fetch(Copy) loc
					[9] id, val  = return (loc#16@_, off@_)
IMPLEMENTS
