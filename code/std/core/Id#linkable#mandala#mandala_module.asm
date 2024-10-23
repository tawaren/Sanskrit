NAME: Id
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
SIGNATURES
FUNCTIONS
IMPLEMENTS
	IMPLEMENT: EqForPublicId$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[projected(sys.Ids.PrivateId[])]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[projected(sys.Ids.PrivateId[])]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.Ids.eqId[](op1@_, op2@_)
	IMPLEMENT: EqForPrivateId$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[sys.Ids.PrivateId[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[sys.Ids.PrivateId[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.Ids.eqPrivateId[](op1@_, op2@_)
	IMPLEMENT: EqForPublicModuleId$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[projected(sys.Ids.PrivateModuleId[])]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[projected(sys.Ids.PrivateModuleId[])]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.Ids.eqModuleId[](op1@_, op2@_)
	IMPLEMENT: EqForPrivateModuleId$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[sys.Ids.PrivateModuleId[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[sys.Ids.PrivateModuleId[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.Ids.eqPrivateModuleId[](op1@_, op2@_)
