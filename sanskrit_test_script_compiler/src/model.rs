use std::collections::HashSet;
use std::collections::HashMap;
use sanskrit_common::capabilities::CapSet;
use script::Index;


pub struct Module {
    pub name: Id,
    pub components: Vec<Component>,
}


pub enum Component {
    Adt{caps:CapSet, name:Id, generics:Vec<Generic>, ctrs:Vec<Case>},
    Err{name:Id},
    Fun{name:Id, vis:Visibility, risks:Vec<Ref>, generics:Vec<Generic>, params:Vec<Var>, returns:Vec<Ret> ,code:Block},
}


pub enum Visibility{
    Private,
    Public,
    Protected(Vec<Id>)
}


pub struct Case {
    pub name:Id,
    pub params:Vec<Type>
}


pub struct Var{
    pub name:Id,
    pub consume:bool,
    pub typ:Type
}


pub struct Ret{
    pub name:Id,
    pub borrow:Vec<Id>,
    pub typ:Type
}


pub struct Generic{
    pub name:Id,
    pub caps:CapSet,
    pub phantom:bool,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Type{
    pub main:Ref,
    pub applies:Vec<Type>
}


pub struct Match{
    pub ctr:Id,
    pub params:Vec<Id>,
    pub code:Block
}


pub struct Catch{
    pub error:Ref,
    pub code:Block
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Id(pub String);

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Lit(pub String);


pub enum Block{
    Error(Ref),
    Return(Vec<OpCode>, Vec<Id>, Vec<Id>),
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum Ref{
    Generic(Id),
    Native(Id),
    This(Id),
    Module(Id,Id),
    Txt(Lit,Lit),
    Account(Lit)
}


pub enum OpCode {
    Lit(Id, Lit, Type),
    Let(Vec<Id>, Box<Block>),
    Copy(Id,Id),
    Fetch(Id,Id,bool),
    Field(Id,Id,Lit,Type,bool),
    CopyField(Id,Id,Lit,Type),
    Drop(Id),
    Free(Id),
    Unpack(Vec<Id>, Id, Type, bool),
    Switch(Vec<Id>, Id, Type, Vec<Match>, bool),
    Pack(Id, Type, Id, Vec<Id>, bool),
    Call(Vec<Id>, Ref, Vec<Type>, Vec<Id>),
    Try(Vec<Id>, Box<Block>, Vec<Catch>),
    ModuleIndex(Id),
}



impl Module {

    pub fn collect_module_dependencies(&self) -> HashSet<Id>{
        let mut aggr = HashSet::new();
        for c in &self.components {
            c.collect_module_dependencies(&mut aggr);
        }
        aggr
    }

    pub fn create_index(&self) -> HashMap<Id,Index> {
        let mut index = HashMap::new();
        let mut funs = 0;
        let mut errs = 0;
        let mut adts = 0;
        for (idx,c) in self.components.iter().enumerate() {
             match *c {
                 Component::Adt { .. } => {
                     index.insert(c.get_id(), Index{ comp_index: idx, elem_index: adts });
                     adts+=1;
                 },
                 Component::Err { .. } => {
                     index.insert(c.get_id(), Index{ comp_index: idx, elem_index: errs });
                     errs+=1;
                 },
                 Component::Fun { .. } => {
                     index.insert(c.get_id(), Index{ comp_index: idx, elem_index: funs });
                     funs+=1;
                 },
            }
        }
        index
    }
}

impl Component {
    fn get_id(&self) -> Id {
        match *self {
            Component::Adt { ref name, .. } |
            Component::Err { ref name, .. } |
            Component::Fun { ref name, .. } => name.clone()
        }
    }

    fn collect_module_dependencies(&self, aggr:&mut HashSet<Id>){
       match *self {
           Component::Adt { ref ctrs, .. } => {
               for c in ctrs {
                   for p in &c.params {
                       p.collect_module_dependencies(aggr)
                   }
               }
           },
           Component::Fun { ref risks, ref params, ref code, ref returns, .. } => {
               for r in risks {
                   r.collect_module_dependencies(aggr);
               }
               for p in params {
                   p.typ.collect_module_dependencies(aggr);
               }
               for p in returns {
                   p.typ.collect_module_dependencies(aggr);
               }
               code.collect_module_dependencies(aggr);
           },
           _ => {}
       }
    }
}


impl Type {
    fn collect_module_dependencies(&self, aggr: &mut HashSet<Id>) {
        self.main.collect_module_dependencies(aggr);
        for a in &self.applies {
            a.collect_module_dependencies(aggr);
        }
    }
}

impl Ref {
    fn collect_module_dependencies(&self, aggr: &mut HashSet<Id>) {
        match *self {
            Ref::Module(ref module, _) => {aggr.insert(module.clone());},
            _ => {},
        };
    }
}

impl OpCode {

    fn collect_module_dependencies(&self, aggr: &mut HashSet<Id>) {
        match *self {


            OpCode::Lit(_, _, ref typ) => typ.collect_module_dependencies(aggr),
            OpCode::Let(_, ref block) => block.collect_module_dependencies(aggr),
            OpCode::Unpack(_, _, ref typ, _) => typ.collect_module_dependencies(aggr),
            OpCode::Switch(_, _, ref typ, ref matches, _) => {
                typ.collect_module_dependencies(aggr);
                for mat in matches {
                    mat.code.collect_module_dependencies(aggr);
                }
            },
            OpCode::Pack(_, ref typ, _, _, _) => typ.collect_module_dependencies(aggr),
            OpCode::Call(_, ref r, ref typs, _) => {
                r.collect_module_dependencies(aggr);
                for typ in typs {
                    typ.collect_module_dependencies(aggr);
                }
            },
            OpCode::Try(_, ref block, ref catches) => {
                block.collect_module_dependencies(aggr);
                for catch in catches {
                    catch.code.collect_module_dependencies(aggr);
                    catch.error.collect_module_dependencies(aggr);
                }
            },

            _ => {},
        };
    }
}

impl Block {
    fn collect_module_dependencies(&self, aggr: &mut HashSet<Id>) {
        match *self {
            Block::Error(ref r) => r.collect_module_dependencies(aggr),
            Block::Return(ref ops,_,_) => {
                for op in ops {
                    op.collect_module_dependencies(aggr);
                }
            },
        };
    }
}

pub struct Transaction {
    pub sigs:Vec<Id>,
    pub news:Vec<Id>,
    pub codes:Vec<ScriptCode>
}


pub enum ScriptCode {
    Lit(Id, Lit, Type),
    Wit(Id, Lit, Type),
    RefGen(Id,Id),
    Copy(Id,Id),
    Fetch(Id,Id,bool),
    Drop(Id),
    Free(Id),
    Unpack(Vec<Id>, Id, Ref, Id, bool),
    Pack(Id, Type, Id, Vec<Id>, bool),
    Call(Vec<Id>, Ref, Vec<Type>, Vec<Id>),
    Token(Id, Id, bool),
    Load(Id, Id, bool),
    Store(Id),
}


impl Transaction {
    pub fn collect_module_dependencies(&self) -> HashSet<Id>{
        let mut aggr = HashSet::new();
        for s in &self.codes {
            s.collect_module_dependencies(&mut aggr);
        }
        aggr
    }
}

impl ScriptCode {

    fn collect_module_dependencies(&self, aggr: &mut HashSet<Id>) {
        match *self {
            ScriptCode::Lit(_, _, ref typ) => typ.collect_module_dependencies(aggr),
            ScriptCode::Unpack(_, _, ref main, _, _) => main.collect_module_dependencies(aggr),
            ScriptCode::Pack(_, ref typ, _, _, _) => typ.collect_module_dependencies(aggr),
            ScriptCode::Call(_, ref r, ref typs, _) => {
                r.collect_module_dependencies(aggr);
                for typ in typs {
                    typ.collect_module_dependencies(aggr);
                }
            },
            _ => {},
        };
    }
}