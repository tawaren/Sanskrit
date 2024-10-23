NAME: SubjectLock
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Locked
		EXTERNAL: false
		ACCESS
			Create: Global
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Locked
				FIELDS
					FIELD: owner
						TYPE: std.Subject.Subject[]
					FIELD: val
						TYPE: T
SIGNATURES
FUNCTIONS
	FUNCTION: lock
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: owner
				TYPE: std.Subject.Subject[]
				CONSUME: false
			PARAM: val
				TYPE: T
				CONSUME: true
		RETURNS
			RETURN: $2
				TYPE: std.SubjectLock.Locked[T]
		CODE
			[1] owner#7 = fetch(Copy) owner
			[2] $2 = pack(Move)#std.SubjectLock.Locked[T]|Locked(owner#7@_, val@_)
	FUNCTION: unlock
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: $0
				TYPE: std.SubjectLock.Locked[T]
				CONSUME: true
			PARAM: auth
				TYPE: std.Capability.Perm[std.Subject.Subject[],std.Subject.Authorize[]]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: std.Subject.Subject[]
			RETURN: #1
				TYPE: T
		CODE
			[1] owner, val = unpack(Move) $0
			[2] $1 = call#std.Subject.checkAuthorization[](owner@_, auth@_)
			[3] #0, #1 = switch(Move) $1:
				case False()
					[4] #0, #1 = rollback(val@_):(std.Subject.Subject[], T)
				case True()
					[5] owner#8 = fetch(Copy) owner
					[6] #0, #1  = return (owner#8@_, val@_)
	FUNCTION: safeUnlock
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: locked
				TYPE: std.SubjectLock.Locked[T]
				CONSUME: true
			PARAM: auth
				TYPE: std.Capability.Perm[std.Subject.Subject[],std.Subject.Authorize[]]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: std.Either.Either[T,std.SubjectLock.Locked[T]]
		CODE
			[1] #0 = try call#std.SubjectLock.unlock[T](essential locked@_, auth@_):
				success($2, val)
					[2] #0 = pack(Move)#std.Either.Either[T,std.SubjectLock.Locked[T]]|Left(val@_)
				fail(lock)
					[3] #0 = pack(Move)#std.Either.Either[T,std.SubjectLock.Locked[T]]|Right(lock@_)
	FUNCTION: lockEntry
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: owner
				TYPE: std.Subject.Subject[]
				CONSUME: false
			PARAM: id
				TYPE: sys.Ids.PrivateId[]
				CONSUME: false
			PARAM: val
				TYPE: T
				CONSUME: true
		RETURNS
			RETURN: $4
				TYPE: sys.Sys.Entry[std.SubjectLock.Locked[T]]
		CODE
			[1] owner#9 = fetch(Copy) owner
			[2] $1 = pack(Move)#std.SubjectLock.Locked[T]|Locked(owner#9@_, val@_)
			[3] id#10 = fetch(Copy) id
			[4] $4 = pack(Move)#sys.Sys.Entry[std.SubjectLock.Locked[T]]|Entry(id#10@_, $1@_)
	FUNCTION: unlockEntry
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: $0
				TYPE: sys.Sys.Entry[std.SubjectLock.Locked[T]]
				CONSUME: true
			PARAM: auth
				TYPE: std.Capability.Perm[std.Subject.Subject[],std.Subject.Authorize[]]
				CONSUME: false
		RETURNS
			RETURN: id
				TYPE: sys.Ids.PrivateId[]
			RETURN: owner
				TYPE: std.Subject.Subject[]
			RETURN: val
				TYPE: T
		CODE
			[1] id, locked = unpack(Move) $0
			[2] owner, val = let:
				[3] owner, val = call#std.SubjectLock.unlock[T](locked@_, auth@_)
			[4] id#11 = fetch(Copy) id
			[5] owner#12 = fetch(Copy) owner
			[6] id, owner, val  = return (id#11@_, owner#12@_, val@_)
IMPLEMENTS
