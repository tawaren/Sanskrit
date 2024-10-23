NAME: Subject$EqualForSubject
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: std.Subject.Subject[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: std.Subject.subjectEq[]
		IMPL_TARGET: std.Subject.EqualForSubject$eq[]
