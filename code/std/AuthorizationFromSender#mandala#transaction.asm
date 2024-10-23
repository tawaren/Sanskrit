NAME: AuthorizationFromSender
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, transaction)
TRANSACTION
FUNCTION: AuthorizationFromSender
	EXTERNAL: false
	TRANSACTIONAL: true
	ACCESS
		Call: Global
	GENERICS
	PARAMS
		PARAM: sender
			TYPE: sys.Sys.Sender[]
			CONSUME: false
	RETURNS
		RETURN: #0
			TYPE: std.Capability.Perm[std.Subject.Subject[],std.Subject.Authorize[]]
	CODE
		[1] $0 = call#std.NativeAuthenticator.authenticateWith[](sender@_)
		[2] #0 = call#std.Capability.toPerm[std.Subject.Subject[],std.Subject.Authorize[]]($0@_)
