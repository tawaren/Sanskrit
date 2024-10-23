NAME: Purse
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Purse
		EXTERNAL: false
		ACCESS
			Create: Local
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Persist
		GENERICS
			GENERIC: F
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Purse
				FIELDS
					FIELD: f
						TYPE: F
	DATA TYPE: Master
		EXTERNAL: false
		ACCESS
			Create: Local
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
		CONSTRUCTORS
			CONSTRUCTOR: Master
				FIELDS
	DATA TYPE: Deposit
		EXTERNAL: false
		ACCESS
			Create: Local
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
		CONSTRUCTORS
			CONSTRUCTOR: Deposit
				FIELDS
	DATA TYPE: Withdraw
		EXTERNAL: false
		ACCESS
			Create: Local
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
		CONSTRUCTORS
			CONSTRUCTOR: Withdraw
				FIELDS
SIGNATURES
FUNCTIONS
	FUNCTION: createDepositCap
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: F
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[std.Purse.Purse[F],std.Purse.Master[]]
				CONSUME: false
		RETURNS
			RETURN: $2
				TYPE: std.Capability.Cap[std.Purse.Purse[F],std.Purse.Deposit[]]
		CODE
			[1] $0 = field#id(Copy) cap
			[2] $2 = pack(Copy)#std.Capability.Cap[std.Purse.Purse[F],std.Purse.Deposit[]]|Cap($0)
	FUNCTION: createWithdrawCap
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: F
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[std.Purse.Purse[F],std.Purse.Master[]]
				CONSUME: false
		RETURNS
			RETURN: $2
				TYPE: std.Capability.Cap[std.Purse.Purse[F],std.Purse.Withdraw[]]
		CODE
			[1] $0 = field#id(Copy) cap
			[2] $2 = pack(Copy)#std.Capability.Cap[std.Purse.Purse[F],std.Purse.Withdraw[]]|Cap($0)
	FUNCTION: createDepositPerm
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: PF
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[PF,std.Purse.Master[]]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: std.Capability.Perm[PF,std.Purse.Deposit[]]
		CODE
			[1] #0 = call#std.Capability.createPerm[PF,std.Purse.Deposit[],std.Purse.Master[]](cap@_)
	FUNCTION: createWithdrawPerm
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: PF
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: cap
				TYPE: std.Capability.Cap[PF,std.Purse.Master[]]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: std.Capability.Perm[PF,std.Purse.Withdraw[]]
		CODE
			[1] #0 = call#std.Capability.createPerm[PF,std.Purse.Withdraw[],std.Purse.Master[]](cap@_)
	FUNCTION: createPurse
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: F
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: f
				TYPE: F
				CONSUME: true
			PARAM: gen
				TYPE: sys.Sys.IdGenerator[]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: sys.Sys.Entry[std.Purse.Purse[F]]
			RETURN: #1
				TYPE: std.Capability.Cap[std.Purse.Purse[F],std.Purse.Master[]]
			RETURN: #2
				TYPE: sys.Sys.IdGenerator[]
		CODE
			[1] id, newGen = let:
				[2] id, newGen = call#sys.Sys.uniqueID[](gen@_)
			[3] $3 = pack(Move)#std.Purse.Purse[F]|Purse(f@_)
			[4] id#23 = fetch(Copy) id
			[5] $1 = pack(Move)#sys.Sys.Entry[std.Purse.Purse[F]]|Entry(id#23@_, $3@_)
			[6] $6 = call#sys.Ids.idFromPrivate[](id@_)
			[7] $5 = pack(Move)#std.Capability.Cap[std.Purse.Purse[F],std.Purse.Master[]]|Cap($6@_)
			[8] $5#24 = fetch(Copy) $5
			[9] #0, #1, #2  = return ($1@_, $5#24@8, newGen@_)
	FUNCTION: dispose
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: F
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: purse
				TYPE: std.Capability.Handle[std.Purse.Purse[F],std.Purse.Master[]]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: F
		CODE
			[1] entry, $0 = let:
				[2] entry, $0 = call#std.Capability.extractHandle[std.Purse.Purse[F],std.Purse.Master[]](purse@_)
			[3] $2 = field#val(Move) entry
			[4] #0 = field#f(Move) $2
	FUNCTION: deposit
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: F
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: purse
				TYPE: std.Capability.Handle[std.Purse.Purse[F],std.Purse.Deposit[]]
				CONSUME: true
			PARAM: f
				TYPE: F
				CONSUME: true
			PARAM: merge
				TYPE: std.Fungible.merge[F]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.Capability.Handle[std.Purse.Purse[F],std.Purse.Deposit[]]
		CODE
			[1] $0, perm = let:
				[2] $0, perm = call#std.Capability.extractHandle[std.Purse.Purse[F],std.Purse.Deposit[]](purse@_)
			[3] id, p = unpack(Move) $0
			[4] $6 = field#f(Move) p
			[5] $5 = sig call#merge($6@_, f@_)
			[6] $4 = pack(Move)#std.Purse.Purse[F]|Purse($5@5)
			[7] id#25 = fetch(Copy) id
			[8] $2 = pack(Move)#sys.Sys.Entry[std.Purse.Purse[F]]|Entry(id#25@_, $4@_)
			[9] #0 = call#std.Capability.createHandle[std.Purse.Purse[F],std.Purse.Deposit[]]($2@_, perm@_)
	FUNCTION: withdraw
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: F
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: purse
				TYPE: std.Capability.Handle[std.Purse.Purse[F],std.Purse.Withdraw[]]
				CONSUME: true
			PARAM: amount
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: split
				TYPE: std.Fungible.split[F]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.Capability.Handle[std.Purse.Purse[F],std.Purse.Withdraw[]]
			RETURN: #1
				TYPE: F
		CODE
			[1] $0, perm = let:
				[2] $0, perm = call#std.Capability.extractHandle[std.Purse.Purse[F],std.Purse.Withdraw[]](purse@_)
			[3] id, p = unpack(Move) $0
			[4] rem, ext = let:
				[5] $2 = field#f(Move) p
				[6] rem, ext = sig call#split($2@_, amount@_)
			[7] $8 = pack(Move)#std.Purse.Purse[F]|Purse(rem@_)
			[8] id#26 = fetch(Copy) id
			[9] $6 = pack(Move)#sys.Sys.Entry[std.Purse.Purse[F]]|Entry(id#26@_, $8@_)
			[10] $5 = call#std.Capability.createHandle[std.Purse.Purse[F],std.Purse.Withdraw[]]($6@_, perm@_)
			[11] #0, #1  = return ($5@_, ext@_)
IMPLEMENTS
	IMPLEMENT: DepositPermSelect$perm
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.PermAssociation.perm[std.Purse.Deposit[],std.Purse.Master[],PF]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: PF
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: perm
				TYPE: std.PermAssociation.perm[std.Purse.Deposit[],std.Purse.Master[],PF]
		BINDINGS
			PARAMS
				BINDING: cap
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Purse.createDepositPerm[PF](cap@_)
	IMPLEMENT: WithdrawPermSelect$perm
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.PermAssociation.perm[std.Purse.Withdraw[],std.Purse.Master[],PF]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: PF
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: perm
				TYPE: std.PermAssociation.perm[std.Purse.Withdraw[],std.Purse.Master[],PF]
		BINDINGS
			PARAMS
				BINDING: cap
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Purse.createWithdrawPerm[PF](cap@_)
