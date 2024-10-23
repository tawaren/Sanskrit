NAME: Subject$EqualForValidatedSubject
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: Equal
	PARAMS
		PARAM: std.Subject.ValidatedSubject[]
IMPLEMENTS
	IMPLEMENT: eq
		GENERICS
		FUN_TARGET: std.Subject.validatedSubjectEq[]
		IMPL_TARGET: std.Subject.EqualForValidatedSubject$eq[]
