use compiler::common::types::*;

impl Privileges {

    pub fn no_privileges() -> Self {
        Privileges(0)
    }

    pub fn new(privilege:u16) -> Self {
        Privileges(privilege)
    }

    pub fn merge_privileges(&self, privileges:&Self) -> Self {
        let &Privileges(cur) = self;
        let &Privileges(other) = privileges;
        Privileges(cur | other)
    }

    pub fn is_valid(&self) -> bool {
        if self.has_privilege(1 << 9) & !self.has_privilege(EXTRACT_PRIVILEGE) {return false}
        if self.has_privilege(1 << 10) & !self.has_privilege(CREATE_PRIVILEGE) {return false}
        if self.has_privilege(1 << 0) & !self.has_privilege(PERSIST_PRIVILEGE) {return false}
        if self.has_privilege(1 << 1) & !self.has_privilege(PERSIST_PRIVILEGE) {return false}
        if self.has_privilege(1 << 4) & !self.has_privilege(PERSIST_PRIVILEGE) {return false}
        if self.has_privilege(1 << 5) & !self.has_privilege(PERSIST_PRIVILEGE) {return false}
        return true;
    }

    pub fn add_privilege(&self,privilege:u16) -> Self {
        let &Privileges(cur) = self;
        Privileges(cur | privilege)
    }

    pub fn has_privilege(&self,privilege:u16) -> bool {
        let &Privileges(cur) = self;
        (cur & privilege) == privilege
    }
    
    pub fn strip_non_recursive(&self) -> Self {
        let &Privileges(cur) = self;
        Privileges(cur & !((WRITE_PRIVILEGE | LOAD_PRIVILEGE | ACCESS_PRIVILEGE | CREATE_PRIVILEGE | UNWRAP_PRIVILEGE | WRAP_PRIVILEGE) & !PERSIST_PRIVILEGE))
    }

    pub fn add_unwrap_privilege(&self) -> Self {
        self.add_privilege(UNWRAP_PRIVILEGE)
    }

    pub fn add_wrap_privilege(&self) -> Self {
        self.add_privilege(WRAP_PRIVILEGE)
    }

    pub fn add_access_privilege(&self) -> Self {
        self.add_privilege(ACCESS_PRIVILEGE)
    }

    pub fn add_create_privilege(&self) -> Self {
        self.add_privilege(CREATE_PRIVILEGE)
    }

    pub fn add_load_privilege(&self) -> Self {
        self.add_privilege(LOAD_PRIVILEGE)
    }

    pub fn add_write_privilege(&self) -> Self {
        self.add_privilege(WRITE_PRIVILEGE)
    }

    pub fn add_discard_privilege(&self) -> Self {
        self.add_privilege(DISCARD_PRIVILEGE)
    }

    pub fn add_copy_privilege(&self) -> Self {
        self.add_privilege(COPY_PRIVILEGE)
    }

    pub fn add_persist_privilege(&self) -> Self {
        self.add_privilege(PERSIST_PRIVILEGE)
    }

    pub fn add_native_privilege(&self) -> Self {
        self.add_privilege(NATIVE_PRIVILEGE)
    }

    pub fn has_unwrap_privilege(&self) -> bool {
        self.has_privilege(UNWRAP_PRIVILEGE)
    }

    pub fn has_wrap_privilege(&self) -> bool {
        self.has_privilege(WRAP_PRIVILEGE)
    }

    pub fn has_extract_privilege(&self) -> bool {
        self.has_privilege(EXTRACT_PRIVILEGE)
    }

    pub fn has_assemble_privilege(&self) -> bool {
        self.has_privilege(ASSEMBLE_PRIVILEGE)
    }

    pub fn has_access_privilege(&self) -> bool {
        self.has_privilege(ACCESS_PRIVILEGE)
    }

    pub fn has_create_privilege(&self) -> bool {
        self.has_privilege(CREATE_PRIVILEGE)
    }

    pub fn has_load_privilege(&self) -> bool {
        self.has_privilege(LOAD_PRIVILEGE)
    }

    pub fn has_write_privilege(&self) -> bool {
        self.has_privilege(WRITE_PRIVILEGE)
    }

    pub fn has_discard_privilege(&self) -> bool {
        self.has_privilege(DISCARD_PRIVILEGE)
    }

    pub fn has_copy_privilege(&self) -> bool {
        self.has_privilege(COPY_PRIVILEGE)
    }

    pub fn has_persist_privilege(&self) -> bool {
        self.has_privilege(PERSIST_PRIVILEGE)
    }

    pub fn has_native_privilege(&self) -> bool {
        self.has_privilege(NATIVE_PRIVILEGE)
    }

    pub fn implies(&self, privileges:&Privileges) -> bool{
        let (&Privileges(me),&Privileges(other)) = (self, privileges);
        !(!me | other) == 0
    }
}

impl ExecutionMode {
    pub fn satisfies(self,mode:ExecutionMode) -> bool{
        match (self,mode) {
            (ExecutionMode::Pure, _) => true,
            (ExecutionMode::Init, ExecutionMode::Pure) => false,
            (ExecutionMode::Init, _) => true,
            (ExecutionMode::Dependent, ExecutionMode::Pure) => false,
            (ExecutionMode::Dependent, ExecutionMode::Init) => false,
            (ExecutionMode::Dependent, _) => true,
            (ExecutionMode::Active, ExecutionMode::Active) => true,
            _ => false
        }
    }
}