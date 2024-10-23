NAME: Data1
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
SIGNATURES
FUNCTIONS
	FUNCTION: joinHash
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: op1
				TYPE: sys.Data.Data1[]
				CONSUME: false
			PARAM: op2
				TYPE: sys.Data.Data1[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
		CODE
			[1] $0 = call#sys.Data.hash1[](op1@_)
			[2] $2 = call#sys.Data.hash1[](op2@_)
			[3] #0 = call#sys.Data.joinHash[]($0@_, $2@_)
IMPLEMENTS
	IMPLEMENT: HashForData1$hash
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Hash.hash[sys.Data.Data1[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: hash
				TYPE: core.Hash.hash[sys.Data.Data1[]]
		BINDINGS
			PARAMS
				BINDING: op
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.Data.hash1[](op@_)
	IMPLEMENT: HashForData1$joinHash
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Hash.joinHash[sys.Data.Data1[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: joinHash
				TYPE: core.Hash.joinHash[sys.Data.Data1[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#core.Data1.joinHash[](op1@_, op2@_)
	IMPLEMENT: EqForData1$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[sys.Data.Data1[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[sys.Data.Data1[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.Data.eq1[](op1@_, op2@_)