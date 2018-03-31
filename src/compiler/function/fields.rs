use compiler::common::view::*;
use compiler::common::types::*;

pub mod main {
    //HeaderFunctionView
    use super::*;

    pub const FUNCTION_MAGIC_NUMBER:u8 = 3; //change

    //Layout
    // Identity -- Header
    pub const MAGIC_NUMBER:FieldLayout = FieldLayout::first(MAGIC_NUMBER_SIZE);
    pub const VERSION:FieldLayout =  FieldLayout::after(MAGIC_NUMBER,VERSION_SIZE);
    pub const MODULE_HASH:FieldLayout =  FieldLayout::after(VERSION,HASH_SIZE);
    pub const FUN_INDEX:FieldLayout =  FieldLayout::after(MODULE_HASH,MEMBER_INDEX_SIZE);

    pub const IDENTITY_HEADER_END:usize = FUN_INDEX.start + FUN_INDEX.len;
    //Attribute -- Header
    pub const HEADER_SIZE:FieldLayout =  FieldLayout::after(FUN_INDEX,LENGTH_SIZE);
    pub const VISIBILITY:FieldLayout =  FieldLayout::after(HEADER_SIZE,VISIBILITY_SIZE);
    pub const EXECUTION_MODE:FieldLayout =  FieldLayout::after(VISIBILITY,EXECUTION_MODE_SIZE);
    pub const OPTIMISATION_DECLARATION:FieldLayout =  FieldLayout::after(EXECUTION_MODE,OPTIMIZATION_DECLARATION_SIZE);

    pub const RETURN_TYPE:FieldLayout =  FieldLayout::after(OPTIMISATION_DECLARATION, TYPE_ID_SIZE);
    pub const RETURN_CONTROL:FieldLayout =  FieldLayout::after(RETURN_TYPE, CONTROL_SIZE);
    pub const CODE_HASH:FieldLayout =  FieldLayout::after(RETURN_CONTROL, HASH_SIZE);

    //Number of dynamic components
    pub const NUM_GENERICS:FieldLayout =  FieldLayout::after(CODE_HASH,AMOUNT_SIZE);
    pub const NUM_PARAMS:FieldLayout =  FieldLayout::after(NUM_GENERICS,AMOUNT_SIZE); //Part of ParamListView
    pub const NUM_MODULE_IMPORTS:FieldLayout =  FieldLayout::after(NUM_PARAMS,AMOUNT_SIZE);
    pub const NUM_TYPE_IMPORTS:FieldLayout =  FieldLayout::after(NUM_MODULE_IMPORTS,AMOUNT_SIZE);

    // Dynamic Header
    pub const DYNAMIC_PART_START:usize =  NUM_TYPE_IMPORTS.start + NUM_TYPE_IMPORTS.len;
    // Generics             (NUM_GENERICS       *   POWER_SIZE)
    // Params               (NUM_PARAMS         *   FIELD_SIZE)
    // Module Imports       (NUM_MODULE_IMPORTS *   HASH_SIZE)
    // Type_Index_Table     (NUM_TYPE_IMPORTS   *   PTR_SIZE) -- Links to Types (because of dynamic size)
    // Types                (NUM_TYPE_IMPORTS   *   ImportedTypeView [Dynamically Sized]) <typ::fields::imported>
    // BODY <function::fields::body>
}

pub mod body {
    //BodyFunctionView
    use super::*;

    pub const BODY_SIZE:FieldLayout =  FieldLayout::first(LENGTH_SIZE);
    pub const NUM_MODULE_IMPORTS:FieldLayout =  FieldLayout::after(BODY_SIZE,AMOUNT_SIZE);
    pub const NUM_TYPE_IMPORTS:FieldLayout =  FieldLayout::after(NUM_MODULE_IMPORTS,AMOUNT_SIZE);
    pub const NUM_FUN_IMPORTS:FieldLayout =  FieldLayout::after(NUM_TYPE_IMPORTS,AMOUNT_SIZE);
    pub const NUM_CONSTRUCTOR_IMPORTS:FieldLayout =  FieldLayout::after(NUM_FUN_IMPORTS,AMOUNT_SIZE);
    pub const NUM_INIT_IMPORTS:FieldLayout =  FieldLayout::after(NUM_CONSTRUCTOR_IMPORTS,AMOUNT_SIZE);

    // Dynamic BODY
    pub const DYNAMIC_PART_START:usize =  NUM_INIT_IMPORTS.start + NUM_INIT_IMPORTS.len;
    // Module Imports           (NUM_MODULE_IMPORTS         *   HASH_SIZE)
    // Type_Index_Table         (NUM_TYPE_IMPORTS           *   PTR_SIZE) -- Links to Types (because of dynamic size)
    // Function_Index_Table     (NUM_CONSTRUCTOR_IMPORTS    *   PTR_SIZE) -- Links to Functions (because of dynamic size)
    // Constructor_Index_Table  (NUM_TYPE_IMPORTS           *   PTR_SIZE) -- Links to Constructors (because of dynamic size)
    // Types                    (NUM_TYPE_IMPORTS           *   ImportedTypeView [Dynamically Sized])       <typ::fields::imported>
    // Functions                (NUM_FUN_IMPORTS            *   ImportedFunctionView [Dynamically Sized])   <function::fields::imported>
    // Constructors             (NUM_CONSTRUCTOR_IMPORTS    *   ImportedTypeBodyView [Dynamically Sized])   <typ::fields::body>
    // CODE                     ByteArray/ByteCodes
}

pub mod imported {
    // ImportedFunctionView
    use super::*;

    pub const MODULE_ID:FieldLayout = FieldLayout::first(MODULE_ID_SIZE);
    pub const VERSION:FieldLayout =  FieldLayout::after(MODULE_ID,VERSION_SIZE);
    pub const FUN_INDEX:FieldLayout =  FieldLayout::after(VERSION,MEMBER_INDEX_SIZE);
    pub const OPTIMISATION_DECLARATION:FieldLayout =  FieldLayout::after(FUN_INDEX,OPTIMIZATION_DECLARATION_SIZE);
    pub const RETURN_TYPE:FieldLayout =  FieldLayout::after(OPTIMISATION_DECLARATION, TYPE_ID_SIZE);
    pub const RETURN_CONTROL:FieldLayout =  FieldLayout::after(RETURN_TYPE, CONTROL_SIZE);

    pub const CODE_HASH:FieldLayout =  FieldLayout::after(RETURN_CONTROL, HASH_SIZE);

    pub const NUM_GENERIC_PARAMS:FieldLayout =  FieldLayout::after(CODE_HASH,AMOUNT_SIZE);
    pub const NUM_PARAMS:FieldLayout =  FieldLayout::after(NUM_GENERIC_PARAMS,AMOUNT_SIZE);

    pub const DYNAMIC_PART_START:usize =  NUM_PARAMS.start + NUM_PARAMS.len;
    // Generics     (NUM_GENERIC_PARAMS * TypeId)
    // Parameters   (NUM_PARAMS * Field)


}