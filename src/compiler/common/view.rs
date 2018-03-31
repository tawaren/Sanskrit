use compiler::common::macros::view::*;
use compiler::errors::parsing::*;
use compiler::common::types::*;
use compiler::typ::import::view::*;
use compiler::typ::view::*;
use compiler::function::import::view::*;

pub const PARSER_VERSION:u8 = 0;

pub struct FieldLayout {
    pub start: usize,
    pub len: usize
}

impl FieldLayout {
    pub const fn first(len:usize) -> Self {
        FieldLayout{start:0,len}
    }
    pub const fn after(self, len:usize) -> Self {
        FieldLayout{start:self.start+self.len,len}
    }
}

//Fixed Size Pointers
// Layout:
//  Value 1
//  ....
//  Value n
view_pointer_impl!(PRIVILEGES_SIZE, PrivilegesViewPointer, Privileges, Privileges);
view_pointer_impl!(BOUND_SIZE, BoundViewPointer, Bound, Bound);
view_pointer_impl!(CONTROL_SIZE, ControlViewPointer, Control, Control);
view_pointer_impl!(TYPE_ID_SIZE, TypeIdViewPointer, TypeId, TypeId);
view_pointer_impl!(FIELD_SIZE, FieldViewPointer, Field, Field);
//view_pointer_impl!(2, SizePolyViewPointer, Coefficient, Coefficient);
view_pointer_impl!(HASH_SIZE, HashViewPointer, Hash<'a>, Hash );
view_pointer_impl!(1, ByteViewPointer, u8, u8);
//Dynamic Sized Pointers
// Layout:
//  Index 1
//  ....
//  Index n
// ....
// Value 1 (Index 1 Points Here)
// ....
// ....
// Value n (Index n Points Here)
view_index_pointer_impl!(ImportedTypePointer,ImportedTypeView<'a>, ImportedTypeView);
view_index_pointer_impl!(ImportedConstructorsPointer,ImportedConstructorsView<'a>,ImportedConstructorsView);
view_index_pointer_impl!(ImportedInitPointer,ImportedInitView<'a>,ImportedInitView);
view_index_pointer_impl!(ImportedFunctionPointer,ImportedFunctionView<'a>,ImportedFunctionView);
view_index_pointer_impl!(CtrViewPointer,CtrView<'a>,CtrView);
