NAME: SubjectFromEdDsa
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, transaction)
TRANSACTION
FUNCTION: SubjectFromEdDsa
	EXTERNAL: false
	TRANSACTIONAL: true
	ACCESS
		Call: Global
	GENERICS
	PARAMS
		PARAM: pk
			TYPE: sys.EdDsa.Pk[]
			CONSUME: false
	RETURNS
		RETURN: #5
			TYPE: std.Subject.Subject[]
	CODE
		[1] $12 = call#std.EdDsaAuthenticator.subjectFor[](pk@_)
		[2] #5 = field#subject(Move) $12
