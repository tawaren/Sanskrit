NAME: AuthorizationIdFromEdDsa
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, transaction)
TRANSACTION
FUNCTION: AuthorizationIdFromEdDsa
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
		RETURN: #2
			TYPE: sys.Ids.PrivateId[]
	CODE
		[1] id, $7 = let:
			[2] id, $7 = call#std.EdDsaAuthenticator.authenticateWith[](pk@_, sig@_, ctx@_)
		[3] #2 = fetch(Copy) id
