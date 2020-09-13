use model::*;
use environment::*;
use std::collections::HashMap;
use sanskrit_core::model::{OpCode as ROpCode, TypeRef};
use sanskrit_core::model::Accessibility as RVisibility;
use sanskrit_core::model::Case as RCase;
use sanskrit_core::model::Generic as RGeneric;
use sanskrit_core::model::Exp;
use sanskrit_common::model::LargeVec;
use sanskrit_common::model::Tag;
use sanskrit_core::model::GenRef;
use sanskrit_core::model::Param;
use sanskrit_common::encoding::ParserAllocator;
use hex::decode;
use byteorder::{WriteBytesExt};
use sanskrit_common::encoding::EncodingByteOrder;
use sanskrit_interpreter::model::LitDesc;


impl Block {
    pub fn compile_with_input<A: ParserAllocator>(&self, inputs:&[Id], env:&mut Environment<A>, imp:&mut CodeImportBuilder) -> Result<Exp,String> {
        let reg = Region(HashMap::new());
        let height = env.stack_depth;
        env.frames.push(reg);
        for input in inputs {
            env.push_new(input.clone())
        }
        let op_codes = LargeVec(self.codes.iter().map(|c|c.compile(env, imp)).collect::<Result<_,_>>()?);
        env.frames.pop();
        env.stack_depth = height;
        Ok(Exp(op_codes))
    }

    pub fn compile<A: ParserAllocator>(&self, env:&mut Environment<A>, imp:&mut CodeImportBuilder) -> Result<Exp, String> {
        self.compile_with_input(&[], env, imp)
    }
}

fn into_vec<F:FnOnce(&mut Vec<u8>)>(f:F)-> Vec<u8>{
    let mut res = Vec::new();
    f(&mut res);
    res
}

pub fn parse_lit_from_desc(input:&str, lit_desc:LitDesc) -> Vec<u8>{
    match lit_desc {
        LitDesc::U8 => into_vec(|res|res.write_u8(input.parse::<u8>().unwrap()).unwrap()),
        LitDesc::I8 => into_vec(|res|res.write_i8(input.parse::<i8>().unwrap()).unwrap()),
        LitDesc::U16 => into_vec(|res|res.write_u16::<EncodingByteOrder>(input.parse::<u16>().unwrap()).unwrap()),
        LitDesc::I16 => into_vec(|res|res.write_i16::<EncodingByteOrder>(input.parse::<i16>().unwrap()).unwrap()),
        LitDesc::U32 => into_vec(|res|res.write_u32::<EncodingByteOrder>(input.parse::<u32>().unwrap()).unwrap()),
        LitDesc::I32 => into_vec(|res|res.write_i32::<EncodingByteOrder>(input.parse::<i32>().unwrap()).unwrap()),
        LitDesc::U64 => into_vec(|res|res.write_u64::<EncodingByteOrder>(input.parse::<u64>().unwrap()).unwrap()),
        LitDesc::I64 => into_vec(|res|res.write_i64::<EncodingByteOrder>(input.parse::<i64>().unwrap()).unwrap()),
        LitDesc::U128 => into_vec(|res|res.write_u128::<EncodingByteOrder>(input.parse::<u128>().unwrap()).unwrap()),
        LitDesc::I128 => into_vec(|res|res.write_i128::<EncodingByteOrder>(input.parse::<i128>().unwrap()).unwrap()),
        LitDesc::Data => decode(&input[2..]).unwrap(),
        LitDesc::Id => decode(input).unwrap(),
    }
}

pub fn parse_lit(input:&str, size:u16) -> Vec<u8>{
    if input.starts_with("0x") {
        decode(&input[2..(((size as usize)*2)+2)]).expect(input)
    } else if input.starts_with("-") {
        match size {
            1 => into_vec(|res|res.write_i8(input.parse::<i8>().expect(input)).expect(input)),
            2 => into_vec(|res|res.write_i16::<EncodingByteOrder>(input.parse::<i16>().expect(input)).expect(input)),
            4 => into_vec(|res|res.write_i32::<EncodingByteOrder>(input.parse::<i32>().expect(input)).expect(input)),
            8 => into_vec(|res|res.write_i64::<EncodingByteOrder>(input.parse::<i64>().expect(input)).expect(input)),
            16 => into_vec(|res|res.write_i128::<EncodingByteOrder>(input.parse::<i128>().expect(input)).expect(input)),
            _ => panic!()
        }
    } else {
        match size {
            1 => into_vec(|res|res.write_u8(input.parse::<u8>().expect(input)).expect(input)),
            2 => into_vec(|res|res.write_u16::<EncodingByteOrder>(input.parse::<u16>().expect(input)).expect(input)),
            4 => into_vec(|res|res.write_u32::<EncodingByteOrder>(input.parse::<u32>().expect(input)).expect(input)),
            8 => into_vec(|res|res.write_u64::<EncodingByteOrder>(input.parse::<u64>().expect(input)).expect(input)),
            16 => into_vec(|res|res.write_u128::<EncodingByteOrder>(input.parse::<u128>().expect(input)).expect(input)),
            _ => panic!()
        }
    }
}


impl OpCode {

    pub fn compile<A: ParserAllocator>(&self, env:&mut Environment<A>, imp:&mut CodeImportBuilder) -> Result<ROpCode, String> {
        match *self {

            OpCode::Lit(ref id,ref lit, ref typ) => {
                let id = Id(id.0.to_lowercase());
                let size = imp.imported_lit_size(typ)?;
                let num:LargeVec<u8> = LargeVec(parse_lit(&lit.0,size));
                env.push_new(id.clone());
                Ok(ROpCode::Lit(num,imp.import_perm_ref(&(Perm::LitCreate,typ.clone()))?))
            },

            OpCode::Let(ref ids, ref block) => {
                let exp = block.compile(env,imp)?;
                for id in ids {
                    env.push_new(id.clone())
                }
                Ok(ROpCode::Let(exp))
            },

            OpCode::Copy(ref to_id, ref from_id) => {
                let from_v = env.get_id_pos(&from_id).unwrap();
                env.push_new(to_id.clone());
                Ok(ROpCode::Copy(from_v))
            },

            OpCode::Return(ref assigs, ref vals) => {
                let inputs = vals.iter().map(|id|env.get_id_pos(id).unwrap()).collect();
                for assig in assigs {
                    env.push_new(assig.clone());
                }
                Ok(ROpCode::Return(inputs))
            },

            OpCode::Project(ref to_id, ref from_id, ref typ) => {
                let from_v = env.get_id_pos(&from_id).unwrap();
                let t = imp.import_typ_ref(&typ)?;
                env.push_new(to_id.clone());
                Ok(ROpCode::Project(t, from_v))
            },

            OpCode::UnProject(ref to_id, ref from_id, ref typ) => {
                let from_v = env.get_id_pos(&from_id).unwrap();
                let t = imp.import_typ_ref(&typ)?;
                env.push_new(to_id.clone());
                Ok(ROpCode::UnProject(t, from_v))
            },

            OpCode::Fetch(ref to_id, ref from_id) => {
                let from_v = env.get_id_pos(&from_id).unwrap();
                env.push_new(to_id.clone());
                Ok(ROpCode::Move(from_v))
            },

            OpCode::Field(ref to_id, ref from_id, ref field_no, ref typ) => {
                let no = field_no.0.parse::<u8>().unwrap();
                let from_v = env.get_id_pos(&from_id).unwrap();
                let t = imp.import_perm_ref(&(Perm::Consume,typ.clone()))?;
                env.push_new(to_id.clone());
                Ok(ROpCode::Field(from_v,t,no))
            },

            OpCode::CopyField(ref to_id, ref from_id, ref field_no, ref typ) => {
                let no = field_no.0.parse::<u8>().unwrap();
                let from_v = env.get_id_pos(&from_id).unwrap();
                let t =imp.import_perm_ref(&(Perm::Inspect,typ.clone()))?;
                env.push_new(to_id.clone());
                Ok(ROpCode::CopyField(from_v,t,no))
            },

            OpCode::Discard(ref val) => {
                let from_v = env.get_id_pos(&val).unwrap();
                Ok(ROpCode::Discard(from_v))
            },
            OpCode::DiscardMany(ref vals) => {
                let inputs = vals.iter().map(|id|env.get_id_pos(id).unwrap()).collect();
                Ok(ROpCode::DiscardMany(inputs))
            },

            OpCode::Unpack(ref assigs, ref val, ref typ) => {
                let from_v = env.get_id_pos(&val).unwrap();
                let r_typ = imp.import_perm_ref(&(Perm::Consume,typ.clone()))?;
                for assig in assigs {
                    env.push_new(assig.clone());
                }
                Ok(ROpCode::Unpack(from_v, r_typ))
            },

            OpCode::Switch(ref assigs, ref val, ref typ, ref matches) => {
                let from_v = env.get_id_pos(&val).unwrap();
                let r_typ = imp.import_perm_ref(&(Perm::Consume,typ.clone()))?;
                let ord = imp.get_ctr_order(&typ)?;
                let mut exprs = Vec::with_capacity(ord.len());
                for o in ord {
                    if let Some(m) = matches.iter().find(|m|m.ctr == o) {
                        exprs.push(m.code.compile_with_input(&m.params, env, imp)?);
                    }
                }
                for assig in assigs {
                    env.push_new(assig.clone());
                }
                Ok(ROpCode::Switch(from_v, r_typ, exprs))
            },

            OpCode::Inspect(ref assigs, ref val, ref typ, ref matches) => {
                let from_v = env.get_id_pos(&val).unwrap();
                let r_typ = imp.import_perm_ref(&(Perm::Inspect,typ.clone()))?;
                let ord = imp.get_ctr_order(&typ)?;
                let mut exprs = Vec::with_capacity(ord.len());
                for o in ord {
                    if let Some(m) = matches.iter().find(|m|m.ctr == o) {
                        exprs.push(m.code.compile_with_input(&m.params, env, imp)?);
                    }
                }
                for assig in assigs {
                    env.push_new(assig.clone());
                }
                Ok(ROpCode::Inspect(from_v, r_typ, exprs))
            },

            OpCode::Pack(ref res_id, ref typ, ref ctr_id, ref vals) => {
                let inputs = vals.iter().map(|id|env.get_id_pos(id).unwrap()).collect();
                let r_typ = imp.import_perm_ref(&(Perm::DataCreate,typ.clone()))?;
                let ord = imp.get_ctr_order(&typ)?;
                let pos = match ord.iter().position(|id|*id==*ctr_id){
                    Some(v) => v,
                    None => 0,
                };
                assert!(pos < u8::max_value() as usize);
                env.push_new(res_id.clone());
                Ok(ROpCode::Pack(r_typ, Tag(pos as u8), inputs))
            },

            OpCode::CreateSig(ref assig, ref sig, ref params) => {
                let inputs = params.iter().map(|id|env.get_id_pos(id).unwrap()).collect();
                let i_ref = imp.import_impl_ref(&sig.main, sig.applies.clone())?;
                let p_ref = imp.import_call_ref(i_ref)?;
                env.push_new(assig.clone());
                Ok(ROpCode::Invoke(p_ref, inputs))
            }

            OpCode::CallSig(ref assigs, ref target, ref sig, ref params) => {
                let inputs = params.iter().map(|id|env.get_id_pos(id).unwrap()).collect();
                let s_typ = imp.import_perm_ref(&(Perm::Call,sig.clone()))?;
                let target_v = env.get_id_pos(&target).unwrap();

                for assig in assigs {
                    env.push_new(assig.clone());
                }
                Ok(ROpCode::InvokeSig(target_v, s_typ, inputs))
            }

            OpCode::Call(ref assigs, ref fun, ref applies, ref params) => {
                let inputs = params.iter().map(|id|env.get_id_pos(id).unwrap()).collect();
                let f_ref = imp.import_fun_ref(&fun, applies.clone())?;
                let p_ref = imp.import_call_ref(f_ref)?;
                for assig in assigs {
                    env.push_new(assig.clone());
                }
                Ok(ROpCode::Invoke(p_ref, inputs))
            },

            OpCode::TryCall(ref assigs, ref fun, ref applies, ref params, ref branches) => {
                let inputs = params.iter().map(|(fail_discard, id)|(*fail_discard, env.get_id_pos(id).unwrap())).collect();
                let f_ref = imp.import_fun_ref(&fun, applies.clone())?;
                let p_ref = imp.import_call_ref(f_ref)?;
                for assig in assigs {
                    env.push_new(assig.clone());
                }
                let success = branches.iter().find(|m|m.ctr.0.to_lowercase() == "success").map(|m|m.code.compile_with_input(&m.params, env, imp).unwrap()).unwrap();
                let fail = branches.iter().find(|m|m.ctr.0.to_lowercase() == "failure").map(|m|m.code.compile_with_input(&m.params, env, imp).unwrap()).unwrap();

                Ok(ROpCode::TryInvoke(p_ref, inputs, success, fail))
            },

            OpCode::Abort(ref assigs, ref params, ref produces) => {
                let inputs = params.iter().map(|id|env.get_id_pos(id).unwrap()).collect();
                let t_refs = produces.iter().map(|typ|imp.import_typ_ref(typ).unwrap()).collect();
                for assig in assigs {
                    env.push_new(assig.clone());
                }
                Ok(ROpCode::RollBack(inputs, t_refs))
            }
        }
    }
}

impl Visibility {
    pub fn compile(&self, generics: &[Generic]) -> RVisibility {
        match *self {
            Visibility::Private => RVisibility::Local,
            Visibility::Public => RVisibility::Global,
            Visibility::Protected(ref ids) => {
                let g_refs = ids.iter().map(|id| {
                    let pos = generics.iter().position(|g| g.name == *id).unwrap();
                    assert!(pos <= u8::max_value() as usize);
                    GenRef(pos as u8)
                }).collect();
                RVisibility::Guarded(g_refs)
            }
        }
    }
}

impl Generic {
    pub fn compile(&self) -> RGeneric {
        if self.phantom {
            RGeneric::Phantom
        } else {
            RGeneric::Physical(self.caps)
        }
    }
}

impl Var {
    pub fn compile<A: ParserAllocator>(&self, env: &mut Environment<A>, imp: &mut CodeImportBuilder) -> Result<Param,String> {
        let typ = imp.import_typ_ref(&self.typ)?;
        env.push_new(self.name.clone());
        Ok(Param{
            consumes: self.consume,
            typ
        })
    }
}

impl Ret {
    pub fn compile<A: ParserAllocator>(&self, env: &mut Environment<A>, imp: &mut CodeImportBuilder) -> Result<TypeRef,String> {
        let typ = imp.import_typ_ref(&self.typ)?;
        env.push_new(self.name.clone());
        Ok(typ)
    }
}

impl Case {
    pub fn compile(&self, imp: &mut CodeImportBuilder) -> Result<RCase,String> {
        let fields = self.params.iter().map(|f|imp.import_typ_ref(f)).collect::<Result<_,_>>()?;
        Ok(RCase { fields })
    }
}


pub fn gen_lit_desc(typ:&Type) -> LitDesc {
    match &typ.main {
        Ref::Module(_,ref id) => match id.0.as_ref() {
            "U8" => LitDesc::U8,
            "I8" => LitDesc::I8,
            "U16" => LitDesc::U16,
            "I16" => LitDesc::I16,
            "U32" => LitDesc::U32,
            "I32" => LitDesc::I32,
            "U64" => LitDesc::U64,
            "I64" => LitDesc::I64,
            "U128" => LitDesc::U128,
            "I128" => LitDesc::I128,
            "Data"  => LitDesc::Data,
            "PublicId" => LitDesc::Id,
            _ => panic!()
        },
        _ => panic!(),
    }
}