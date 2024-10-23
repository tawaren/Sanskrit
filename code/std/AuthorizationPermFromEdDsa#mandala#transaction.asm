NAME: AuthorizationPermFromEdDsa
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, transaction)
TRANSACTION
FUNCTION: AuthorizationPermFromEdDsa
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
		RETURN: #0
			TYPE: std.Capability.Perm[std.Subject.Subject[],std.Subject.Authorize[]]
	CODE
		[1] $0, auth = let:
			[2] $0, auth = call#std.EdDsaAuthenticator.authenticateWith[](pk@_, sig@_, ctx@_)
		[3] #0 = call#std.Capability.toPerm[std.Subject.Subject[],std.Subject.Authorize[]](auth@_)
