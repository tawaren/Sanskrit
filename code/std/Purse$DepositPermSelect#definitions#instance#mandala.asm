NAME: Purse$DepositPermSelect
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: PermAssociation
	PARAMS
		PARAM: std.Purse.Deposit[]
		PARAM: std.Purse.Master[]
IMPLEMENTS
	IMPLEMENT: perm
		GENERICS
			GENERIC: PF
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Purse.createDepositPerm[PF]
		IMPL_TARGET: std.Purse.DepositPermSelect$perm[PF]
