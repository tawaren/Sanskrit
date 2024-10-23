NAME: Data
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Data1
		EXTERNAL: true
			SIZE: 1
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
	DATA TYPE: Data2
		EXTERNAL: true
			SIZE: 2
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
	DATA TYPE: Data4
		EXTERNAL: true
			SIZE: 4
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
	DATA TYPE: Data8
		EXTERNAL: true
			SIZE: 8
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
	DATA TYPE: Data12
		EXTERNAL: true
			SIZE: 12
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
	DATA TYPE: Data16
		EXTERNAL: true
			SIZE: 16
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
	DATA TYPE: Data20
		EXTERNAL: true
			SIZE: 20
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
	DATA TYPE: Data24
		EXTERNAL: true
			SIZE: 24
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
	DATA TYPE: Data28
		EXTERNAL: true
			SIZE: 28
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
	DATA TYPE: Data32
		EXTERNAL: true
			SIZE: 32
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
SIGNATURES
FUNCTIONS
	FUNCTION: eq1
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data1[]
				CONSUME: false
			PARAM: data2
				TYPE: sys.Data.Data1[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: eq2
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data2[]
				CONSUME: false
			PARAM: data2
				TYPE: sys.Data.Data2[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: eq4
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data4[]
				CONSUME: false
			PARAM: data2
				TYPE: sys.Data.Data4[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: eq8
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data8[]
				CONSUME: false
			PARAM: data2
				TYPE: sys.Data.Data8[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: eq12
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data12[]
				CONSUME: false
			PARAM: data2
				TYPE: sys.Data.Data12[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: eq16
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data16[]
				CONSUME: false
			PARAM: data2
				TYPE: sys.Data.Data16[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: eq20
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data20[]
				CONSUME: false
			PARAM: data2
				TYPE: sys.Data.Data20[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: eq24
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data24[]
				CONSUME: false
			PARAM: data2
				TYPE: sys.Data.Data24[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: eq28
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data28[]
				CONSUME: false
			PARAM: data2
				TYPE: sys.Data.Data28[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: eq32
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data32[]
				CONSUME: false
			PARAM: data2
				TYPE: sys.Data.Data32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: joinHash
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data20[]
				CONSUME: false
			PARAM: data2
				TYPE: sys.Data.Data20[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Data.Data20[]
	FUNCTION: hash1
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data1[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	FUNCTION: hash2
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data2[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	FUNCTION: hash4
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data4[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	FUNCTION: hash8
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data8[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	FUNCTION: hash12
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data12[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	FUNCTION: hash16
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data16[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	FUNCTION: hash20
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data20[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	FUNCTION: hash24
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data24[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	FUNCTION: hash28
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data28[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
	FUNCTION: hash32
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: data1
				TYPE: sys.Data.Data32[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
IMPLEMENTS
