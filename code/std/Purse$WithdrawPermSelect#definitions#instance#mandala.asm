NAME: Purse$WithdrawPermSelect
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, instance, definitions)
IMPLEMENTS
	CLASS: PermAssociation
	PARAMS
		PARAM: std.Purse.Withdraw[]
		PARAM: std.Purse.Master[]
IMPLEMENTS
	IMPLEMENT: perm
		GENERICS
			GENERIC: PF
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		FUN_TARGET: std.Purse.createWithdrawPerm[PF]
		IMPL_TARGET: std.Purse.WithdrawPermSelect$perm[PF]
