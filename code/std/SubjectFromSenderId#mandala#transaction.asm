NAME: SubjectFromSenderId
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, transaction)
TRANSACTION
FUNCTION: SubjectFromSenderId
	EXTERNAL: false
	TRANSACTIONAL: true
	ACCESS
		Call: Global
	GENERICS
	PARAMS
		PARAM: senderId
			TYPE: projected(sys.Ids.PrivateId[])
			CONSUME: false
	RETURNS
		RETURN: #2
			TYPE: std.Subject.Subject[]
	CODE
		[1] $3 = call#std.NativeAuthenticator.subjectFor[](senderId@_)
		[2] #2 = field#subject(Move) $3
