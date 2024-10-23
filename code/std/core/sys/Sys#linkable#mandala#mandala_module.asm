NAME: Sys
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Entry
		EXTERNAL: false
		ACCESS
			Create: Guarded(Set(T))
			Consume: Guarded(Set(T))
			Inspect: Guarded(Set(T))
		CAPABILITIES:  Value Unbound Copy Drop
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Persist
		CONSTRUCTORS
			CONSTRUCTOR: Entry
				FIELDS
					FIELD: id
						TYPE: sys.Ids.PrivateId[]
					FIELD: val
						TYPE: T
	DATA TYPE: Context
		EXTERNAL: false
		ACCESS
			Create: Local
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Copy Drop
		GENERICS
		CONSTRUCTORS
			CONSTRUCTOR: Context
				FIELDS
					FIELD: bundleHash
						TYPE: sys.Data.Data20[]
					FIELD: blockNo
						TYPE: sys.IntU64.U64[]
					FIELD: sectionNo
						TYPE: sys.IntU8.U8[]
					FIELD: txNo
						TYPE: sys.IntU8.U8[]
	DATA TYPE: IdGenerator
		EXTERNAL: false
		ACCESS
			Create: Local
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Drop
		GENERICS
		CONSTRUCTORS
			CONSTRUCTOR: IdGenerator
				FIELDS
					FIELD: txtId
						TYPE: sys.Ids.PrivateId[]
					FIELD: ctr
						TYPE: sys.IntU64.U64[]
	DATA TYPE: Sender
		EXTERNAL: false
		ACCESS
			Consume: Global
			Inspect: Global
			Create: Local
		CAPABILITIES:  Value Unbound Copy Drop
		GENERICS
		CONSTRUCTORS
			CONSTRUCTOR: Sender
				FIELDS
					FIELD: senderId
						TYPE: sys.Ids.PrivateId[]
SIGNATURES
FUNCTIONS
	FUNCTION: entryId
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: entry
				TYPE: sys.Sys.Entry[T]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(sys.Ids.PrivateId[])
		CODE
			[1] $0 = field#id(Copy) entry
			[2] #0 = call#sys.Ids.idFromPrivate[]($0@_)
	FUNCTION: bundleHash
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: ctx
				TYPE: sys.Sys.Context[]
				CONSUME: false
		RETURNS
			RETURN: txtHash
				TYPE: sys.Data.Data20[]
		CODE
			[1] txtHash = field#bundleHash(Copy) ctx
	FUNCTION: blockNo
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: ctx
				TYPE: sys.Sys.Context[]
				CONSUME: false
		RETURNS
			RETURN: blockNo
				TYPE: sys.IntU64.U64[]
		CODE
			[1] blockNo = field#blockNo(Copy) ctx
	FUNCTION: sectionNo
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: ctx
				TYPE: sys.Sys.Context[]
				CONSUME: false
		RETURNS
			RETURN: sectionNo
				TYPE: sys.IntU8.U8[]
		CODE
			[1] sectionNo = field#sectionNo(Copy) ctx
	FUNCTION: txNo
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: ctx
				TYPE: sys.Sys.Context[]
				CONSUME: false
		RETURNS
			RETURN: txNo
				TYPE: sys.IntU8.U8[]
		CODE
			[1] txNo = field#txNo(Copy) ctx
	FUNCTION: txtId
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: gen
				TYPE: sys.Sys.IdGenerator[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(sys.Ids.PrivateId[])
		CODE
			[1] $0 = field#txtId(Copy) gen
			[2] #0 = call#sys.Ids.idFromPrivate[]($0@_)
	FUNCTION: uniqueID
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: gen
				TYPE: sys.Sys.IdGenerator[]
				CONSUME: true
		RETURNS
			RETURN: id
				TYPE: sys.Ids.PrivateId[]
			RETURN: genOut
				TYPE: sys.Sys.IdGenerator[]
		CODE
			[1] $0 = let:
				[2] $0 = fetch(Move) gen
			[3] txtId, ctr = unpack(Move) $0
			[4] $3 = call#sys.IntU64.hash[](ctr@_)
			[5] $1 = call#sys.Ids.privateIdDerive[](txtId@_, $3@_)
			[6] $9 = lit 0x0000000000000001
			[7] $7 = call#sys.IntU64.add[](ctr@_, $9@_)
			[8] txtId#1 = fetch(Copy) txtId
			[9] $5 = pack(Move)#sys.Sys.IdGenerator[]|IdGenerator(txtId#1@_, $7@_)
			[10] id, genOut  = return ($1@_, $5@_)
	FUNCTION: subGenerator
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: gen
				TYPE: sys.Sys.IdGenerator[]
				CONSUME: true
		RETURNS
			RETURN: newGen
				TYPE: sys.Sys.IdGenerator[]
			RETURN: genOut
				TYPE: sys.Sys.IdGenerator[]
		CODE
			[1] $0 = let:
				[2] $0 = fetch(Move) gen
			[3] txtId, ctr = unpack(Move) $0
			[4] $4 = call#sys.IntU64.hash[](ctr@_)
			[5] $2 = call#sys.Ids.privateIdDerive[](txtId@_, $4@_)
			[6] $6 = lit 0x0000000000000000
			[7] $6#2 = fetch(Copy) $6
			[8] $1 = pack(Move)#sys.Sys.IdGenerator[]|IdGenerator($2@_, $6#2@7)
			[9] $11 = lit 0x0000000000000001
			[10] $9 = call#sys.IntU64.add[](ctr@_, $11@_)
			[11] txtId#3 = fetch(Copy) txtId
			[12] $7 = pack(Move)#sys.Sys.IdGenerator[]|IdGenerator(txtId#3@_, $9@_)
			[13] newGen, genOut  = return ($1@_, $7@_)
IMPLEMENTS
