NAME: Unsafe
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Unsafe
		EXTERNAL: true
			SIZE: 0
		ACCESS
			Create: Local
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Copy Drop
		GENERICS
			GENERIC: T
				PHANTOM: true
				CAPABILITIES: 
SIGNATURES
FUNCTIONS
	FUNCTION: _unProject
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Local
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: t
				TYPE: projected(typeParam(0))
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: T
	FUNCTION: _packUnsafe
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Local
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: t
				TYPE: T
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: sys.Unsafe.Unsafe[T]
	FUNCTION: _unpackUnsafe
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Local
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: t
				TYPE: sys.Unsafe.Unsafe[T]
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: T
	FUNCTION: _copy
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Local
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: t
				TYPE: T
				CONSUME: false
		RETURNS
			RETURN: res
				TYPE: T
	FUNCTION: _consume
		EXTERNAL: true
		TRANSACTIONAL: false
		ACCESS
			Call: Local
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: t
				TYPE: T
				CONSUME: true
		RETURNS
IMPLEMENTS
