NAME: SecureMessage
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Signed
		EXTERNAL: false
		ACCESS
			Create: Local
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Signed
				FIELDS
					FIELD: source
						TYPE: projected(sys.Ids.PrivateId[])
					FIELD: msg
						TYPE: T
	DATA TYPE: Sealed
		EXTERNAL: false
		ACCESS
			Create: Local
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Sealed
				FIELDS
					FIELD: target
						TYPE: projected(sys.Ids.PrivateId[])
					FIELD: msg
						TYPE: T
	DATA TYPE: StaticSigned
		EXTERNAL: false
		ACCESS
			Create: Guarded(Set(S))
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
			GENERIC: S
				PHANTOM: true
				CAPABILITIES: 
			GENERIC: T
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: StaticSigned
				FIELDS
					FIELD: msg
						TYPE: T
	DATA TYPE: StaticSealed
		EXTERNAL: false
		ACCESS
			Create: Guarded(Set(S))
			Consume: Guarded(Set(S))
			Inspect: Guarded(Set(S))
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
			GENERIC: S
				PHANTOM: true
				CAPABILITIES: 
			GENERIC: T
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: StaticSealed
				FIELDS
					FIELD: msg
						TYPE: T
	DATA TYPE: Once
		EXTERNAL: false
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Unbound Persist
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Once
				FIELDS
					FIELD: msg
						TYPE: T
SIGNATURES
FUNCTIONS
	FUNCTION: sign
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: source
				TYPE: sys.Ids.PrivateId[]
				CONSUME: false
			PARAM: msg
				TYPE: T
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.SecureMessage.Signed[T]
		CODE
			[1] $0 = call#sys.Ids.idFromPrivate[](source@_)
			[2] #0 = pack(Move)#std.SecureMessage.Signed[T]|Signed($0@_, msg@_)
	FUNCTION: signer
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: msg
				TYPE: std.SecureMessage.Signed[T]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(sys.Ids.PrivateId[])
		CODE
			[1] #0 = inspect msg:
				case Signed(src, $1)
					[2] #0 = fetch(Copy) src
	FUNCTION: signedMessage
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: $0
				TYPE: std.SecureMessage.Signed[T]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: T
		CODE
			[1] $1, msg = unpack(Move) $0
			[2] #0 = fetch(Move) msg
	FUNCTION: seal
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: target
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
			PARAM: msg
				TYPE: T
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.SecureMessage.Sealed[T]
		CODE
			[1] target#17 = fetch(Copy) target
			[2] #0 = pack(Move)#std.SecureMessage.Sealed[T]|Sealed(target#17@_, msg@_)
	FUNCTION: unseal
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: receiver
				TYPE: sys.Ids.PrivateId[]
				CONSUME: false
			PARAM: msg
				TYPE: std.SecureMessage.Sealed[T]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: T
		CODE
			[1] #0 = switch(Move) msg:
				case Sealed(target, msg)
					[2] $3 = call#sys.Ids.idFromPrivate[](receiver@_)
					[3] $1 = call#sys.Ids.eqId[](target@_, $3@_)
					[4] #0 = switch(Move) $1:
						case False()
							[5] #0 = rollback(msg@_):(T)
						case True()
							[6] #0 = fetch(Move) msg
	FUNCTION: receiver
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: msg
				TYPE: std.SecureMessage.Sealed[T]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(sys.Ids.PrivateId[])
		CODE
			[1] #0 = inspect msg:
				case Sealed(rev, $1)
					[2] #0 = fetch(Copy) rev
	FUNCTION: deliverOnce
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: msg
				TYPE: T
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.SecureMessage.Once[T]
		CODE
			[1] #0 = pack(Move)#std.SecureMessage.Once[T]|Once(msg@_)
	FUNCTION: sealedEntry
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: target
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
			PARAM: entryId
				TYPE: sys.Ids.PrivateId[]
				CONSUME: false
			PARAM: msg
				TYPE: T
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: sys.Sys.Entry[std.SecureMessage.Sealed[T]]
		CODE
			[1] $1 = call#std.SecureMessage.seal[T](target@_, msg@_)
			[2] entryId#18 = fetch(Copy) entryId
			[3] #0 = pack(Move)#sys.Sys.Entry[std.SecureMessage.Sealed[T]]|Entry(entryId#18@_, $1@_)
	FUNCTION: unsealEntry
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist
		PARAMS
			PARAM: receiver
				TYPE: sys.Ids.PrivateId[]
				CONSUME: false
			PARAM: entry
				TYPE: sys.Sys.Entry[std.SecureMessage.Sealed[T]]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: sys.Ids.PrivateId[]
			RETURN: #1
				TYPE: T
		CODE
			[1] #0, #1 = switch(Move) entry:
				case Entry(entryId, sealed)
					[2] $2 = call#std.SecureMessage.unseal[T](receiver@_, sealed@_)
					[3] entryId#19 = fetch(Copy) entryId
					[4] #0, #1  = return (entryId#19@_, $2@_)
IMPLEMENTS
