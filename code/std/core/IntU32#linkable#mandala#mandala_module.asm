NAME: IntU32
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
				TYPE: sys.IntU32.U32[]
				CONSUME: false
			PARAM: op2
				TYPE: sys.IntU32.U32[]
				CONSUME: false
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
		CODE
			[1] $0 = call#sys.IntU32.hash[](op1@_)
			[2] $2 = call#sys.IntU32.hash[](op2@_)
			[3] #0 = call#sys.Data.joinHash[]($0@_, $2@_)
IMPLEMENTS
	IMPLEMENT: HashForU32$hash
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Hash.hash[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: hash
				TYPE: core.Hash.hash[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.hash[](op@_)
	IMPLEMENT: HashForU32$joinHash
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Hash.joinHash[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: joinHash
				TYPE: core.Hash.joinHash[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#core.IntU32.joinHash[](op1@_, op2@_)
	IMPLEMENT: EqForU32$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.eq[](op1@_, op2@_)
	IMPLEMENT: BitOpsForU32$and
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.and[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: and
				TYPE: core.BitOps.and[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.and[](op1@_, op2@_)
	IMPLEMENT: BitOpsForU32$or
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.or[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: or
				TYPE: core.BitOps.or[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.or[](op1@_, op2@_)
	IMPLEMENT: BitOpsForU32$xor
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.xor[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: xor
				TYPE: core.BitOps.xor[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.xor[](op1@_, op2@_)
	IMPLEMENT: BitOpsForU32$not
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.BitOps.not[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: not
				TYPE: core.BitOps.not[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.not[](op@_)
	IMPLEMENT: ArithForU32$add
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: core.Arith.add[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: add
				TYPE: core.Arith.add[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.add[](op1@_, op2@_)
	IMPLEMENT: ArithForU32$sub
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: core.Arith.sub[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: sub
				TYPE: core.Arith.sub[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.sub[](op1@_, op2@_)
	IMPLEMENT: ArithForU32$mul
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: core.Arith.mul[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: mul
				TYPE: core.Arith.mul[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.mul[](op1@_, op2@_)
	IMPLEMENT: ArithForU32$div
		TRANSACTIONAL: true
		EXTERNAL: false
		DEFINES: core.Arith.div[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: div
				TYPE: core.Arith.div[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.div[](op1@_, op2@_)
	IMPLEMENT: CompareForU32$lt
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Compare.lt[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: lt
				TYPE: core.Compare.lt[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.lt[](op1@_, op2@_)
	IMPLEMENT: CompareForU32$lte
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Compare.lte[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: lte
				TYPE: core.Compare.lte[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.lte[](op1@_, op2@_)
	IMPLEMENT: CompareForU32$gt
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Compare.gt[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: gt
				TYPE: core.Compare.gt[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.gt[](op1@_, op2@_)
	IMPLEMENT: CompareForU32$gte
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Compare.gte[sys.IntU32.U32[]]
		ACCESS
			Call: Global
		GENERICS
		PARAMS
		RETURNS
			RETURN: gte
				TYPE: core.Compare.gte[sys.IntU32.U32[]]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#sys.IntU32.gte[](op1@_, op2@_)
