NAME: EdDsa
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Pk
		EXTERNAL: true
			SIZE: 32
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
	DATA TYPE: Sig
		EXTERNAL: true
			SIZE: 64
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
SIGNATURES
FUNCTIONS
	FUNCTION: derivePublicId
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: projected(sys.Ids.PrivateId[])
	FUNCTION: verify1
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: msg
				TYPE: sys.Data.Data1[]
				CONSUME: false
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
			PARAM: sig
				TYPE: sys.EdDsa.Sig[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: verify2
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: msg
				TYPE: sys.Data.Data2[]
				CONSUME: false
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
			PARAM: sig
				TYPE: sys.EdDsa.Sig[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: verify4
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: msg
				TYPE: sys.Data.Data4[]
				CONSUME: false
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
			PARAM: sig
				TYPE: sys.EdDsa.Sig[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: verify8
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: msg
				TYPE: sys.Data.Data8[]
				CONSUME: false
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
			PARAM: sig
				TYPE: sys.EdDsa.Sig[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: verify12
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: msg
				TYPE: sys.Data.Data12[]
				CONSUME: false
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
			PARAM: sig
				TYPE: sys.EdDsa.Sig[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: verify16
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: msg
				TYPE: sys.Data.Data16[]
				CONSUME: false
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
			PARAM: sig
				TYPE: sys.EdDsa.Sig[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: verify20
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: msg
				TYPE: sys.Data.Data20[]
				CONSUME: false
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
			PARAM: sig
				TYPE: sys.EdDsa.Sig[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: verify24
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: msg
				TYPE: sys.Data.Data24[]
				CONSUME: false
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
			PARAM: sig
				TYPE: sys.EdDsa.Sig[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: verify28
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: msg
				TYPE: sys.Data.Data28[]
				CONSUME: false
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
			PARAM: sig
				TYPE: sys.EdDsa.Sig[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: verify32
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: msg
				TYPE: sys.Data.Data32[]
				CONSUME: false
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
			PARAM: sig
				TYPE: sys.EdDsa.Sig[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: verifyTx
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: ctx
				TYPE: sys.Sys.Context[]
				CONSUME: false
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
			PARAM: sig
				TYPE: sys.EdDsa.Sig[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
		CODE
			[1] $0 = call#sys.Sys.bundleHash[](ctx@_)
			[2] res = call#sys.EdDsa.verify20[]($0@_, pk@_, sig@_)
IMPLEMENTS
