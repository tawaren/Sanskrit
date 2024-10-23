NAME: NodeUtils
LANGUAGE: mandala
VERSION: 0
CLASSIFIER: Set(mandala, linkable, mandala_module)
DATA TYPES
	DATA TYPE: InspectResult
		EXTERNAL: false
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
			GENERIC: E
				PHANTOM: false
				CAPABILITIES: 
			GENERIC: L
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Last
				FIELDS
					FIELD: e
						TYPE: E
			CONSTRUCTOR: Full
				FIELDS
					FIELD: e
						TYPE: E
					FIELD: lst
						TYPE: L
	DATA TYPE: InjectResult
		EXTERNAL: false
		ACCESS
			Create: Global
			Consume: Global
			Inspect: Global
		CAPABILITIES:  Value Unbound Persist Copy Drop
		GENERICS
			GENERIC: L
				PHANTOM: false
				CAPABILITIES: 
		CONSTRUCTORS
			CONSTRUCTOR: Joint
				FIELDS
					FIELD: l
						TYPE: L
			CONSTRUCTOR: Overflow
				FIELDS
					FIELD: fresh
						TYPE: L
					FIELD: old
						TYPE: L
SIGNATURES
FUNCTIONS
IMPLEMENTS
