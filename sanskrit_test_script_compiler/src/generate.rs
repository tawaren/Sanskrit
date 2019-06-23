use model::*;
use environment::*;
use std::collections::HashMap;
use sanskrit_runtime::model::ScriptCode as RScriptCode;
use sanskrit_core::model::OpCode as ROpCode;
use sanskrit_core::model::Visibility as RVisibility;
use sanskrit_core::model::Ret as RRet;
use sanskrit_core::model::Case as RCase;
use sanskrit_core::model::Generic as RGeneric;
use sanskrit_core::model::Exp;
use sanskrit_common::model::LargeVec;
use native::*;
use sanskrit_common::model::Tag;
use sanskrit_core::model::GenRef;
use sanskrit_core::model::Param;
use sanskrit_common::encoding::ParserAllocator;
use sanskrit_common::model::Ptr;
use sanskrit_common::model::ValueRef;
use hex::decode;
use byteorder::{WriteBytesExt};
use sanskrit_common::encoding::EncodingByteOrder;
use sanskrit_interpreter::model::LitDesc;


impl Block {
    pub fn compile_with_input<A: ParserAllocator>(&self, inputs:&[Id], env:&mut Environment<A>, imp:&mut CodeImportBuilder) -> Result<Exp,String> {
        match *self {
            Block::Error(ref err) => {
                let err_ref = imp.import_err_ref(err)?;
                Ok(Exp::Throw(err_ref))
            },
            Block::Return(ref codes, ref rets/*, ref drops*/) => {
                let reg = Region(HashMap::new());
                let height = env.stack_depth;
                env.frames.push(reg);
                for input in inputs {
                    env.push_new(input.clone())
                }
                let op_codes = LargeVec(codes.iter().map(|c|c.compile(env, imp)).collect::<Result<_,_>>()?);
                let ref_vals = rets.iter().map(|id|env.get_id_pos(id).unwrap()).collect();
                //let drop_vals = drops.iter().map(|id|env.get_id_pos(id).unwrap()).collect();
                env.frames.pop();
                env.stack_depth = height;
                Ok(Exp::Ret(op_codes,ref_vals/*, drop_vals*/))
            },
        }
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
        decode(&input[2..(((size as usize)*2)+2)]).unwrap()
    } else if input.starts_with("-") {
        match size {
            1 => into_vec(|res|res.write_i8(input.parse::<i8>().unwrap()).unwrap()),
            2 => into_vec(|res|res.write_i16::<EncodingByteOrder>(input.parse::<i16>().unwrap()).unwrap()),
            4 => into_vec(|res|res.write_i32::<EncodingByteOrder>(input.parse::<i32>().unwrap()).unwrap()),
            8 => into_vec(|res|res.write_i64::<EncodingByteOrder>(input.parse::<i64>().unwrap()).unwrap()),
            16 => into_vec(|res|res.write_i128::<EncodingByteOrder>(input.parse::<i128>().unwrap()).unwrap()),
            _ => panic!()
        }
    } else {
        match size {
            1 => into_vec(|res|res.write_u8(input.parse::<u8>().unwrap()).unwrap()),
            2 => into_vec(|res|res.write_u16::<EncodingByteOrder>(input.parse::<u16>().unwrap()).unwrap()),
            4 => into_vec(|res|res.write_u32::<EncodingByteOrder>(input.parse::<u32>().unwrap()).unwrap()),
            8 => into_vec(|res|res.write_u64::<EncodingByteOrder>(input.parse::<u64>().unwrap()).unwrap()),
            16 => into_vec(|res|res.write_u128::<EncodingByteOrder>(input.parse::<u128>().unwrap()).unwrap()),
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
                Ok(ROpCode::Lit(num,imp.import_typ_ref(&typ)?))
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
                Ok(ROpCode::CopyFetch(from_v))
            },

            OpCode::Image(ref to_id, ref from_id) => {
                let from_v = env.get_id_pos(&from_id).unwrap();
                env.push_new(to_id.clone());
                Ok(ROpCode::Image(from_v))
            },

            OpCode::Fetch(ref to_id, ref from_id, borrow) => {
                println!("{}", from_id.0);
                let from_v = env.get_id_pos(&from_id).unwrap();
                env.push_new(to_id.clone());
                Ok(if borrow {
                    ROpCode::BorrowFetch(from_v)
                } else {
                    ROpCode::Fetch(from_v)
                })
            },

            OpCode::Field(ref to_id, ref from_id, ref field_no, ref typ, borrow) => {
                let no = field_no.0.parse::<u8>().unwrap();
                let from_v = env.get_id_pos(&from_id).unwrap();
                let t = imp.import_typ_ref(&typ)?;
                env.push_new(to_id.clone());
                Ok(if borrow {
                    ROpCode::BorrowField(from_v,t,no)
                } else {
                    ROpCode::Field(from_v,t,no)
                })
            },

            OpCode::CopyField(ref to_id, ref from_id, ref field_no, ref typ) => {
                let no = field_no.0.parse::<u8>().unwrap();
                let from_v = env.get_id_pos(&from_id).unwrap();
                let t = imp.import_typ_ref(&typ)?;
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
            OpCode::DiscardBorrowed(ref val, ref borrows) => {
                let borrows_v = borrows.iter().map(|id|env.get_id_pos(id).unwrap()).collect();
                let from_v = env.get_id_pos(&val).unwrap();
                Ok(ROpCode::DiscardBorrowed(from_v, borrows_v))
            },

            OpCode::Unpack(ref assigs, ref val, ref typ, borrow) => {
                let from_v = env.get_id_pos(&val).unwrap();
                let r_typ = imp.import_typ_ref(&typ)?;
                for assig in assigs {
                    env.push_new(assig.clone());
                }
                Ok(if borrow {
                    ROpCode::BorrowUnpack(from_v, r_typ)
                } else {
                    ROpCode::Unpack(from_v, r_typ)
                })
            },

            OpCode::Switch(ref assigs, ref val, ref typ, ref matches, borrow) => {
                let from_v = env.get_id_pos(&val).unwrap();
                let r_typ = imp.import_typ_ref(&typ)?;
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
                Ok(if borrow {
                    ROpCode::BorrowSwitch(from_v, r_typ, exprs)
                } else {
                    ROpCode::Switch(from_v, r_typ, exprs)
                })
            },

            OpCode::Pack(ref res_id, ref typ, ref ctr_id, ref vals, borrow) => {
                let inputs = vals.iter().map(|id|env.get_id_pos(id).unwrap()).collect();
                let r_typ = imp.import_typ_ref(&typ)?;
                let ord = imp.get_ctr_order(&typ)?;
                let pos = match ord.iter().position(|id|*id==*ctr_id){
                    Some(v) => v,
                    None => 0,
                };
                assert!(pos < u8::max_value() as usize);
                env.push_new(res_id.clone());
                Ok(if borrow {
                    ROpCode::BorrowPack(r_typ, Tag(pos as u8), inputs)
                } else {
                    ROpCode::Pack(r_typ, Tag(pos as u8), inputs)
                })
            },

            OpCode::Call(ref assigs, ref fun, ref applies, ref params) => {
                let inputs = params.iter().map(|id|env.get_id_pos(id).unwrap()).collect();
                let f_ref = imp.import_fun_ref(&fun, applies.clone())?;
                for assig in assigs {
                    env.push_new(assig.clone());
                }
                Ok(ROpCode::Invoke(f_ref, inputs))
            },

            OpCode::Try(ref assigs, ref block, ref catches) => {
                let try = block.compile(env, imp)?;
                let mut c_exprs = Vec::with_capacity(catches.len());
                for catch in catches {
                    let err_ref = imp.import_err_ref(&catch.error)?;
                    c_exprs.push((err_ref,catch.code.compile(env, imp)?));
                }
                for assig in assigs {
                    env.push_new(assig.clone());
                }
                Ok(ROpCode::Try(try,c_exprs))
            },

            /*OpCode::ModuleIndex(ref id) => {
                env.push_new(id.clone());
                Ok(ROpCode::ModuleIndex)
            },*/
        }
    }
}

impl Visibility {
    pub fn compile(&self, generics: &[Generic]) -> RVisibility {
        match *self {
            Visibility::Private => RVisibility::Private,
            Visibility::Public => RVisibility::Public,
            Visibility::Protected(ref ids) => {
                let g_refs = ids.iter().map(|id| {
                    let pos = generics.iter().position(|g| g.name == *id).unwrap();
                    assert!(pos <= u8::max_value() as usize);
                    GenRef(pos as u8)
                }).collect();
                RVisibility::Protected(g_refs)
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
    pub fn compile<A: ParserAllocator>(&self, env: &mut Environment<A>, imp: &mut CodeImportBuilder) -> Result<RRet,String> {
        let typ = imp.import_typ_ref(&self.typ)?;
        let borrows = self.borrow.iter().map(|b|env.get_id_pos(b).unwrap()).collect();
        env.push_new(self.name.clone());
        Ok(RRet { borrows, typ })
    }
}

impl Case {
    pub fn compile(&self, imp: &mut CodeImportBuilder) -> Result<RCase,String> {
        let fields = self.params.iter().map(|f|imp.import_typ_ref(f)).collect::<Result<_,_>>()?;
        Ok(RCase { fields })
    }
}


//todo: I Hate this but without changing the Scripting we need this hack
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

impl ScriptCode {
    pub fn compile<'a,'c, 'h, A: ParserAllocator>(&self, env:&mut Environment<'c, A>, imp:&mut ScriptContext<'a,'c,'h>) -> Result<Ptr<'c,RScriptCode<'c>>, String> {
        Ok(imp.alloc.alloc(match *self {
            ScriptCode::Lit(ref id,ref lit, ref typ) => {
                let desc = gen_lit_desc(&typ);
                let num = imp.alloc.copy_alloc_slice(&parse_lit_from_desc(&lit.0,desc))?;
                //todo: I Hate this but without changing the Scripting we need this hack
                let adt = imp.generate_adt_ref(&typ.main)?;
                env.push_new(id.clone());
                RScriptCode::Lit(num,desc,adt)
            },

            ScriptCode::Wit(ref id,ref lit, ref typ) => {
                let desc = gen_lit_desc(&typ);
                let num = imp.alloc.copy_alloc_slice(&parse_lit_from_desc(&lit.0,desc))?;
                env.push_new(id.clone());
                //todo: I Hate this but without changing the Scripting we need this hack
                let adt = imp.generate_adt_ref(&typ.main)?;
                let wit_ref = imp.generate_wit_ref(num);
                RScriptCode::Wit(wit_ref,desc,adt)
            },

            ScriptCode::Copy(ref to_id, ref from_id) => {
                let from_v = env.get_id_pos(&from_id).unwrap();
                env.push_new(to_id.clone());
                RScriptCode::Copy(from_v)
            }

            ScriptCode::Fetch(ref to_id, ref from_id, borrow) => {
                let from_v = env.get_id_pos(&from_id).unwrap();
                env.push_new(to_id.clone());
                if borrow {
                    RScriptCode::BorrowFetch(from_v)
                } else {
                    RScriptCode::Fetch(from_v)
                }
            },

            ScriptCode::Drop(ref val) => {
                let from_v = env.get_id_pos(&val).unwrap();
                RScriptCode::Drop(from_v)
            },

            ScriptCode::Free(ref val) => {
                let from_v = env.get_id_pos(&val).unwrap();
                RScriptCode::Free(from_v)
            },

            ScriptCode::Unpack(ref assigs, ref val, ref main, ref ctr, borrow) => {
                let from_v = env.get_id_pos(&val).unwrap();
                let adt_r = imp.generate_adt_ref(main)?;
                let tag = imp.get_tag(main, ctr.clone())?;
                for assig in assigs {
                    env.push_new(assig.clone());
                }
                if borrow {
                    RScriptCode::BorrowUnpack(adt_r, tag,from_v)
                } else {
                    RScriptCode::Unpack(adt_r, tag,from_v)
                }
            },

            ScriptCode::Pack(ref res_id, ref typ, ref ctr_id, ref vals, borrow) => {
                let inputs = imp.alloc.iter_alloc_slice(vals.iter().map(|id|env.get_id_pos(id).unwrap()))?;
                let adt_r = imp.generate_adt_ref(&typ.main)?;
                let tag = imp.get_tag(&typ.main, ctr_id.clone())?;
                let types = imp.alloc.iter_result_alloc_slice(typ.applies.iter().map(|t|imp.generate_type_ref(t)))?;
                env.push_new(res_id.clone());
                if borrow {
                    RScriptCode::BorrowPack(adt_r,types,tag, inputs)
                } else {
                    RScriptCode::Pack(adt_r,types,tag, inputs)
                }
            },

            ScriptCode::Call(ref assigs, ref fun, ref applies, ref params) => {
                let inputs = imp.alloc.iter_alloc_slice(params.iter().map(|id|env.get_id_pos(id).unwrap()))?;
                let f_ref = imp.generate_func_ref(&fun)?;
                let types = imp.alloc.iter_result_alloc_slice(applies.iter().map(|t|imp.generate_type_ref(t)))?;

                for assig in assigs {
                    env.push_new(assig.clone());
                }
                RScriptCode::Invoke(f_ref, types,inputs)
            },

            ScriptCode::Token(ref assig, ref param, borrow) => {
                let pos = imp.get_token_offset(param);
                let from_v = ValueRef((env.stack_depth + pos as usize) as u16 );
                env.push_new(assig.clone());
                if borrow {
                    RScriptCode::BorrowFetch(from_v)
                } else {
                    RScriptCode::Fetch(from_v)
                }
            },

            ScriptCode::Load(ref assig, ref val, borrow) => {
                let from_v = env.get_id_pos(val).unwrap();
                env.push_new(assig.clone());
                if borrow {
                    RScriptCode::BorrowLoad(from_v)
                } else {
                    RScriptCode::Load(from_v)
                }
            }
            ScriptCode::Store(ref val) => {
                let from_v = env.get_id_pos(val).unwrap();
                RScriptCode::Store(from_v)
            },

        }))
    }
}