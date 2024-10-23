NAME: AuthorizationFromEdDsa
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, transaction)
TRANSACTION
FUNCTION: AuthorizationFromEdDsa
	EXTERNAL: false
	TRANSACTIONAL: true
	ACCESS
		Call: Global
	GENERICS
	PARAMS
		PARAM: pk
			TYPE: sys.EdDsa.Pk[]
			CONSUME: false
		PARAM: sig
			TYPE: sys.EdDsa.Sig[]
			CONSUME: false
		PARAM: ctx
			TYPE: sys.Sys.Context[]
			CONSUME: false
	RETURNS
		RETURN: #3
			TYPE: sys.Ids.PrivateId[]
		RETURN: #4
			TYPE: std.Capability.Cap[std.Subject.Subject[],std.Subject.Authorize[]]
	CODE
		[1] #3, #4 = call#std.EdDsaAuthenticator.authenticateWith[](pk@_, sig@_, ctx@_)
