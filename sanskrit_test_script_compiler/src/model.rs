use std::collections::HashSet;
use std::collections::HashMap;
use script::Index;
use sanskrit_runtime::model::{RetType, ParamMode};
use sanskrit_core::model::bitsets::CapSet;
use sanskrit_common::model::Hash;

pub struct Module {
    pub is_system:bool,
    pub name: Id,
    pub components: Vec<Component>,
}


pub enum Component {
    Txt{ transactional:bool, params:Vec<In>, returns:Vec<Out> ,code:Block},
    Impl{perms:Vec<(Vec<Id>, Visibility)>, name:Id, generics:Vec<Generic>, captures:Vec<Var>, sig:Type, params:Vec<Id>, code:Block},
    Adt{perms:Vec<(Vec<Id>, Visibility)>, top:bool, caps:CapSet, name:Id, generics:Vec<Generic>, ctrs:Vec<Case>},
    ExtLit{perms:Vec<(Vec<Id>, Visibility)>, top:bool, caps:CapSet, name:Id, generics:Vec<Generic>, size:u16},
    Sig{perms:Vec<(Vec<Id>, Visibility)>, transactional:bool, caps:CapSet, name:Id, generics:Vec<Generic>, params:Vec<Var>, returns:Vec<Ret>},
    Fun{perms:Vec<(Vec<Id>, Visibility)>, transactional:bool, name:Id, generics:Vec<Generic>, params:Vec<Var>, returns:Vec<Ret> ,code:Block},
    ExtFun{perms:Vec<(Vec<Id>, Visibility)>, transactional:bool, name:Id, generics:Vec<Generic>, params:Vec<Var>, returns:Vec<Ret>},

}

//A transaction
#[derive(Eq, PartialEq, Clone)]
pub struct TransactionCompilationResult {
    //Consts:
    //transaction type
    pub txt_fun: Vec<u8>,
    //parameter source & fetch mode
    pub param_types: Vec<ParamData>,
    //returns jobs
    pub ret_types: Vec<RetType>,

}

#[derive(Eq, PartialEq, Clone)]
pub enum ParamData {
    Load(ParamMode, Hash),
    Literal(Vec<u8>),
    Witness(Vec<u8>),
    Provided
}

#[derive(Eq, PartialEq, Clone)]
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
    pub typ:Type
}

pub enum In {
    Copy(Key,Id,Type),
    Borrow(Key,Id,Type),
    Consume(Key,Id,Type),
    Literal(Data,Id,Type),
    Witness(Data,Id,Type),
    Provided(Id,Type)
}

pub enum Key {
    Plain(Lit),
    Derived(Id,Lit)
}

pub enum Out {
    Store(Id,Type),
    Drop(Id,Type),
    Log(Id,Type)
}

pub enum Data{
    Adt{ typ:Ref, ctr:Id, params:Vec<Data> },
    Lit{ typ:Ref, lit:Lit }
}

pub struct Generic{
    pub name:Id,
    pub caps:CapSet,
    pub phantom:bool,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Type{
    pub main:Ref,
    pub applies:Vec<Type>,
    pub projections:usize,
}


pub struct Match{
    pub ctr:Id,
    pub params:Vec<Id>,
    pub code:Block
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Id(pub String);

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Lit(pub String);


pub struct Block{
    pub codes:Vec<OpCode>
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum Ref{
    Generic(Id),
    This(Id),
    Module(Id,Id)
}


pub enum OpCode {
    Lit(Id, Lit, Type),
    Let(Vec<Id>, Box<Block>),
    Copy(Id,Id),
    Return(Vec<Id>, Vec<Id>),
    Project(Id, Id, Type),
    UnProject(Id, Id, Type),
    Fetch(Id,Id),
    Field(Id,Id,Lit,Type),
    CopyField(Id,Id,Lit,Type),
    Discard(Id),
    DiscardMany(Vec<Id>),
    Unpack(Vec<Id>, Id, Type),
    Switch(Vec<Id>, Id, Type, Vec<Match>),
    Inspect(Vec<Id>, Id, Type, Vec<Match>),
    Pack(Id, Type, Id, Vec<Id>),
    CreateSig(Id, Type, Vec<Id>),
    CallSig(Vec<Id>, Id, Type, Vec<Id>),
    Call(Vec<Id>, Ref, Vec<Type>, Vec<Id>),
    TryCall(Vec<Id>, Ref, Vec<Type>, Vec<(bool,Id)>, Vec<Match>),
    Abort(Vec<Id>, Vec<Id>,  Vec<Type>)

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
        let mut types = 0;
        let mut impls = 0;
        let mut sigs = 0;

        for (idx,c) in self.components.iter().enumerate() {
             match *c {
                 Component::Adt { .. } | Component::ExtLit {..} => {
                     index.insert(c.get_id(), Index{ comp_index: idx, elem_index: types });
                     types+=1;
                 },
                 Component::Fun { .. } | Component::ExtFun { .. }=> {
                     index.insert(c.get_id(), Index{ comp_index: idx, elem_index: funs });
                     funs+=1;
                 },
                 Component::Impl { .. } => {
                     index.insert(c.get_id(), Index{ comp_index: idx, elem_index: impls });
                     impls+=1;
                 },
                 Component::Sig { .. } => {
                     index.insert(c.get_id(), Index{ comp_index: idx, elem_index: sigs });
                     sigs+=1;
                 },
                 Component::Txt { .. } => unreachable!()
             }
        }
        index
    }
}

impl Component {
    fn get_id(&self) -> Id {
        match *self {
            Component::Adt { ref name, .. }
            | Component::ExtLit { ref name, .. }
            | Component::Fun { ref name, .. }
            | Component::ExtFun { ref name, .. }
            | Component::Impl { ref name, .. }
            | Component::Sig { ref name, .. } => name.clone(),
            Component::Txt { .. } => unreachable!()

        }
    }

    pub fn collect_module_dependencies(&self, aggr:&mut HashSet<Id>){
       match *self {
           Component::Adt { ref ctrs, .. } => {
               for c in ctrs {
                   for p in &c.params {
                       p.collect_module_dependencies(aggr)
                   }
               }
           },
           Component::Fun { ref params, ref code, ref returns, .. } => {
               for p in params {
                   p.typ.collect_module_dependencies(aggr);
               }
               for p in returns {
                   p.typ.collect_module_dependencies(aggr);
               }
               code.collect_module_dependencies(aggr);
           },
           Component::ExtFun { ref params, ref returns, .. } => {
               for p in params {
                   p.typ.collect_module_dependencies(aggr);
               }
               for p in returns {
                   p.typ.collect_module_dependencies(aggr);
               }
           },
           Component::Txt { ref params, ref code, ref returns, .. } => {
               for p in params {
                   p.collect_module_dependencies(aggr);
               }
               for p in returns {
                   p.collect_module_dependencies(aggr);
               }
               code.collect_module_dependencies(aggr);
           }
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

impl In {
    fn collect_module_dependencies(&self, aggr: &mut HashSet<Id>) {
        match *self {
            In::Copy(ref key, _, ref typ)
            | In::Borrow(ref key, _, ref typ)
            | In::Consume(ref key, _, ref typ) => {
                key.collect_module_dependencies(aggr);
                typ.collect_module_dependencies(aggr)
            },
            In::Literal(ref data, _, ref typ)
            | In::Witness(ref data, _, ref typ) => {
                typ.collect_module_dependencies(aggr);
                data.collect_module_dependencies(aggr);
            },
            In::Provided(_, ref typ)=> {
                typ.collect_module_dependencies(aggr);
            }
        };
    }

    pub fn name(&self) -> Id {
        match *self {
            In::Copy(_, ref name, _)
            | In::Borrow(_, ref name, _)
            | In::Consume(_, ref name, _)
            | In::Provided(ref name, _)
            | In::Literal(_, ref name, _)
            | In::Witness(_, ref name, _) => name.clone(),
        }
    }

    pub fn consume(&self) -> bool {
        match *self {
            In::Copy(_, _, _)
            | In::Consume(_, _, _)
            | In::Provided(_, _)
            | In::Literal(_, _, _)
            | In::Witness(_, _, _)  => true,
            In::Borrow(_, _, _) => false

        }
    }

    pub fn is_val(&self) -> bool {
        match *self {
            In::Copy(_, _, _)
            | In::Consume(_, _, _)
            | In::Borrow(_, _, _)=> true,
            In::Provided(_, _)
            | In::Literal(_, _, _)
            | In::Witness(_, _, _)=> false,

        }
    }

    pub fn is_lit(&self) -> bool {
        match *self {
            In::Copy(_, _, _)
            | In::Consume(_, _, _)
            | In::Borrow(_, _, _)
            | In::Provided(_, _)
            | In::Witness(_, _, _) => false,
            In::Literal(_, _, _)=> true,

        }
    }

    pub fn is_wit(&self) -> bool {
        match *self {
            In::Copy(_, _, _)
            | In::Consume(_, _, _)
            | In::Borrow(_, _, _)
            | In::Provided(_, _)
            | In::Literal(_, _, _) => false,
            In::Witness(_, _, _) => true,
        }
    }

    pub fn typ(&self) -> &Type {
        match *self {
            In::Copy(_, _, ref typ)
            | In::Borrow(_, _, ref typ)
            | In::Consume(_, _, ref typ)
            | In::Literal(_, _, ref typ)
            | In::Witness(_, _, ref typ)
            | In::Provided(_, ref typ) => typ,

        }
    }
}

impl Key {
    fn collect_module_dependencies(&self, aggr: &mut HashSet<Id>) {
        match *self {
            Key::Plain(_) => {},
            Key::Derived(ref module, _) => {aggr.insert(module.clone()); },
        };
    }
}

impl Out {
    fn collect_module_dependencies(&self, aggr: &mut HashSet<Id>) {
        match *self {
            Out::Drop(_, ref typ)
            | Out::Log(_, ref typ)
            | Out::Store(_, ref typ) => typ.collect_module_dependencies(aggr),
        };
    }

    pub fn name(&self) -> Id {
        match *self {
            Out::Store(ref name, _)
            | Out::Drop(ref name, _)
            | Out::Log(ref name, _)  => name.clone(),
        }
    }


    pub fn typ(&self) -> &Type {
        match *self {
            Out::Drop(_, ref typ)
            | Out::Log(_, ref typ)
            | Out::Store(_, ref typ) => typ,
        }
    }
}

impl Data {
    fn collect_module_dependencies(&self, aggr: &mut HashSet<Id>) {
        match self {
            Data::Adt { ref typ, ref params, .. } => {
                typ.collect_module_dependencies(aggr);
                for p in params {
                    p.collect_module_dependencies(aggr);
                }
            },
            Data::Lit { ref typ, .. } =>  typ.collect_module_dependencies(aggr),
        }

    }
}

impl OpCode {

    fn collect_module_dependencies(&self, aggr: &mut HashSet<Id>) {
        match *self {


            OpCode::Lit(_, _, ref typ) => typ.collect_module_dependencies(aggr),
            OpCode::Let(_, ref block) => block.collect_module_dependencies(aggr),
            OpCode::Unpack(_, _, ref typ) => typ.collect_module_dependencies(aggr),
            OpCode::Switch(_, _, ref typ, ref matches) => {
                typ.collect_module_dependencies(aggr);
                for mat in matches {
                    mat.code.collect_module_dependencies(aggr);
                }
            },
            OpCode::Pack(_, ref typ, _, _) => typ.collect_module_dependencies(aggr),
            OpCode::TryCall(_, ref r, ref typs, _, ref matches) => {
                r.collect_module_dependencies(aggr);
                for typ in typs {
                    typ.collect_module_dependencies(aggr);
                }
                for mat in matches {
                    mat.code.collect_module_dependencies(aggr);
                }
            }
            OpCode::Call(_, ref r, ref typs, _) => {
                r.collect_module_dependencies(aggr);
                for typ in typs {
                    typ.collect_module_dependencies(aggr);
                }
            },

            _ => {},
        };
    }
}

impl Block {
    fn collect_module_dependencies(&self, aggr: &mut HashSet<Id>) {
        for op in &self.codes {
            op.collect_module_dependencies(aggr);
        }
    }
}