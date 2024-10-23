NAME: Subject
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Subject
		EXTERNAL: false
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Primitive Unbound Persist Copy Drop
		GENERICS
		CONSTRUCTORS
			CONSTRUCTOR: Subject
				FIELDS
					FIELD: id
						TYPE: projected(sys.Ids.PrivateId[])
	DATA TYPE: Authorize
		EXTERNAL: false
		ACCESS
			Create: Local
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
	DATA TYPE: ValidatedSubject
		EXTERNAL: false
		ACCESS
			Consume: Global
			Inspect: Global
			Create: Local
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
		CONSTRUCTORS
			CONSTRUCTOR: ValidatedSubject
				FIELDS
					FIELD: subject
						TYPE: std.Subject.Subject[]
SIGNATURES
FUNCTIONS
	FUNCTION: deriveSubjectId
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: issuer
				TYPE: projected(sys.Ids.PrivateModuleId[])
				CONSUME: false
			PARAM: subject
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
		RETURNS
			RETURN: $3
				TYPE: projected(sys.Ids.PrivateId[])
		CODE
			[1] $1 = call#sys.Ids.idToData[](subject@_)
			[2] $3 = call#sys.Ids.moduleIdDerive[](issuer@_, $1@_)
	FUNCTION: derivePrivateId
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: issuer
				TYPE: sys.Ids.PrivateModuleId[]
				CONSUME: false
			PARAM: subject
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
		RETURNS
			RETURN: $3
				TYPE: sys.Ids.PrivateId[]
		CODE
			[1] $1 = call#sys.Ids.idToData[](subject@_)
			[2] $3 = call#sys.Ids.privateModuleIdDerive[](issuer@_, $1@_)
	FUNCTION: createSubject
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Local
		GENERICS
		PARAMS
			PARAM: issuer
				TYPE: projected(sys.Ids.PrivateModuleId[])
				CONSUME: false
			PARAM: id
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
		RETURNS
			RETURN: $3
				TYPE: std.Subject.Subject[]
		CODE
			[1] $0 = call#std.Subject.deriveSubjectId[](issuer@_, id@_)
			[2] $3 = pack(Move)#std.Subject.Subject[]|Subject($0@_)
	FUNCTION: validateSubject
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: issuer
				TYPE: sys.Ids.PrivateModuleId[]
				CONSUME: false
			PARAM: subject
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: std.Subject.ValidatedSubject[]
		CODE
			[1] $1 = call#sys.Ids.moduleIdFromPrivate[](issuer@_)
			[2] $0 = call#std.Subject.createSubject[]($1@_, subject@_)
			[3] #0 = pack(Move)#std.Subject.ValidatedSubject[]|ValidatedSubject($0@_)
	FUNCTION: authenticate
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: issuer
				TYPE: sys.Ids.PrivateModuleId[]
				CONSUME: false
			PARAM: subject
				TYPE: projected(sys.Ids.PrivateId[])
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: std.Capability.Cap[std.Subject.Subject[],std.Subject.Authorize[]]
		CODE
			[1] $1 = call#sys.Ids.moduleIdFromPrivate[](issuer@_)
			[2] $0 = call#std.Subject.deriveSubjectId[]($1@_, subject@_)
			[3] #0 = call#std.Capability.createCap[std.Subject.Subject[],std.Subject.Authorize[]]($0@_)
	FUNCTION: checkAuthorization
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: subj
				TYPE: std.Subject.Subject[]
				CONSUME: false
			PARAM: auth
				TYPE: std.Capability.Perm[std.Subject.Subject[],std.Subject.Authorize[]]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] $0 = field#id(Copy) subj
			[2] $1 = field#id(Copy) auth
			[3] #0 = call#sys.Ids.eqId[]($0@_, $1@_)
	FUNCTION: subjectEq
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: s1
				TYPE: std.Subject.Subject[]
				CONSUME: false
			PARAM: s2
				TYPE: std.Subject.Subject[]
				CONSUME: false
		RETURNS
			RETURN: $4
				TYPE: sys.Bool.Bool[]
		CODE
			[1] $0 = field#id(Copy) s1
			[2] $1 = field#id(Copy) s2
			[3] $4 = call#sys.Ids.eqId[]($0@_, $1@_)
	FUNCTION: validatedSubjectEq
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: s1
				TYPE: std.Subject.ValidatedSubject[]
				CONSUME: false
			PARAM: s2
				TYPE: std.Subject.ValidatedSubject[]
				CONSUME: false
		RETURNS
			RETURN: $4
				TYPE: sys.Bool.Bool[]
		CODE
			[1] $0 = field#subject(Copy) s1
			[2] $1 = field#subject(Copy) s2
			[3] $4 = call#std.Subject.subjectEq[]($0@_, $1@_)
IMPLEMENTS
	IMPLEMENT: EqualForSubject$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[std.Subject.Subject[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[std.Subject.Subject[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Subject.subjectEq[](op1@_, op2@_)
	IMPLEMENT: EqualForValidatedSubject$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[std.Subject.ValidatedSubject[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[std.Subject.ValidatedSubject[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Subject.validatedSubjectEq[](op1@_, op2@_)
