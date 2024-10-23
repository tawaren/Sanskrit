NAME: NativeAuthenticator
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
			PARAM: senderId
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: std.Subject.ValidatedSubject[]
		CODE
			[1] $0 = call#sys.Ids.moduleId[]()
			[2] #0 = call#std.Subject.validateSubject[]($0@_, senderId@_)
	FUNCTION: authenticateWith
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: $0
				TYPE: sys.Sys.Sender[]
				CONSUME: false
		RETURNS
			RETURN: auth
				TYPE: std.Capability.Cap[std.Subject.Subject[],std.Subject.Authorize[]]
		CODE
			[1] privId = inspect $0
			[2] $1 = call#sys.Ids.moduleId[]()
			[3] $2 = call#sys.Ids.idFromPrivate[](privId@_)
			[4] auth = call#std.Subject.authenticate[]($1@_, $2@_)
IMPLEMENTS
