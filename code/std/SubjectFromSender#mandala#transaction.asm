NAME: SubjectFromSender
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, transaction)
TRANSACTION
FUNCTION: SubjectFromSender
	EXTERNAL: false
	TRANSACTIONAL: true
	ACCESS
		Call: Global
	GENERICS
	PARAMS
		PARAM: $5
			TYPE: sys.Sys.Sender[]
			CONSUME: false
	RETURNS
		RETURN: #3
			TYPE: std.Subject.Subject[]
	CODE
		[1] senderId = inspect $5
		[2] $7 = project senderId
		[3] $6 = call#std.NativeAuthenticator.subjectFor[]($7@_)
		[4] #3 = field#subject(Move) $6
