NAME: Publish
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Shared
		EXTERNAL: false
		ACCESS
			Inspect: Global
			Consume: Local
			Create: Local
		CAPABILITIES:  Value Unbound Persist Copy
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Copy
		CONSTRUCTORS
			CONSTRUCTOR: Shared
				FIELDS
					FIELD: val
						TYPE: T
SIGNATURES
FUNCTIONS
	FUNCTION: share
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist Copy
		PARAMS
			PARAM: id
				TYPE: sys.Ids.PrivateId[]
				CONSUME: false
			PARAM: val
				TYPE: T
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: sys.Sys.Entry[std.Publish.Shared[T]]
		CODE
			[1] $1 = pack(Move)#std.Publish.Shared[T]|Shared(val@_)
			[2] id#22 = fetch(Copy) id
			[3] #0 = pack(Move)#sys.Sys.Entry[std.Publish.Shared[T]]|Entry(id#22@_, $1@_)
	FUNCTION: get
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist Copy
		PARAMS
			PARAM: $0
				TYPE: sys.Sys.Entry[std.Publish.Shared[T]]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: T
		CODE
			[1] $1, v = inspect $0
			[2] #0 = field#val(Copy) v
	FUNCTION: dispose
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Guarded(Set(T))
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound Persist Copy
		PARAMS
			PARAM: $0
				TYPE: sys.Sys.Entry[std.Publish.Shared[T]]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: T
		CODE
			[1] $1, v = unpack(Move) $0
			[2] #0 = field#val(Move) v
IMPLEMENTS
