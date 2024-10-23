NAME: Ids
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: PrivateId
		EXTERNAL: true
			SIZE: 20
		ACCESS
			Create: Local
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
	DATA TYPE: PrivateModuleId
		EXTERNAL: false
		ACCESS
			Create: Local
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
		CONSTRUCTORS
			CONSTRUCTOR: PrivateModuleId
				FIELDS
					FIELD: id
						TYPE: sys.Ids.PrivateId[]
SIGNATURES
FUNCTIONS
	FUNCTION: moduleId
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: #0
				TYPE: sys.Ids.PrivateModuleId[]
	FUNCTION: idFromData
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: dat
				TYPE: sys.Data.Data20[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(sys.Ids.PrivateId[])
	FUNCTION: idToData
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: id
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	FUNCTION: moduleIdFromData
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: dat
				TYPE: sys.Data.Data20[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(sys.Ids.PrivateModuleId[])
	FUNCTION: moduleIdToData
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: id
				TYPE: projected(sys.Ids.PrivateModuleId[])
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	FUNCTION: eqId
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: id1
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
			PARAM: id2
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
	FUNCTION: eqModuleId
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: id1
				TYPE: projected(sys.Ids.PrivateModuleId[])
				CONSUME: false
			PARAM: id2
				TYPE: projected(sys.Ids.PrivateModuleId[])
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
	FUNCTION: privateIdDerive
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: priv
				TYPE: sys.Ids.PrivateId[]
				CONSUME: false
			PARAM: hash
				TYPE: sys.Data.Data20[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Ids.PrivateId[]
	FUNCTION: idDerive
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: id
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
			PARAM: hash
				TYPE: sys.Data.Data20[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(sys.Ids.PrivateId[])
	FUNCTION: privateModuleIdDerive
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: priv
				TYPE: sys.Ids.PrivateModuleId[]
				CONSUME: false
			PARAM: hash
				TYPE: sys.Data.Data20[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Ids.PrivateId[]
	FUNCTION: moduleIdDerive
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: id
				TYPE: projected(sys.Ids.PrivateModuleId[])
				CONSUME: false
			PARAM: hash
				TYPE: sys.Data.Data20[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(sys.Ids.PrivateId[])
	FUNCTION: idFromPrivate
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: priv
				TYPE: sys.Ids.PrivateId[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(sys.Ids.PrivateId[])
		CODE
			[1] #0 = project priv
	FUNCTION: moduleIdFromPrivate
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: priv
				TYPE: sys.Ids.PrivateModuleId[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(sys.Ids.PrivateModuleId[])
		CODE
			[1] #0 = project priv
	FUNCTION: privateIdToData
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: priv
				TYPE: sys.Ids.PrivateId[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
		CODE
			[1] $0 = project priv
			[2] #0 = call#sys.Ids.idToData[]($0@_)
	FUNCTION: privateModuleIdToData
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: priv
				TYPE: sys.Ids.PrivateModuleId[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
		CODE
			[1] $1 = field#id(Copy) priv
			[2] $0 = project $1
			[3] #0 = call#sys.Ids.idToData[]($0@_)
	FUNCTION: eqPrivateId
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: id1
				TYPE: sys.Ids.PrivateId[]
				CONSUME: false
			PARAM: id2
				TYPE: sys.Ids.PrivateId[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] $0 = project id1
			[2] $2 = project id2
			[3] #0 = call#sys.Ids.eqId[]($0@_, $2@_)
	FUNCTION: eqPrivateModuleId
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: id1
				TYPE: sys.Ids.PrivateModuleId[]
				CONSUME: false
			PARAM: id2
				TYPE: sys.Ids.PrivateModuleId[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] $0 = project id1
			[2] $2 = project id2
			[3] #0 = call#sys.Ids.eqModuleId[]($0@_, $2@_)
IMPLEMENTS
