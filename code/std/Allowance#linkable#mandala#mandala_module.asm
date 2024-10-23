NAME: Allowance
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Allowance
		EXTERNAL: false
		ACCESS
			Create: Global
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Persist Drop
		GENERICS
			GENERIC: F
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Allowance
				FIELDS
					FIELD: cap
						TYPE: std.Capability.Cap[std.Purse.Purse[F],std.Purse.Withdraw[]]
					FIELD: amount
						TYPE: sys.IntU128.U128[]
SIGNATURES
FUNCTIONS
	FUNCTION: amount
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: F
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: allowance
				TYPE: std.Allowance.Allowance[F]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.IntU128.U128[]
		CODE
			[1] #0 = field#amount(Copy) allowance
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
			PARAM: entry
				TYPE: sys.Sys.Entry[std.Purse.Purse[F]]
				CONSUME: true
			PARAM: amount
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: allowance
				TYPE: std.Allowance.Allowance[F]
				CONSUME: true
			PARAM: split
				TYPE: std.Fungible.split[F]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: sys.Sys.Entry[std.Purse.Purse[F]]
			RETURN: #1
				TYPE: std.Allowance.Allowance[F]
			RETURN: #2
				TYPE: F
		CODE
			[1] $0 = let:
				[2] $0 = fetch(Move) allowance
			[3] withdrawCap, remAmount = unpack(Move) $0
			[4] $1 = call#sys.IntU128.lte[](remAmount@_, amount@_)
			[5] #0, #1, #2 = switch(Move) $1:
				case False()
					[6] #0, #1, #2 = rollback(entry@_):(sys.Sys.Entry[std.Purse.Purse[F]], std.Allowance.Allowance[F], F)
					discard split@_
					[7] #0, #1, #2  = return (#0, #1, #2)
				case True()
					[8] handle = let:
						[9] $5 = call#std.Capability.toPerm[std.Purse.Purse[F],std.Purse.Withdraw[]](withdrawCap@_)
						[10] handle = call#std.Capability.createHandle[std.Purse.Purse[F],std.Purse.Withdraw[]](entry@_, $5@_)
					[11] $7, ext = let:
						[12] $7, ext = call#std.Purse.withdraw[F](handle@_, amount@_, split@_)
					[13] newPurse = unpack(Move) $7
					[14] newAllowance = let:
						[15] $11 = call#sys.IntU128.sub[](remAmount@_, amount@_)
						[16] withdrawCap#27 = fetch(Copy) withdrawCap
						[17] newAllowance = pack(Move)#std.Allowance.Allowance[F]|Allowance(withdrawCap#27@_, $11@_)
					[18] #0, #1, #2  = return (newPurse@_, newAllowance@_, ext@_)
IMPLEMENTS
