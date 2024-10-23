NAME: AuthorizationCapFromEdDsa
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, transaction)
TRANSACTION
FUNCTION: AuthorizationCapFromEdDsa
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
		RETURN: #1
			TYPE: std.Capability.Cap[std.Subject.Subject[],std.Subject.Authorize[]]
	CODE
		[1] $4, auth = let:
			[2] $4, auth = call#std.EdDsaAuthenticator.authenticateWith[](pk@_, sig@_, ctx@_)
		[3] #1 = fetch(Copy) auth
