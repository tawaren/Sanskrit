NAME: AuthorizationCapFromSender
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, transaction)
TRANSACTION
FUNCTION: AuthorizationCapFromSender
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
		RETURN: #1
			TYPE: std.Capability.Cap[std.Subject.Subject[],std.Subject.Authorize[]]
	CODE
		[1] #1 = call#std.NativeAuthenticator.authenticateWith[](sender@_)
