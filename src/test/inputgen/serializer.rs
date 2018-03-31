use compiler::common::parsing::*;
use compiler::common::types::*;
use byteorder::WriteBytesExt;


pub fn add_ser_at<Ser:Serializable>(vec:&mut Vec<u8>, pos:usize, elem:Ser){
    while vec.len() < pos + elem.len() {
        vec.push(0);
    };
    elem.to_bytes(&mut vec[..(pos + elem.len())],pos);
}

pub fn push_ser<Ser:Serializable>(vec:&mut Vec<u8>, elem:Ser){
    let pos = vec.len();
    for _ in 0..elem.len() {
        vec.push(0);
    }
    elem.to_bytes(&mut vec[..(pos + elem.len())],pos);
}


pub trait Serializable{
    fn to_bytes(&self,data:&mut [u8], start:usize);
    fn len(&self) -> usize;
}

impl Serializable for u8 {
    fn to_bytes(&self, data: &mut [u8], start:usize) {
        data[start] = *self;
    }

    fn len(&self) -> usize {
        1
    }
}

impl<'a> Serializable for &'a [u8] {
    fn to_bytes(&self, data: &mut [u8], start:usize) {
        data[start..].copy_from_slice(self)
    }

    fn len(&self) -> usize {
       <[u8]>::len(self.to_owned())
    }
}


impl Serializable for Tag {
    fn to_bytes(&self, data: &mut [u8], start:usize) {
        let Tag(byte) = *self;
        data[start] = byte;
    }

    fn len(&self) -> usize {
        TAG_SIZE
    }
}

impl Serializable for Flag {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        let Flag(byte) = *self;
        data[start] = if byte {1} else {0};
    }

    fn len(&self) -> usize {
        FLAG_SIZE
    }
}


impl Serializable for Version {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        let Version(byte) = *self;
        data[start] = byte;
    }

    fn len(&self) -> usize {
        VERSION_SIZE
    }
}

impl Serializable for MemberIndex {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        let MemberIndex(byte) = *self;
        data[start] = byte;
    }

    fn len(&self) -> usize {
        MEMBER_INDEX_SIZE
    }
}

impl Serializable for Field {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        let Field(control,typ) = *self;
        typ.to_bytes(data,start);
        control.to_bytes(data, start + typ.len());
    }

    fn len(&self) -> usize {
        let Field(control,typ) = *self;
        control.len()+typ.len()
    }
}

impl Serializable for Ptr {

    fn to_bytes(&self, mut data: &mut [u8], start:usize) {
        let Ptr(short) = *self;
        let mut data:&mut [u8] = &mut data[start..];
        data.write_u16::<Endian>(short).unwrap()
    }

    fn len(&self) -> usize {
        PTR_SIZE
    }
}

impl Serializable for Length {

    fn to_bytes(&self, mut data: &mut [u8], start:usize) {
        let Length(short) = *self;
        let mut data:&mut [u8] = &mut data[start..];
        data.write_u16::<Endian>(short).unwrap()
    }

    fn len(&self) -> usize {
        LENGTH_SIZE
    }
}

impl Serializable for Coefficient {

    fn to_bytes(&self, mut data: &mut [u8], start:usize) {
        let Coefficient(short) = *self;
        let mut data:&mut [u8] = &mut data[start..];
        data.write_u16::<Endian>(short).unwrap()
    }

    fn len(&self) -> usize {
        COEFFICIENT_SIZE
    }
}

impl Serializable for ValueId {

    fn to_bytes(&self, mut data: &mut [u8], start:usize) {
        let ValueId(short) = *self;
        let mut data:&mut [u8] = &mut data[start..];
        data.write_u16::<Endian>(short).unwrap()
    }

    fn len(&self) -> usize {
        VALUE_ID_SIZE
    }
}

impl Serializable for TypeId {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        let TypeId(byte) = *self;
        data[start] = byte;
    }

    fn len(&self) -> usize {
        TYPE_ID_SIZE
    }
}

impl Serializable for ModuleId {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        let ModuleId(byte) = *self;
        data[start] = byte;
    }

    fn len(&self) -> usize {
        MODULE_ID_SIZE
    }
}

impl Serializable for FunId {
    fn to_bytes(&self, data: &mut [u8], start:usize) {
        let FunId(byte) = *self;
        data[start] = byte;
    }

    fn len(&self) -> usize {
        FUN_ID_SIZE
    }
}


impl Serializable for InitId {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        let InitId(byte) = *self;
        data[start] = byte;
    }

    fn len(&self) -> usize {
        INIT_ID_SIZE
    }
}

impl Serializable for CtrId {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        let CtrId(byte) = *self;
        data[start] = byte;
    }

    fn len(&self) -> usize {
        CTR_ID_SIZE
    }
}

impl Serializable for ParamId {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        let ParamId(byte) = *self;
        data[start] = byte;
    }

    fn len(&self) -> usize {
        PARAM_ID_SIZE
    }
}

impl Serializable for Amount {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        let Amount(byte) = *self;
        data[start] = byte;
    }

    fn len(&self) -> usize {
        AMOUNT_SIZE
    }
}

impl Serializable for Privileges {

    fn to_bytes(&self, mut data: &mut [u8], start:usize) {
        let Privileges(short) = *self;
        let mut data:&mut [u8] = &mut data[start..];
        data.write_u16::<Endian>(short).unwrap();
    }

    fn len(&self) -> usize { PRIVILEGES_SIZE }
}

impl Serializable for Control {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        data[start] = match *self {
            Control::Ref => 0,
            Control::Owned => 1,
            Control::Borrowed => 2,
            Control::UnusedOwned => 3,
            Control::UnusedBorrowed => 4
        }
    }

    fn len(&self) -> usize {
        CONTROL_SIZE
    }
}

impl Serializable for Bound {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        data[start] = match *self {
            Bound::Phantom => 0,
            Bound::Dynamic => 1
        }
    }

    fn len(&self) -> usize {
        BOUND_SIZE
    }
}

impl Serializable for Visibility {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        data[start] = match *self {
            Visibility::Public => 0,
            Visibility::Private => 1
        }
    }

    fn len(&self) -> usize {
        VISIBILITY_SIZE
    }
}

impl Serializable for TypeKind {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        data[start] = match *self {
            TypeKind::Cell => 0,
            TypeKind::View => 1,
            TypeKind::Normal => 2
        }
    }

    fn len(&self) -> usize {
        TYPE_KIND_SIZE
    }
}

impl Serializable for ExecutionMode {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        data[start] = match *self {
            ExecutionMode::Pure => 0,
            ExecutionMode::Init => 1,
            ExecutionMode::Dependent => 2,
            ExecutionMode::Active => 3
        }
    }

    fn len(&self) -> usize {
        EXECUTION_MODE_SIZE
    }
}

impl Serializable for OptimizationDeclaration {

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        data[start] = match *self {
            OptimizationDeclaration::Empty => 0,
            OptimizationDeclaration::Wrapper => 1,
            OptimizationDeclaration::Normal => 2,
            OptimizationDeclaration::EmptyWrapper => 3,
        }
    }

    fn len(&self) -> usize {
        OPTIMIZATION_DECLARATION_SIZE
    }
}


impl<'a> Serializable for Hash<'a>{

    fn to_bytes(&self, data: &mut [u8], start:usize) {
        data[start..].copy_from_slice(self.d)
    }

    fn len(&self) -> usize {
        self.d.len()
    }
}
