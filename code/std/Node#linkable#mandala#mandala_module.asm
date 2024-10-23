NAME: Node
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: Node
		EXTERNAL: false
		ACCESS
			Create: Local
			Consume: Local
			Inspect: Local
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: One
				FIELDS
					FIELD: fst
						TYPE: E
			CONSTRUCTOR: Two
				FIELDS
					FIELD: fst
						TYPE: E
					FIELD: snd
						TYPE: E
			CONSTRUCTOR: Three
				FIELDS
					FIELD: fst
						TYPE: E
					FIELD: snd
						TYPE: E
					FIELD: trd
						TYPE: E
			CONSTRUCTOR: Four
				FIELDS
					FIELD: fst
						TYPE: E
					FIELD: snd
						TYPE: E
					FIELD: trd
						TYPE: E
					FIELD: frt
						TYPE: E
SIGNATURES
FUNCTIONS
	FUNCTION: single
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: head
				TYPE: E
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.Node.Node[E]
		CODE
			[1] #0 = pack(Move)#std.Node.Node[E]|One(head@_)
	FUNCTION: inspection
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: lst
				TYPE: std.Node.Node[E]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.NodeUtils.InspectResult[E,std.Node.Node[E]]
		CODE
			[1] #0 = switch(Move) lst:
				case One(fst)
					[2] #0 = pack(Move)#std.NodeUtils.InspectResult[E,std.Node.Node[E]]|Last(fst@_)
				case Two(fst, snd)
					[3] $3 = pack(Move)#std.Node.Node[E]|One(snd@_)
					[4] #0 = pack(Move)#std.NodeUtils.InspectResult[E,std.Node.Node[E]]|Full(fst@_, $3@_)
				case Three(fst, snd, trd)
					[5] $6 = pack(Move)#std.Node.Node[E]|Two(snd@_, trd@_)
					[6] #0 = pack(Move)#std.NodeUtils.InspectResult[E,std.Node.Node[E]]|Full(fst@_, $6@_)
				case Four(fst, snd, trd, frt)
					[7] $10 = pack(Move)#std.Node.Node[E]|Three(snd@_, trd@_, frt@_)
					[8] #0 = pack(Move)#std.NodeUtils.InspectResult[E,std.Node.Node[E]]|Full(fst@_, $10@_)
	FUNCTION: insertion
		EXTERNAL: false
		TRANSACTIONAL: false
		ACCESS
			Call: Global
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
			PARAM: head
				TYPE: E
				CONSUME: true
			PARAM: tail
				TYPE: std.Node.Node[E]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.NodeUtils.InjectResult[std.Node.Node[E]]
		CODE
			[1] #0 = switch(Move) tail:
				case One(fst)
					[2] $1 = pack(Move)#std.Node.Node[E]|Two(head@_, fst@_)
					[3] #0 = pack(Move)#std.NodeUtils.InjectResult[std.Node.Node[E]]|Joint($1@_)
				case Two(fst, snd)
					[4] $4 = pack(Move)#std.Node.Node[E]|Three(head@_, fst@_, snd@_)
					[5] #0 = pack(Move)#std.NodeUtils.InjectResult[std.Node.Node[E]]|Joint($4@_)
				case Three(fst, snd, trd)
					[6] $8 = pack(Move)#std.Node.Node[E]|Four(head@_, fst@_, snd@_, trd@_)
					[7] #0 = pack(Move)#std.NodeUtils.InjectResult[std.Node.Node[E]]|Joint($8@_)
				case Four(fst, snd, trd, frt)
					[8] $13 = pack(Move)#std.Node.Node[E]|One(head@_)
					[9] $15 = pack(Move)#std.Node.Node[E]|Four(fst@_, snd@_, trd@_, frt@_)
					[10] #0 = pack(Move)#std.NodeUtils.InjectResult[std.Node.Node[E]]|Overflow($13@_, $15@_)
	FUNCTION: singleRec
		EXTERNAL: false
		TRANSACTIONAL: false
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
			PARAM: innerSingle
				TYPE: std.NodeUtils.single[E,N]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.Node.Node[N]
		CODE
			[1] $0 = sig call#innerSingle(head@_)
			[2] #0 = pack(Move)#std.Node.Node[N]|One($0@1)
	FUNCTION: inspectionRec
		EXTERNAL: false
		TRANSACTIONAL: false
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
			PARAM: lst
				TYPE: std.Node.Node[N]
				CONSUME: true
			PARAM: innerInspect
				TYPE: std.NodeUtils.inspection[E,N]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.NodeUtils.InspectResult[E,std.Node.Node[N]]
		CODE
			[1] #0 = switch(Move) lst:
				case One(fst)
					[2] $1 = sig call#innerInspect(fst@_)
					[3] #0 = switch(Move) $1:
						case Last(head)
							[4] #0 = pack(Move)#std.NodeUtils.InspectResult[E,std.Node.Node[N]]|Last(head@_)
						case Full(head, tail)
							[5] $5 = pack(Move)#std.Node.Node[N]|One(tail@_)
							[6] #0 = pack(Move)#std.NodeUtils.InspectResult[E,std.Node.Node[N]]|Full(head@_, $5@_)
				case Two(fst, snd)
					[7] $7 = sig call#innerInspect(fst@_)
					[8] #0 = switch(Move) $7:
						case Last(head)
							[9] $10 = pack(Move)#std.Node.Node[N]|One(snd@_)
							[10] #0 = pack(Move)#std.NodeUtils.InspectResult[E,std.Node.Node[N]]|Full(head@_, $10@_)
						case Full(head, tail)
							[11] $13 = pack(Move)#std.Node.Node[N]|Two(tail@_, snd@_)
							[12] #0 = pack(Move)#std.NodeUtils.InspectResult[E,std.Node.Node[N]]|Full(head@_, $13@_)
				case Three(fst, snd, trd)
					[13] $16 = sig call#innerInspect(fst@_)
					[14] #0 = switch(Move) $16:
						case Last(head)
							[15] $19 = pack(Move)#std.Node.Node[N]|Two(snd@_, trd@_)
							[16] #0 = pack(Move)#std.NodeUtils.InspectResult[E,std.Node.Node[N]]|Full(head@_, $19@_)
						case Full(head, tail)
							[17] $23 = pack(Move)#std.Node.Node[N]|Three(tail@_, snd@_, trd@_)
							[18] #0 = pack(Move)#std.NodeUtils.InspectResult[E,std.Node.Node[N]]|Full(head@_, $23@_)
				case Four(fst, snd, trd, frt)
					[19] $27 = sig call#innerInspect(fst@_)
					[20] #0 = switch(Move) $27:
						case Last(head)
							[21] $30 = pack(Move)#std.Node.Node[N]|Three(snd@_, trd@_, frt@_)
							[22] #0 = pack(Move)#std.NodeUtils.InspectResult[E,std.Node.Node[N]]|Full(head@_, $30@_)
						case Full(head, tail)
							[23] $35 = pack(Move)#std.Node.Node[N]|Four(tail@_, snd@_, trd@_, frt@_)
							[24] #0 = pack(Move)#std.NodeUtils.InspectResult[E,std.Node.Node[N]]|Full(head@_, $35@_)
	FUNCTION: insertionRec
		EXTERNAL: false
		TRANSACTIONAL: false
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
				TYPE: std.Node.Node[N]
				CONSUME: true
			PARAM: innerInsert
				TYPE: std.NodeUtils.insertion[E,N]
				CONSUME: true
		RETURNS
			RETURN: #0
				TYPE: std.NodeUtils.InjectResult[std.Node.Node[N]]
		CODE
			[1] #0 = switch(Move) tail:
				case One(fst)
					[2] $1 = sig call#innerInsert(head@_, fst@_)
					[3] #0 = switch(Move) $1:
						case Joint(cur)
							[4] $4 = pack(Move)#std.Node.Node[N]|One(cur@_)
							[5] #0 = pack(Move)#std.NodeUtils.InjectResult[std.Node.Node[N]]|Joint($4@_)
						case Overflow(fresh, old)
							[6] $6 = pack(Move)#std.Node.Node[N]|Two(fresh@_, old@_)
							[7] #0 = pack(Move)#std.NodeUtils.InjectResult[std.Node.Node[N]]|Joint($6@_)
				case Two(fst, snd)
					[8] $9 = sig call#innerInsert(head@_, fst@_)
					[9] #0 = switch(Move) $9:
						case Joint(cur)
							[10] $12 = pack(Move)#std.Node.Node[N]|Two(cur@_, snd@_)
							[11] #0 = pack(Move)#std.NodeUtils.InjectResult[std.Node.Node[N]]|Joint($12@_)
						case Overflow(fresh, old)
							[12] $15 = pack(Move)#std.Node.Node[N]|Three(fresh@_, old@_, snd@_)
							[13] #0 = pack(Move)#std.NodeUtils.InjectResult[std.Node.Node[N]]|Joint($15@_)
				case Three(fst, snd, trd)
					[14] $19 = sig call#innerInsert(head@_, fst@_)
					[15] #0 = switch(Move) $19:
						case Joint(cur)
							[16] $22 = pack(Move)#std.Node.Node[N]|Three(cur@_, snd@_, trd@_)
							[17] #0 = pack(Move)#std.NodeUtils.InjectResult[std.Node.Node[N]]|Joint($22@_)
						case Overflow(fresh, old)
							[18] $26 = pack(Move)#std.Node.Node[N]|Four(fresh@_, old@_, snd@_, trd@_)
							[19] #0 = pack(Move)#std.NodeUtils.InjectResult[std.Node.Node[N]]|Joint($26@_)
				case Four(fst, snd, trd, frt)
					[20] $31 = sig call#innerInsert(head@_, fst@_)
					[21] #0 = switch(Move) $31:
						case Joint(cur)
							[22] $34 = pack(Move)#std.Node.Node[N]|Four(cur@_, snd@_, trd@_, frt@_)
							[23] #0 = pack(Move)#std.NodeUtils.InjectResult[std.Node.Node[N]]|Joint($34@_)
						case Overflow(fresh, old)
							[24] $39 = pack(Move)#std.Node.Node[N]|One(fresh@_)
							[25] $41 = pack(Move)#std.Node.Node[N]|Four(old@_, snd@_, trd@_, frt@_)
							[26] #0 = pack(Move)#std.NodeUtils.InjectResult[std.Node.Node[N]]|Overflow($39@_, $41@_)
IMPLEMENTS
	IMPLEMENT: InspectResultForNode$single
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.NodeUtils.single[E,std.Node.Node[E]]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: single
				TYPE: std.NodeUtils.single[E,std.Node.Node[E]]
		BINDINGS
			PARAMS
				BINDING: head
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Node.single[E](head@_)
	IMPLEMENT: InspectResultForNode$inspection
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.NodeUtils.inspection[E,std.Node.Node[E]]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: inspection
				TYPE: std.NodeUtils.inspection[E,std.Node.Node[E]]
		BINDINGS
			PARAMS
				BINDING: lst
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Node.inspection[E](lst@_)
	IMPLEMENT: InspectResultForNode$insertion
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.NodeUtils.insertion[E,std.Node.Node[E]]
		ACCESS
			Call: Global
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES:  Value Unbound
		PARAMS
		RETURNS
			RETURN: insertion
				TYPE: std.NodeUtils.insertion[E,std.Node.Node[E]]
		BINDINGS
			PARAMS
				BINDING: head
				BINDING: tail
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Node.insertion[E](head@_, tail@_)
	IMPLEMENT: InspectResultForNodeRecursive$single
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.NodeUtils.single[E,std.Node.Node[N]]
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
			PARAM: innerSingle
				TYPE: std.NodeUtils.single[E,N]
				CONSUME: true
		RETURNS
			RETURN: single
				TYPE: std.NodeUtils.single[E,std.Node.Node[N]]
		BINDINGS
			PARAMS
				BINDING: head
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Node.singleRec[E,N](head@_, innerSingle@_)
	IMPLEMENT: InspectResultForNodeRecursive$inspection
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.NodeUtils.inspection[E,std.Node.Node[N]]
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
			PARAM: innerInspect
				TYPE: std.NodeUtils.inspection[E,N]
				CONSUME: true
		RETURNS
			RETURN: inspection
				TYPE: std.NodeUtils.inspection[E,std.Node.Node[N]]
		BINDINGS
			PARAMS
				BINDING: lst
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Node.inspectionRec[E,N](lst@_, innerInspect@_)
	IMPLEMENT: InspectResultForNodeRecursive$insertion
		TRANSACTIONAL: false
		EXTERNAL: false
		DEFINES: std.NodeUtils.insertion[E,std.Node.Node[N]]
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
			PARAM: innerInsert
				TYPE: std.NodeUtils.insertion[E,N]
				CONSUME: true
		RETURNS
			RETURN: insertion
				TYPE: std.NodeUtils.insertion[E,std.Node.Node[N]]
		BINDINGS
			PARAMS
				BINDING: head
				BINDING: tail
			RETURNS
				BINDING: #0
		CODE
			[param] #0 = call#std.Node.insertionRec[E,N](head@_, tail@_, innerInsert@_)
