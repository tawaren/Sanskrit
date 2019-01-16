use core::result::Result as OResult;
use alloc::prelude::String;


//The error in case we need something to interpret (in tests)
#[cfg(not(feature = "string_errors"))]
pub type Result<T> = OResult<T,u8>;

//te error if we are not interested in the details (Spares us a lot of expensive (space) string constants)
#[cfg(feature = "string_errors")]
pub type Result<T> = OResult<T,String>;

//Fast way out if we need debug infos
#[cfg(feature = "panic_errors")]
pub fn pre_error(){panic!()}

#[cfg(not(feature = "panic_errors"))]
pub fn pre_error(){}

//All the errors

#[cfg(not(feature = "string_errors"))]
pub fn type_does_not_exist_error<T>() -> Result<T>{
    pre_error();
    Err(0)
}

#[cfg(feature = "string_errors")]
pub fn type_does_not_exist_error<T>() -> Result<T>{
    pre_error();
    Err("Type does not exist".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn module_does_not_exist_error<T>() -> Result<T>{
    pre_error();
    Err(1)
}

#[cfg(feature = "string_errors")]
pub fn module_does_not_exist_error<T>() -> Result<T>{
    pre_error();
    Err("Module does not exist".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn error_does_not_exist_error<T>() -> Result<T>{
    pre_error();
    Err(2)
}

#[cfg(feature = "string_errors")]
pub fn error_does_not_exist_error<T>() -> Result<T>{
    pre_error();
    Err("Error does not exist".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn generics_dont_have_ctrs_error<T>() -> Result<T>{
    pre_error();
    Err(3)
}

#[cfg(feature = "string_errors")]
pub fn generics_dont_have_ctrs_error<T>() -> Result<T>{
    pre_error();
    Err("Generics do not have constructors".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn num_applied_generics_error<T>() -> Result<T> {
    pre_error();
    Err(4)
}

#[cfg(feature = "string_errors")]
pub fn num_applied_generics_error<T>() -> Result<T> {
    pre_error();
    Err("Number of applied type parameters must match the number of declared generics".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn can_not_apply_phantom_to_physical_error<T>() -> Result<T> {
    pre_error();
    Err(5)
}

#[cfg(feature = "string_errors")]
pub fn can_not_apply_phantom_to_physical_error<T>() -> Result<T> {
    pre_error();
    Err("Physical generics can not be instantiated by phantom generics".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn type_apply_constraint_violation<T>() -> Result<T> {
    pre_error();
    Err(6)
}

#[cfg(feature = "string_errors")]
pub fn type_apply_constraint_violation<T>() -> Result<T> {
    pre_error();
    Err("An apply must have all capabilities required by the generic".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn consuming_moved_error<T>() -> Result<T> {
    pre_error();
    Err(7)
}

#[cfg(feature = "string_errors")]
pub fn consuming_moved_error<T>() -> Result<T> {
    pre_error();
    Err("Consuming moved slot is forbidden".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn locking_moved_error<T>() -> Result<T> {
    pre_error();
    Err(8)
}

#[cfg(feature = "string_errors")]
pub fn locking_moved_error<T>() -> Result<T> {
    pre_error();
    Err("Locking moved slot is forbidden".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn free_error<T>() -> Result<T> {
    pre_error();
    Err(9)
}

#[cfg(feature = "string_errors")]
pub fn free_error<T>() -> Result<T> {
    pre_error();
    Err("Only consumed and not locked elem slots can be freed".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn consumed_cannot_be_returned_error<T>() -> Result<T> {
    pre_error();
    Err(10)
}

#[cfg(feature = "string_errors")]
pub fn consumed_cannot_be_returned_error<T>() -> Result<T> {
    pre_error();
    Err("A consumed slots can not be returned".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn out_of_range_stack_addressing<T>() -> Result<T> {
    pre_error();
    Err(11)
}

#[cfg(feature = "string_errors")]
pub fn out_of_range_stack_addressing<T>() -> Result<T> {
    pre_error();
    Err("Targeted element lies outside of stack".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn cannot_access_consumed<T>() -> Result<T> {
    pre_error();
    Err(12)
}

#[cfg(feature = "string_errors")]
pub fn cannot_access_consumed<T>() -> Result<T> {
    pre_error();
    Err("Can not access already moved slot".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn parsing_terminated<T>() -> Result<T> {
    pre_error();
    Err(13)
}

#[cfg(feature = "string_errors")]
pub fn parsing_terminated<T>() -> Result<T> {
    pre_error();
    Err("Input size does not match parsed size".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn parsing_case_failure<T>() -> Result<T> {
    pre_error();
    Err(14)
}

#[cfg(feature = "string_errors")]
pub fn parsing_case_failure<T>() -> Result<T> {
    pre_error();
    Err("Tag of parsed adt out of range".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn item_not_found<T>() -> Result<T> {
    pre_error();
    Err(15)
}

#[cfg(feature = "string_errors")]
pub fn item_not_found<T>() -> Result<T> {
    pre_error();
    Err("Requested item not present".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn item_already_exists<T>() -> Result<T> {
    pre_error();
    Err(16)
}

#[cfg(feature = "string_errors")]
pub fn item_already_exists<T>() -> Result<T> {
    pre_error();
    Err("Element already in store".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn elem_out_of_frame<T>() -> Result<T> {
    pre_error();
    Err(17)
}

#[cfg(feature = "string_errors")]
pub fn elem_out_of_frame<T>() -> Result<T> {
    pre_error();
    Err("Can not handle element from outside of the active frame".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn cycle_error<T>() -> Result<T> {
    pre_error();
    Err(18)
}

#[cfg(feature = "string_errors")]
pub fn cycle_error<T>() -> Result<T> {
    pre_error();
    Err("Cycle Detected".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn size_limit_exceeded_error<T>() -> Result<T> {
    pre_error();
    Err(19)
}

#[cfg(feature = "string_errors")]
pub fn size_limit_exceeded_error<T>() -> Result<T> {
    pre_error();
    Err("Element limit reached".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn no_ctr_available<T>() -> Result<T> {
    pre_error();
    Err(20)
}

#[cfg(feature = "string_errors")]
pub fn no_ctr_available<T>() -> Result<T> {
    pre_error();
    Err("Type has no constructors".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn native_type_not_exist_error<T>() -> Result<T> {
    pre_error();
    Err(21)
}

#[cfg(feature = "string_errors")]
pub fn native_type_not_exist_error<T>() -> Result<T> {
    pre_error();
    Err("Requested native type does not exist".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn literal_data_error<T>() -> Result<T> {
    pre_error();
    Err(22)
}

#[cfg(feature = "string_errors")]
pub fn literal_data_error<T>() -> Result<T> {
    pre_error();
    Err("Provided literal data is outside of the valid range".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn not_a_literal_error<T>() -> Result<T> {
    pre_error();
    Err(23)
}

#[cfg(feature = "string_errors")]
pub fn not_a_literal_error<T>() -> Result<T> {
    pre_error();
    Err("Provided type is not a literal".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn generic_args_mismatch<T>() -> Result<T> {
    pre_error();
    Err(24)
}

#[cfg(feature = "string_errors")]
pub fn generic_args_mismatch<T>() -> Result<T> {
    pre_error();
    Err("Wrong number or type of generic arguments".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn capability_missing_error<T>() -> Result<T> {
    pre_error();
    Err(25)
}

#[cfg(feature = "string_errors")]
pub fn capability_missing_error<T>() -> Result<T> {
    pre_error();
    Err("Required capability is missing".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn requested_ctr_missing<T>() -> Result<T> {
    pre_error();
    Err(26)
}

#[cfg(feature = "string_errors")]
pub fn requested_ctr_missing<T>() -> Result<T> {
    pre_error();
    Err("Requested Constructor unavailable".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn num_fields_mismatch<T>() -> Result<T> {
    pre_error();
    Err(27)
}

#[cfg(feature = "string_errors")]
pub fn num_fields_mismatch<T>() -> Result<T> {
    pre_error();
    Err("Number of supplied fields mismatch".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn empty_borrow_error<T>() -> Result<T> {
    pre_error();
    Err(28)
}

#[cfg(feature = "string_errors")]
pub fn empty_borrow_error<T>() -> Result<T> {
    pre_error();
    Err("Can only borrow if their is an argument".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn borrow_input_error<T>() -> Result<T> {
    pre_error();
    Err(29)
}

#[cfg(feature = "string_errors")]
pub fn borrow_input_error<T>() -> Result<T> {
    pre_error();
    Err("borrow declaration mismatch".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn type_mismatch<T>() -> Result<T> {
    pre_error();
    Err(30)
}

#[cfg(feature = "string_errors")]
pub fn type_mismatch<T>() -> Result<T> {
    pre_error();
    Err("Type mismatch".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn constructor_mismatch<T>() -> Result<T> {
    pre_error();
    Err(31)
}

#[cfg(feature = "string_errors")]
pub fn constructor_mismatch<T>() -> Result<T> {
    pre_error();
    Err("Wrong constructor specified".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn visibility_violation<T>() -> Result<T> {
    pre_error();
    Err(32)
}

#[cfg(feature = "string_errors")]
pub fn visibility_violation<T>() -> Result<T> {
    pre_error();
    Err("Function is not visible".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn num_param_mismatch<T>() -> Result<T> {
    pre_error();
    Err(33)
}

#[cfg(feature = "string_errors")]
pub fn num_param_mismatch<T>() -> Result<T> {
    pre_error();
    Err("Number of params is wrong".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn num_return_mismatch<T>() -> Result<T> {
    pre_error();
    Err(34)
}

#[cfg(feature = "string_errors")]
pub fn num_return_mismatch<T>() -> Result<T> {
    pre_error();
    Err("Number of returns is wrong".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn interpreter_error<T>() -> Result<T> {
    pre_error();
    Err(35)
}

#[cfg(feature = "string_errors")]
pub fn interpreter_error<T>() -> Result<T> {
    pre_error();
    Err("Error was produced".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn type_index_error<T>() -> Result<T> {
    pre_error();
    Err(36)
}

#[cfg(feature = "string_errors")]
pub fn type_index_error<T>() -> Result<T> {
    pre_error();
    Err("Type index is out of bounds".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn borrow_missing<T>() -> Result<T> {
    pre_error();
    Err(37)
}

#[cfg(feature = "string_errors")]
pub fn borrow_missing<T>() -> Result<T> {
    pre_error();
    Err("Borrowed value required".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn risk_missing<T>() -> Result<T> {
    pre_error();
    Err(38)
}

#[cfg(feature = "string_errors")]
pub fn risk_missing<T>() -> Result<T> {
    pre_error();
    Err("Risk is not declared".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn cannot_be_borrowed<T>() -> Result<T> {
    pre_error();
    Err(39)
}

#[cfg(feature = "string_errors")]
pub fn cannot_be_borrowed<T>() -> Result<T> {
    pre_error();
    Err("Input not allowed to be borrowed".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn wrong_opcode<T>() -> Result<T> {
    pre_error();
    Err(40)
}

#[cfg(feature = "string_errors")]
pub fn wrong_opcode<T>() -> Result<T> {
    pre_error();
    Err("opcode not defined for the requested type".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn steal_violation<T>() -> Result<T> {
    pre_error();
    Err(41)
}

#[cfg(feature = "string_errors")]
pub fn steal_violation<T>() -> Result<T> {
    pre_error();
    Err("can not steal borrows".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn branch_ret_mismatch<T>() -> Result<T> {
    pre_error();
    Err(42)
}

#[cfg(feature = "string_errors")]
pub fn branch_ret_mismatch<T>() -> Result<T> {
    pre_error();
    Err("branches must induce the same post state".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn fun_sig_mismatch<T>() -> Result<T> {
    pre_error();
    Err(43)
}

#[cfg(feature = "string_errors")]
pub fn fun_sig_mismatch<T>() -> Result<T> {
    pre_error();
    Err("Function signature mismatch".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn capability_implication_mismatch<T>() -> Result<T> {
    pre_error();
    Err(44)
}

#[cfg(feature = "string_errors")]
pub fn capability_implication_mismatch<T>() -> Result<T> {
    pre_error();
    Err("Illegal capability set".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn capability_constraints_violation<T>() -> Result<T> {
    pre_error();
    Err(45)
}

#[cfg(feature = "string_errors")]
pub fn capability_constraints_violation<T>() -> Result<T> {
    pre_error();
    Err("Type does not full fill capability requirements".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn vals_need_to_be_real<T>() -> Result<T> {
    pre_error();
    Err(46)
}

#[cfg(feature = "string_errors")]
pub fn vals_need_to_be_real<T>() -> Result<T> {
    pre_error();
    Err("Phantom Generics can not be used for values".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn wrong_protect_declaration<T>() -> Result<T> {
    pre_error();
    Err(47)
}

#[cfg(feature = "string_errors")]
pub fn wrong_protect_declaration<T>() -> Result<T> {
    pre_error();
    Err("Protected visibility must point to a generic parameter of the function".into())
}

#[cfg(not(feature = "string_errors"))]
pub fn signature_error<T>() -> Result<T> {
    pre_error();
    Err(48)
}

#[cfg(feature = "string_errors")]
pub fn signature_error<T>() -> Result<T> {
    pre_error();
    Err("Could not verify signature".into())
}
