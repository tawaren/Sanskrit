NAME: Asset
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Asset
		EXTERNAL: false
		ACCESS
			Inspect: Local
			Create: Local
			Consume: Local
		CAPABILITIES:  Value Unbound Persist
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Asset
				FIELDS
					FIELD: id
						TYPE: sys.Ids.PrivateId[]
					FIELD: assetData
						TYPE: T
SIGNATURES
FUNCTIONS
	FUNCTION: mint
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Guarded(Set(T))
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: assetData
				TYPE: T
				CONSUME: true
			PARAM: gen
				TYPE: sys.Sys.IdGenerator[]
				CONSUME: true
		RETURNS
			RETURN: res
				TYPE: std.Asset.Asset[T]
			RETURN: newGen
				TYPE: sys.Sys.IdGenerator[]
		CODE
			[1] id, newGen = let:
				[2] id, newGen = call#sys.Sys.uniqueID[](gen@_)
			[3] id#28 = fetch(Copy) id
			[4] $1 = pack(Move)#std.Asset.Asset[T]|Asset(id#28@_, assetData@_)
			[5] res, newGen  = return ($1@_, newGen@_)
	FUNCTION: burn
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Guarded(Set(T))
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: ass1
				TYPE: std.Asset.Asset[T]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: T
		CODE
			[1] $0 = let:
				[2] $0 = fetch(Move) ass1
			[3] id, assetData = unpack(Move) $0
			[4] #0 = fetch(Move) assetData
	FUNCTION: getId
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: ass1
				TYPE: std.Asset.Asset[T]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(sys.Ids.PrivateId[])
		CODE
			[1] $0 = field#id(Copy) ass1
			[2] #0 = project $0
	FUNCTION: getData
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound Copy
		PARAMS
			PARAM: ass1
				TYPE: std.Asset.Asset[T]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: T
		CODE
			[1] #0 = field#assetData(Copy) ass1
IMPLEMENTS
