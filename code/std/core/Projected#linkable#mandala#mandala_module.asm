NAME: Projected
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
SIGNATURES
FUNCTIONS
	FUNCTION: projectedEq
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: op1
				TYPE: projected(typeParam(0))
				CONSUME: false
			PARAM: op2
				TYPE: projected(typeParam(0))
				CONSUME: false
			PARAM: eqFun
				TYPE: core.Equal.eq[T]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: sys.Bool.Bool[]
		CODE
			[1] unOp1 = let:
				[2] unOp1 = call#sys.Unsafe._unProject[T](op1@_)
			[3] unOp2 = let:
				[4] unOp2 = call#sys.Unsafe._unProject[T](op2@_)
			[5] res = let:
				[6] res = sig call#eqFun(unOp1@_, unOp2@_)
			[7]  = let:
				[8]  = call#sys.Unsafe._consume[T](unOp1@_)
			[9]  = let:
				[10]  = call#sys.Unsafe._consume[T](unOp2@_)
			[11] #0 = fetch(Copy) res
	FUNCTION: projectedHash
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: op1
				TYPE: projected(typeParam(0))
				CONSUME: false
			PARAM: hashFun
				TYPE: core.Hash.hash[T]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
		CODE
			[1] unOp1 = let:
				[2] unOp1 = call#sys.Unsafe._unProject[T](op1@_)
			[3] res = let:
				[4] res = sig call#hashFun(unOp1@_)
			[5]  = let:
				[6]  = call#sys.Unsafe._consume[T](unOp1@_)
			[7] #0 = fetch(Copy) res
	FUNCTION: projectedJoinHash
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: op1
				TYPE: projected(typeParam(0))
				CONSUME: false
			PARAM: op2
				TYPE: projected(typeParam(0))
				CONSUME: false
			PARAM: hashFun
				TYPE: core.Hash.joinHash[T]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: sys.Data.Data20[]
		CODE
			[1] unOp1 = let:
				[2] unOp1 = call#sys.Unsafe._unProject[T](op1@_)
			[3] unOp2 = let:
				[4] unOp2 = call#sys.Unsafe._unProject[T](op2@_)
			[5] res = let:
				[6] res = sig call#hashFun(unOp1@_, unOp2@_)
			[7]  = let:
				[8]  = call#sys.Unsafe._consume[T](unOp1@_)
			[9]  = let:
				[10]  = call#sys.Unsafe._consume[T](unOp2@_)
			[11] #0 = fetch(Copy) res
IMPLEMENTS
	IMPLEMENT: EqForProjected$eq
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Equal.eq[projected(typeParam(0))]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: eqFun
				TYPE: core.Equal.eq[T]
				CONSUME: true
		RETURNS
			RETURN: eq
				TYPE: core.Equal.eq[projected(typeParam(0))]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#core.Projected.projectedEq[T](op1@_, op2@_, eqFun@_)
	IMPLEMENT: HashForProjected$hash
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Hash.hash[projected(typeParam(0))]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: hashFun
				TYPE: core.Hash.hash[T]
				CONSUME: true
		RETURNS
			RETURN: hash
				TYPE: core.Hash.hash[projected(typeParam(0))]
		BINDINGS
			PARAMS
				BINDING: op
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#core.Projected.projectedHash[T](op@_, hashFun@_)
	IMPLEMENT: HashForProjected$joinHash
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: core.Hash.joinHash[projected(typeParam(0))]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: T
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: hashFun
				TYPE: core.Hash.joinHash[T]
				CONSUME: true
		RETURNS
			RETURN: joinHash
				TYPE: core.Hash.joinHash[projected(typeParam(0))]
		BINDINGS
			PARAMS
				BINDING: op1
				BINDING: op2
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#core.Projected.projectedJoinHash[T](op1@_, op2@_, hashFun@_)
