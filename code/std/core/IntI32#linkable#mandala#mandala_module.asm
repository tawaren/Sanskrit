NAME: IntI32
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
				TYPE: sys.IntI32.I32[]
				CONSUME: false
			PARAM: op2
				TYPE: sys.IntI32.I32[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
		CODE
			[1] $0 = call#sys.IntI32.hash[](op1@_)
			[2] $2 = call#sys.IntI32.hash[](op2@_)
			[3] #0 = call#sys.Data.joinHash[]($0@_, $2@_)
IMPLEMENTS
	IMPLEMENT: HashForI32$hash
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Hash.hash[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: hash
				TYPE: core.Hash.hash[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.hash[](op@_)
	IMPLEMENT: HashForI32$joinHash
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Hash.joinHash[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: joinHash
				TYPE: core.Hash.joinHash[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#core.IntI32.joinHash[](op1@_, op2@_)
	IMPLEMENT: EqForI32$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.eq[](op1@_, op2@_)
	IMPLEMENT: BitOpsForI32$and
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.and[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: and
				TYPE: core.BitOps.and[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.and[](op1@_, op2@_)
	IMPLEMENT: BitOpsForI32$or
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.or[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: or
				TYPE: core.BitOps.or[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.or[](op1@_, op2@_)
	IMPLEMENT: BitOpsForI32$xor
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.xor[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: xor
				TYPE: core.BitOps.xor[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.xor[](op1@_, op2@_)
	IMPLEMENT: BitOpsForI32$not
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.not[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: not
				TYPE: core.BitOps.not[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.not[](op@_)
	IMPLEMENT: ArithForI32$add
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: core.Arith.add[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: add
				TYPE: core.Arith.add[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.add[](op1@_, op2@_)
	IMPLEMENT: ArithForI32$sub
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: core.Arith.sub[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: sub
				TYPE: core.Arith.sub[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.sub[](op1@_, op2@_)
	IMPLEMENT: ArithForI32$mul
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: core.Arith.mul[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: mul
				TYPE: core.Arith.mul[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.mul[](op1@_, op2@_)
	IMPLEMENT: ArithForI32$div
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: core.Arith.div[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: div
				TYPE: core.Arith.div[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.div[](op1@_, op2@_)
	IMPLEMENT: CompareForI32$lt
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Compare.lt[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: lt
				TYPE: core.Compare.lt[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.lt[](op1@_, op2@_)
	IMPLEMENT: CompareForI32$lte
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Compare.lte[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: lte
				TYPE: core.Compare.lte[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.lte[](op1@_, op2@_)
	IMPLEMENT: CompareForI32$gt
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Compare.gt[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: gt
				TYPE: core.Compare.gt[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.gt[](op1@_, op2@_)
	IMPLEMENT: CompareForI32$gte
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Compare.gte[sys.IntI32.I32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: gte
				TYPE: core.Compare.gte[sys.IntI32.I32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntI32.gte[](op1@_, op2@_)
