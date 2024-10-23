NAME: IntU128
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: U128
		EXTERNAL: true
			SIZE: 16
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
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU128.U128[]
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
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU128.U128[]
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
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU128.U128[]
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
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU128.U128[]
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
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU128.U128[]
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
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU128.U128[]
	FUNCTION: sub
		EXTERNAL: true
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU128.U128[]
	FUNCTION: div
		EXTERNAL: true
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU128.U128[]
	FUNCTION: mul
		EXTERNAL: true
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU128.U128[]
	FUNCTION: and
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU128.U128[]
	FUNCTION: or
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU128.U128[]
	FUNCTION: xor
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU128.U128[]
				CONSUME: false
			PARAM: num2
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU128.U128[]
	FUNCTION: not
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num1
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU128.U128[]
	FUNCTION: toData
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Data.Data16[]
	FUNCTION: fromData
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: dat
				TYPE: sys.Data.Data16[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.IntU128.U128[]
	FUNCTION: hash
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: num
				TYPE: sys.IntU128.U128[]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Data.Data20[]
IMPLEMENTS
