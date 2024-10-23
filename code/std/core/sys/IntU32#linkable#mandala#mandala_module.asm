NAME: IntU32
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: U32
		EXTERNAL: true
			SIZE: 4
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
SIGNATURES
FUNCTIONS
	FUNCTION: eq
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: lt
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: lte
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: gt
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: gte
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Bool.Bool[]
	FUNCTION: add
		EXTERNAL: true
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU32.U32[]
	FUNCTION: sub
		EXTERNAL: true
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU32.U32[]
	FUNCTION: div
		EXTERNAL: true
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU32.U32[]
	FUNCTION: mul
		EXTERNAL: true
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU32.U32[]
	FUNCTION: and
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU32.U32[]
	FUNCTION: or
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU32.U32[]
	FUNCTION: xor
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU32.U32[]
	FUNCTION: not
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU32.U32[]
	FUNCTION: toData
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Data.Data4[]
	FUNCTION: fromData
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: dat
				TYPE: sys.Data.Data4[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU32.U32[]
	FUNCTION: hash
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Data.Data20[]
IMPLEMENTS
