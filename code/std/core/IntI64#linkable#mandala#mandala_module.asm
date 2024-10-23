NAME: IntI64
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
SIGNATURES
FUNCTIONS
	FUNCTION: joinHash
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
		PARAMS
			PARAM: op1
				TYPE: sys.IntI64.I64[]
				CONSUME: false
			PARAM: op2
				TYPE: sys.IntI64.I64[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
		CODE
			[1] $0 = call#sys.IntI64.hash[](op1@_)
			[2] $2 = call#sys.IntI64.hash[](op2@_)
			[3] #0 = call#sys.Data.joinHash[]($0@_, $2@_)
IMPLEMENTS
	IMPLEMENT: HashForI64$hash
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Hash.hash[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: hash
				TYPE: core.Hash.hash[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.hash[](op@_)
	IMPLEMENT: HashForI64$joinHash
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Hash.joinHash[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: joinHash
				TYPE: core.Hash.joinHash[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#core.IntI64.joinHash[](op1@_, op2@_)
	IMPLEMENT: EqForI64$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.eq[](op1@_, op2@_)
	IMPLEMENT: BitOpsForI64$and
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.and[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: and
				TYPE: core.BitOps.and[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.and[](op1@_, op2@_)
	IMPLEMENT: BitOpsForI64$or
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.or[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: or
				TYPE: core.BitOps.or[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.or[](op1@_, op2@_)
	IMPLEMENT: BitOpsForI64$xor
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.xor[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: xor
				TYPE: core.BitOps.xor[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.xor[](op1@_, op2@_)
	IMPLEMENT: BitOpsForI64$not
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.not[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: not
				TYPE: core.BitOps.not[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.not[](op@_)
	IMPLEMENT: ArithForI64$add
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: core.Arith.add[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: add
				TYPE: core.Arith.add[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.add[](op1@_, op2@_)
	IMPLEMENT: ArithForI64$sub
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: core.Arith.sub[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: sub
				TYPE: core.Arith.sub[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.sub[](op1@_, op2@_)
	IMPLEMENT: ArithForI64$mul
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: core.Arith.mul[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: mul
				TYPE: core.Arith.mul[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.mul[](op1@_, op2@_)
	IMPLEMENT: ArithForI64$div
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: core.Arith.div[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: div
				TYPE: core.Arith.div[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.div[](op1@_, op2@_)
	IMPLEMENT: CompareForI64$lt
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Compare.lt[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: lt
				TYPE: core.Compare.lt[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.lt[](op1@_, op2@_)
	IMPLEMENT: CompareForI64$lte
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Compare.lte[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: lte
				TYPE: core.Compare.lte[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.lte[](op1@_, op2@_)
	IMPLEMENT: CompareForI64$gt
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Compare.gt[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: gt
				TYPE: core.Compare.gt[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.gt[](op1@_, op2@_)
	IMPLEMENT: CompareForI64$gte
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Compare.gte[sys.IntI64.I64[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: gte
				TYPE: core.Compare.gte[sys.IntI64.I64[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI64.gte[](op1@_, op2@_)
