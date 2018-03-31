use compiler::common::view::*;
use compiler::common::types::*;

pub mod main {
    //TypeView
    use super::*;

    pub const TYPE_MAGIC_NUMBER: u8 = 1; //change

    //Layout
    // Identity -- Header
    pub const MAGIC_NUMBER:FieldLayout = FieldLayout::first(MAGIC_NUMBER_SIZE);
    pub const VERSION:FieldLayout =  FieldLayout::after(MAGIC_NUMBER,VERSION_SIZE);
    pub const MODULE_HASH:FieldLayout =  FieldLayout::after(VERSION,HASH_SIZE);
    pub const TYPE_INDEX:FieldLayout =  FieldLayout::after(MODULE_HASH,MEMBER_INDEX_SIZE);

    pub const IDENTITY_HEADER_END:usize = TYPE_INDEX.start + TYPE_INDEX.len;
    //Attribute -- Header
    pub const HEADER_SIZE:FieldLayout =  FieldLayout::after(TYPE_INDEX,LENGTH_SIZE);
    pub const MAX_SUPPORTED_PRIVILEGES:FieldLayout =  FieldLayout::after(HEADER_SIZE, PRIVILEGES_SIZE);
    pub const VISIBILITY:FieldLayout =  FieldLayout::after(MAX_SUPPORTED_PRIVILEGES, VISIBILITY_SIZE);
    pub const KIND:FieldLayout =  FieldLayout::after(VISIBILITY,TYPE_KIND_SIZE);

    pub const OPTIMISATION_DECLARATION:FieldLayout =  FieldLayout::after(KIND,OPTIMIZATION_DECLARATION_SIZE);
    //ONLY IN WITH INIT VERISON

    //Number of dynamic components
    pub const NUM_GENERICS:FieldLayout =  FieldLayout::after(OPTIMISATION_DECLARATION,AMOUNT_SIZE);
    pub const NUM_MODULE_IMPORTS:FieldLayout =  FieldLayout::after(NUM_GENERICS,AMOUNT_SIZE);
    pub const NUM_TYPE_IMPORTS:FieldLayout =  FieldLayout::after(NUM_MODULE_IMPORTS,AMOUNT_SIZE);
    pub const NUM_CONSTRUCTORS:FieldLayout =  FieldLayout::after(NUM_TYPE_IMPORTS,AMOUNT_SIZE);

    //ONLY IN WITH INIT VERISON
    pub const INIT_CODE_HASH:FieldLayout =  FieldLayout::after(NUM_CONSTRUCTORS, HASH_SIZE);

    // Dynamic Header
    pub const DYNAMIC_PART_START:usize =  NUM_CONSTRUCTORS.start + NUM_CONSTRUCTORS.len;
    pub const DYNAMIC_PART_START_WITH_INIT:usize =  INIT_CODE_HASH.start + INIT_CODE_HASH.len;

    // Generics             (NUM_GENERICS       *   BOUND_SIZE)
    // Size Coefficients    (NUM_GENERICS+1     *   COEFFICIENT_SIZE)
    // Module Imports       (NUM_MODULE_IMPORTS *   HASH_SIZE)
    // Type_Index_Table     (NUM_TYPE_IMPORTS   *   PTR_SIZE) -- Links to Types (because of dynamic size)
    // Ctr_Index_Table      (NUM_CONSTRUCTORS   *   PTR_SIZE) -- Links to Constructors (because of dynamic size)
    // Types                (NUM_TYPE_IMPORTS   *   ImportedTypeView [Dynamically Sized]) <typ::fields::imported>
    // Constructors         (NUM_CONSTRUCTORS   *   CtrView [Dynamically Sized]) <typ::fields::constructor>
}

pub mod constructor {
    //CtrView
    use super::*;

    pub const NUM_FIELDS:FieldLayout =  FieldLayout::first(AMOUNT_SIZE);
    pub const DYNAMIC_PART_START:usize =  NUM_FIELDS.start + NUM_FIELDS.len;
    //  Parameters    (NUM_FIELDS * Fields)

}

pub mod body {
    //BodyInitView
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
    // Constructors             (NUM_CONSTRUCTOR_IMPORTS    *   ImportedTypeBodyView [Dynamically Sized])   <typ::fields::imported_constructors>
    // Initializers             (NUM_INIT_IMPORTS           *   ImportedInitView [Dynamically Sized])       <typ::fields::imported_init>
    // CODE                     ByteArray/ByteCodes
}

pub mod imported_constructors {

    //ImportedTypeBodyView
    use super::*;

    pub const TYPE_ID:FieldLayout =  FieldLayout::first(TYPE_ID_SIZE);
    pub const NUM_CASES:FieldLayout =  FieldLayout::after(TYPE_ID, AMOUNT_SIZE);
    pub const DYNAMIC_PART_START:usize =  NUM_CASES.start + NUM_CASES.len;
    // Constructors (embedded)    (NUM_CASES * CtrView) <typ::fields::constructor>

}

pub mod imported_init {

    //ImportedInitView
    use super::*;

    pub const TYPE_ID:FieldLayout =  FieldLayout::first(TYPE_ID_SIZE);
    pub const INIT_CODE_HASH:FieldLayout =  FieldLayout::after(TYPE_ID, HASH_SIZE);
    pub const INIT_RETURN_TYPE:FieldLayout =  FieldLayout::after(INIT_CODE_HASH, TYPE_ID_SIZE);


}

pub mod imported {
    //ImportedTypeView |Parameter|
    use super::*;


    pub const MODULE_ID:FieldLayout = FieldLayout::first(MODULE_ID_SIZE);
    pub const VERSION:FieldLayout =  FieldLayout::after(MODULE_ID,VERSION_SIZE);
    pub const TYPE_INDEX:FieldLayout =  FieldLayout::after(VERSION,MEMBER_INDEX_SIZE);
    pub const OPTIMISATION_DECLARATION:FieldLayout =  FieldLayout::after(TYPE_INDEX,OPTIMIZATION_DECLARATION_SIZE);
    pub const PRIVILEGES_DECLARATION:FieldLayout =  FieldLayout::after(OPTIMISATION_DECLARATION, PRIVILEGES_SIZE);
    pub const KIND_DECLARATION:FieldLayout =  FieldLayout::after(PRIVILEGES_DECLARATION,FLAG_SIZE);
    pub const NUM_TYPE_PARAMS:FieldLayout =  FieldLayout::after(KIND_DECLARATION,TYPE_KIND_SIZE);
    // pub const NUM_COEFFICIENTS:FieldLayout =  FieldLayout::after(NUM_TYPE_PARAMS,AMOUNT_SIZE);


    pub const DYNAMIC_PART_START:usize =  NUM_TYPE_PARAMS.start + NUM_TYPE_PARAMS.len;
    // Type Params          (NUM_TYPE_PARAMS    *   TYPE_ID_SIZE)
    //OUT FOR NOW Size Coefficients    (NUM_COEFFICIENTS   *   COEFFICIENT_SIZE)

}