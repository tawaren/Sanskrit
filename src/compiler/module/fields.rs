use compiler::common::view::*;
use compiler::common::types::*;

pub mod main {
    //ModuleView
    use super::*;

    pub const MODULE_MAGIC_NUMBER:u8 = 0; //change

    //Layout
    // Identity -- Header
    pub const MAGIC_NUMBER:FieldLayout = FieldLayout::first(MAGIC_NUMBER_SIZE);
    pub const VERSION:FieldLayout =  FieldLayout::after(MAGIC_NUMBER,VERSION_SIZE);
    //Todo: Meta Layer Hash + Type

    //Number of dynamic components
    pub const NUM_TYPES:FieldLayout =  FieldLayout::after(VERSION,AMOUNT_SIZE);
    pub const NUM_FUNCTIONS:FieldLayout =  FieldLayout::after(NUM_TYPES,AMOUNT_SIZE);
    pub const NUM_CONSTANTS:FieldLayout =  FieldLayout::after(NUM_FUNCTIONS,AMOUNT_SIZE);

    // Dynamic Header
    pub const DYNAMIC_PART_START:usize =  NUM_CONSTANTS.start + NUM_CONSTANTS.len;
    // Types                 (NUM_TYPES     *   HASH_SIZE)
    // Functions             (NUM_FUNCTIONS *   HASH_SIZE)
    // Cells                 (NUM_CELLS     *   HASH_SIZE)
    // Constants             (NUM_CONSTANTS *   HASH_SIZE)


}