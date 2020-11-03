use manager::{State, Param, Ret};
use sanskrit_common::errors::*;
use sanskrit_common::model::{Hash, hash_from_slice};
use sanskrit_common::encoding::{Serializable, Serializer};
use hex::decode;
use convert_error;
use sanskrit_common::hashing::HashingDomain;
use std::collections::BTreeSet;

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Debug)]
pub struct Execute{
    pub name:String,
    pub params:Vec<ParamInput>,
    pub rets:Vec<RetInput>
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Debug)]
pub enum ParamInput {
    Lit(LitInput),
    Sig(String),
    Pk(String),
    Consume(String),
    Read(String),
    Copy(String),
    Inject
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Debug)]
pub enum RetInput {
    Log(String),
    Store(String),
    Assign(String),
    Drop
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Debug)]
pub enum LitInput {
    Id(String),
    Derive(Box<LitInput>,Box<LitInput>),
    Data(String),
    Union(u8,Vec<LitInput>),
    Struct(Vec<LitInput>),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128)
}


impl Execute {
    pub fn txt_hash(&self, state:&State) -> Result<Hash> {
        match convert_error(state.transaction_name_mapping.get(&self.name))? {
            None => error(||"Transaction name unknown"),
            Some(h) => Ok(hash_from_slice(&h)),
        }
    }

    pub fn build_params(&self, state:&State, local_bindings:&BTreeSet<String>) -> Result<Vec<Param>> {

        fn derive(main:&[u8], derive:&[u8], s:&mut Serializer) {
            let mut context = HashingDomain::Derive.get_domain_hasher();
            context.update(&main);
            context.update(&derive);
            context.finalize().serialize(s).unwrap();
        }

        fn build_top_lit(input:&LitInput, state:&State) -> Result<Vec<u8>> {
            let mut s = Serializer::new(16);
            build_lit(input, state, &mut s)?;
            Ok(s.extract())
        }

        fn build_lit(input:&LitInput, state:&State, s:&mut Serializer) -> Result<()> {
            Ok(match input {
                LitInput::Id(name) => match convert_error(state.module_name_mapping.get(name))? {
                    Some(h) => hash_from_slice(&h).serialize(s)?,
                    None => return error(||"Module name unknown")
                },
                LitInput::Derive(first, second) => {
                    let first_data = build_top_lit(first, state)?;
                    let second_data = build_top_lit(second, state)?;
                    derive(&first_data, &second_data, s)
                }

                LitInput::Data(unparsed_hex_data) => s.produce_bytes(&convert_error(decode(&unparsed_hex_data[2..]))?),
                LitInput::Union(tag, fields) => {
                    tag.serialize(s)?;
                    for f in fields { build_lit(f,state, s)?; }
                },
                LitInput::Struct(fields) => for f in fields { build_lit(f,state, s)?; },
                LitInput::U8(num) => num.serialize(s)?,
                LitInput::U16(num) => num.serialize(s)?,
                LitInput::U32(num) => num.serialize(s)?,
                LitInput::U64(num) => num.serialize(s)?,
                LitInput::U128(num) => num.serialize(s)?,
                LitInput::I8(num) => num.serialize(s)?,
                LitInput::I16(num) => num.serialize(s)?,
                LitInput::I32(num) => num.serialize(s)?,
                LitInput::I64(num) => num.serialize(s)?,
                LitInput::I128(num) => num.serialize(s)?,
            })
        }

        let mut res = Vec::with_capacity(self.params.len());
        for param in &self.params {
            res.push(match param{
                ParamInput::Lit(input) => Param::Lit(build_top_lit(input,state)?),
                ParamInput::Sig(name) => Param::Sig(name.clone()),
                ParamInput::Pk(name) => Param::Pk(name.clone()),
                ParamInput::Consume(key) => if local_bindings.contains(key) {
                    Param::LocalConsume(key.clone())
                } else {
                    Param::Consume(match convert_error(state.tracking.active_elems.get(key))? {
                        None => return error(||"Element name unknown"),
                        Some(h) => hash_from_slice(&h),
                    })
                },

                ParamInput::Read(key) => if local_bindings.contains(key) {
                    Param::LocalBorrow(key.clone())
                } else {
                    Param::Borrow(match convert_error(state.tracking.active_elems.get(key))?{
                        None => return error(||"Element name unknown"),
                        Some(h) => hash_from_slice(&h),
                    })
                },
                ParamInput::Copy(key) => if local_bindings.contains(key) {
                    Param::LocalCopy(key.clone())
                } else {
                    Param::Copy(match convert_error(state.tracking.active_elems.get(key))?{
                        None => return error(||"Element name unknown"),
                        Some(h) => hash_from_slice(&h),
                    })
                },
                ParamInput::Inject => Param::Provided,
            })
        }

        Ok(res)
    }

    pub fn build_returns(&self, bindings:&mut BTreeSet<String>) -> Vec<Ret> {
        self.rets.iter().map(|ret|match ret{
            RetInput::Log(_) => Ret::Log,
            RetInput::Store(_) => Ret::Elem,
            RetInput::Drop => Ret::Drop,
            RetInput::Assign(name) => {
                bindings.insert(name.clone());
                Ret::Assign(name.clone())
            }
        }).collect()
    }

    pub fn build_param_names(&self, state:&mut State) {
        for param in &self.params {
            state.tracking.exec_state.param_names.push_back(match param{
                ParamInput::Lit(_) => "(lit)".to_string(),
                ParamInput::Sig(name) => name.clone(),
                ParamInput::Pk(name) => name.clone(),
                ParamInput::Consume(name) => name.clone(),
                ParamInput::Read(name) => name.clone(),
                ParamInput::Copy(name) => name.clone(),
                ParamInput::Inject => "(injected)".to_string()
            })
        }
    }

    pub fn build_return_names(&self,state:&mut State) {
        for ret in &self.rets {
            state.tracking.exec_state.return_names.push_back(match ret{
                RetInput::Log(name) => name.clone(),
                RetInput::Store(name) => name.clone(),
                RetInput::Assign(name) => name.clone(),
                RetInput::Drop => "".to_owned(),
            })
        }
    }
}