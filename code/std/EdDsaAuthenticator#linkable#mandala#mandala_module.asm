NAME: EdDsaAuthenticator
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
SIGNATURES
FUNCTIONS
	FUNCTION: subjectFor
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: pk
				TYPE: sys.EdDsa.Pk[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: std.Subject.ValidatedSubject[]
		CODE
			[1] $0 = call#sys.Ids.moduleId[]()
			[2] $1 = call#sys.EdDsa.derivePublicId[](pk@_)
			[3] #0 = call#std.Subject.validateSubject[]($0@_, $1@_)
	FUNCTION: authenticateWith
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
			RETURN: sk
				TYPE: sys.Ids.PrivateId[]
			RETURN: auth
				TYPE: std.Capability.Cap[std.Subject.Subject[],std.Subject.Authorize[]]
		CODE
			[1] $0 = call#sys.EdDsa.verifyTx[](ctx@_, pk@_, sig@_)
			[2] sk, auth = switch(Move) $0:
				case False()
					[3] sk, auth = rollback():(sys.Ids.PrivateId[], std.Capability.Cap[std.Subject.Subject[],std.Subject.Authorize[]])
				case True()
					[4] internalSubject = let:
						[5] internalSubject = call#sys.EdDsa.derivePublicId[](pk@_)
					[6] privateId = let:
						[7] $5 = call#sys.Ids.moduleId[]()
						[8] privateId = call#std.Subject.derivePrivateId[]($5@_, internalSubject@_)
					[9] auth = let:
						[10] $7 = call#sys.Ids.moduleId[]()
						[11] auth = call#std.Subject.authenticate[]($7@_, internalSubject@_)
					[12] privateId#20 = fetch(Copy) privateId
					[13] auth#21 = fetch(Copy) auth
					[14] sk, auth  = return (privateId#20@_, auth#21@_)
IMPLEMENTS
