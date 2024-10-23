NAME: TreeList
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: TreeList
		EXTERNAL: false
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
			GENERIC: N
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Empty
				FIELDS
			CONSTRUCTOR: Full
				FIELDS
					FIELD: tree
						TYPE: N
SIGNATURES
FUNCTIONS
	FUNCTION: nil
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: N
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: #0
				TYPE: std.TreeList.TreeList[N]
		CODE
			[1] #0 = pack(Move)#std.TreeList.TreeList[N]|Empty()
	FUNCTION: con
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: N
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: head
				TYPE: E
				CONSUME: true
			PARAM: tail
				TYPE: std.TreeList.TreeList[N]
				CONSUME: true
			PARAM: single
				TYPE: std.NodeUtils.single[E,N]
				CONSUME: true
			PARAM: insertion
				TYPE: std.NodeUtils.insertion[E,N]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.TreeList.TreeList[N]
		CODE
			[1] #0 = switch(Move) tail:
				case Empty()
					[2] $1 = sig call#single(head@_)
					[3] #0 = pack(Move)#std.TreeList.TreeList[N]|Full($1@2)
					discard insertion@_
					[4] #0  = return (#0)
				case Full(tree)
					[5] $3 = sig call#insertion(head@_, tree@_)
					[6] #0 = switch(Move) $3:
						case Joint(nTree)
							[7] #0 = pack(Move)#std.TreeList.TreeList[N]|Full(nTree@_)
						case Overflow($7, $8)
							[8] #0 = rollback($7@_, $8@_):(std.TreeList.TreeList[N])
					discard single@_
					[9] #0  = return (#0)
	FUNCTION: headTail
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: N
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: list
				TYPE: std.TreeList.TreeList[N]
				CONSUME: true
			PARAM: inspection
				TYPE: std.NodeUtils.inspection[E,N]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: E
			RETURN: #1
				TYPE: std.TreeList.TreeList[N]
		CODE
			[1] #0, #1 = switch(Move) list:
				case Empty()
					[2] #0, #1 = rollback():(E, std.TreeList.TreeList[N])
					discard inspection@_
					[3] #0, #1  = return (#0, #1)
				case Full(tree)
					[4] $1 = sig call#inspection(tree@_)
					[5] #0, #1 = switch(Move) $1:
						case Last(head)
							[6] $4 = pack(Move)#std.TreeList.TreeList[N]|Empty()
							[7] #0, #1  = return (head@_, $4@_)
						case Full(head, tail)
							[8] $6 = pack(Move)#std.TreeList.TreeList[N]|Full(tail@_)
							[9] #0, #1  = return (head@_, $6@_)
	FUNCTION: head
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
			GENERIC: N
				PHANTOM: false
				CAPABILITIES:  Value Unbound Copy Drop
		PARAMS
			PARAM: list
				TYPE: std.TreeList.TreeList[N]
				CONSUME: false
			PARAM: inspection
				TYPE: std.NodeUtils.inspection[E,N]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: E
		CODE
			[1] head, $0 = let:
				[2] list#29 = fetch(Copy) list
				[3] head, $0 = call#std.TreeList.headTail[E,N](list#29@_, inspection@_)
			[4] #0 = fetch(Move) head
	FUNCTION: tail
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound Drop
			GENERIC: N
				PHANTOM: false
				CAPABILITIES:  Value Unbound Copy
		PARAMS
			PARAM: list
				TYPE: std.TreeList.TreeList[N]
				CONSUME: false
			PARAM: inspection
				TYPE: std.NodeUtils.inspection[E,N]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.TreeList.TreeList[N]
		CODE
			[1] $0, tail = let:
				[2] list#30 = fetch(Copy) list
				[3] $0, tail = call#std.TreeList.headTail[E,N](list#30@_, inspection@_)
			[4] #0 = fetch(Move) tail
	FUNCTION: test
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: e1
				TYPE: E
				CONSUME: true
			PARAM: e2
				TYPE: E
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.TreeList.TreeList[std.Node.Node[E]]
		CODE
			[1] $3 = call#std.TreeList.nil[std.Node.Node[E]]()
			[2] $#31 = call#std.Node.InspectResultForNode$single[E]()
			[3] $#32 = call#std.Node.InspectResultForNode$insertion[E]()
			[4] $1 = call#std.TreeList.con[E,std.Node.Node[E]](e2@_, $3@_, $#31@2, $#32@3)
			[2] $#33 = call#std.Node.InspectResultForNode$single[E]()
			[3] $#34 = call#std.Node.InspectResultForNode$insertion[E]()
			[5] #0 = call#std.TreeList.con[E,std.Node.Node[E]](e1@_, $1@_, $#33@2, $#34@3)
	FUNCTION: test2
		EXTERNAL: false
		TRANSACTIONAL: true
		ACCESS
			Call: Global
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: e1
				TYPE: E
				CONSUME: true
			PARAM: e2
				TYPE: E
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.TreeList.TreeList[std.Node.Node[std.Node.Node[std.Node.Node[std.Node.Node[E]]]]]
		CODE
			[1] $3 = call#std.TreeList.nil[std.Node.Node[std.Node.Node[std.Node.Node[std.Node.Node[E]]]]]()
			[2] $#35 = call#std.Node.InspectResultForNode$single[E]()
			[2] $#36 = call#std.Node.InspectResultForNodeRecursive$single[E,std.Node.Node[E]]($#35@2)
			[2] $#37 = call#std.Node.InspectResultForNodeRecursive$single[E,std.Node.Node[std.Node.Node[E]]]($#36@2)
			[3] $#38 = call#std.Node.InspectResultForNodeRecursive$single[E,std.Node.Node[std.Node.Node[std.Node.Node[E]]]]($#37@2)
			[4] $#39 = call#std.Node.InspectResultForNode$insertion[E]()
			[4] $#40 = call#std.Node.InspectResultForNodeRecursive$insertion[E,std.Node.Node[E]]($#39@4)
			[4] $#41 = call#std.Node.InspectResultForNodeRecursive$insertion[E,std.Node.Node[std.Node.Node[E]]]($#40@4)
			[5] $#42 = call#std.Node.InspectResultForNodeRecursive$insertion[E,std.Node.Node[std.Node.Node[std.Node.Node[E]]]]($#41@4)
			[6] $1 = call#std.TreeList.con[E,std.Node.Node[std.Node.Node[std.Node.Node[std.Node.Node[E]]]]](e2@_, $3@_, $#38@3, $#42@5)
			[2] $#43 = call#std.Node.InspectResultForNode$single[E]()
			[2] $#44 = call#std.Node.InspectResultForNodeRecursive$single[E,std.Node.Node[E]]($#43@2)
			[2] $#45 = call#std.Node.InspectResultForNodeRecursive$single[E,std.Node.Node[std.Node.Node[E]]]($#44@2)
			[3] $#46 = call#std.Node.InspectResultForNodeRecursive$single[E,std.Node.Node[std.Node.Node[std.Node.Node[E]]]]($#45@2)
			[4] $#47 = call#std.Node.InspectResultForNode$insertion[E]()
			[4] $#48 = call#std.Node.InspectResultForNodeRecursive$insertion[E,std.Node.Node[E]]($#47@4)
			[4] $#49 = call#std.Node.InspectResultForNodeRecursive$insertion[E,std.Node.Node[std.Node.Node[E]]]($#48@4)
			[5] $#50 = call#std.Node.InspectResultForNodeRecursive$insertion[E,std.Node.Node[std.Node.Node[std.Node.Node[E]]]]($#49@4)
			[7] #0 = call#std.TreeList.con[E,std.Node.Node[std.Node.Node[std.Node.Node[std.Node.Node[E]]]]](e1@_, $1@_, $#46@3, $#50@5)
IMPLEMENTS
