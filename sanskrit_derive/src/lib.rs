
extern crate syn;
extern crate quote;
extern crate proc_macro;
extern crate proc_macro2;


use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};
use syn::Field;
use proc_macro2::Ident;
use proc_macro2::Span;
use syn::Generics;
use quote::ToTokens;

//Todo: Attributed Enums seem Wrong

#[proc_macro_derive(Serializable, attributes(ByteSize, Transient, VirtualSize, StartIndex))]
pub fn serialize_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust sys as a syntax tree
    // that we can manipulate
    let ast:DeriveInput  = syn::parse(input).unwrap();

    // Build the trait implementation
    //println!("{}", impl_serialize_macro(&ast));
    impl_serialize_macro(&ast).into()
}

#[proc_macro_derive(AllocParsable, attributes(ByteSize, Transient, VirtualSize, StartIndex))]
pub fn alloc_parse_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_derive_core(input, true)
}

#[proc_macro_derive(Parsable, attributes(AllocLifetime, ByteSize, Transient, VirtualSize, StartIndex))]
pub fn parse_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_derive_core(input, false)
}

fn parse_derive_core(input: proc_macro::TokenStream, auto_alloc:bool) -> proc_macro::TokenStream {
    // Construct a representation of Rust sys as a syntax tree
    // that we can manipulate
    let ast:DeriveInput  = syn::parse(input).unwrap();
    // Build the trait implementation
    //println!("{:?}",impl_parsable_macro(&ast));
    impl_parsable_macro(&ast, auto_alloc).into()
}


#[proc_macro_derive(VirtualSize)]
pub fn virtual_size_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust sys as a syntax tree
    // that we can manipulate
    let ast:DeriveInput  = syn::parse(input).unwrap();

    // Build the trait implementation
    //println!("{}",impl_virtual_size_macro(&ast));
     impl_virtual_size_macro(&ast).into()
}


fn impl_serialize_macro(ast:&DeriveInput) -> TokenStream {
    let prefix = &ast.ident;
    let body = match ast.data {
        Data::Struct(ref ds) => {
            let size_field = extract_named_field("ByteSize",ds.fields.iter());
            let virt_field = extract_named_field("VirtualSize",ds.fields.iter());
            let transient_fields = extract_named_fields("Transient",ds.fields.iter());
            let start_field = extract_named_field("StartIndex",ds.fields.iter());

            let mut body = Vec::new();
            for (idx,f) in ds.fields.iter().filter(|f|
                (size_field.is_none() || f.ident != size_field)
                    && (virt_field.is_none() || f.ident != virt_field)
                    && (f.ident.is_none() || !transient_fields.contains(&f.ident.as_ref().unwrap()))
                    && (start_field.is_none() || f.ident != start_field)
            ).enumerate() {
                body.push(match &f.ident {
                    None => {
                        let idx = syn::Index::from(idx);
                        quote!{self.#idx.serialize(s)?}
                    },
                    Some(id) => quote!{self.#id.serialize(s)?},
                })
            }
            quote!{#(#body;)*}
        }
        Data::Enum(ref ed) => {
            let mut cases = Vec::new();
            for (idx,v) in ed.variants.iter().enumerate() {
                let mut fields = Vec::new();
                let mut named = false;
                for (num,f) in v.fields.iter().enumerate() {
                    fields.push(match f.ident {
                        None => {
                            let id = Ident::new(&format!("_{}",num), Span::call_site());
                            quote!{ref #id}
                        },
                        Some(ref id) => {
                            named = true;
                            quote!{ref #id}
                        },
                    })
                }
                let name = &v.ident;
                let num = idx as u8;
                if v.fields.iter().len() == 0 {
                    cases.push(quote!{#prefix::#name => s.produce_byte(#num)})
                } else {
                    let size_field = extract_named_field("ByteSize",v.fields.iter());
                    let virt_field = extract_named_field("VirtualSize",v.fields.iter());
                    let transient_fields = extract_named_fields("Transient",v.fields.iter());
                    let start_field = extract_named_field("StartIndex",v.fields.iter());

                    let body = pattern_body_serialize(v.fields.iter(),size_field,transient_fields,start_field,virt_field);
                    if named {
                        cases.push(quote!{#prefix::#name{#(#fields),*} => {
                            s.produce_byte(#num);
                            s.increment_depth()?;
                            #body
                            s.decrement_depth();
                        }})
                    } else {
                        cases.push(quote!{#prefix::#name(#(#fields),*) => {
                            s.produce_byte(#num);
                            s.increment_depth()?;
                            #body
                            s.decrement_depth();
                        }})
                    }
                }
            }
            quote!{
                match *self {
                    #(#cases),*
                }
            }
        },
        Data::Union(_) => unimplemented!(),
    };

    let generics = &ast.generics;
    let plain = extract_plain_generics(generics);
    let argumented = extract_agumented_generics(generics, quote!{Serializable});

    quote!{
        impl<#(#argumented),*> Serializable for #prefix<#(#plain),*> {
             fn serialize(&self, s:&mut Serializer) -> Result<()> {
                #body
                Ok(())
             }
        }
    }
}

fn impl_parsable_macro(ast:&DeriveInput, auto_alloc:bool) -> TokenStream {
    let prefix = &ast.ident;
    let body = match ast.data {
        Data::Struct(ref ds) => {
            let size_field = extract_named_field("ByteSize",ds.fields.iter());
            let virt_field = extract_named_field("VirtualSize",ds.fields.iter());
            let transient_fields = extract_named_fields("Transient",ds.fields.iter());
            let start_field = extract_named_field("StartIndex",ds.fields.iter());

            let pos_fetch = match (&size_field,&start_field) {
                (None,None) => quote!{},
                _ => quote!{let start = p.index;},
            };

            let virt_pos_fetch = match &virt_field {
                None => quote!{},
                _ => quote!{let virt_start = alloc.allocated_virtual_bytes();},
            };

            let mut named = false;
            let mut body:Vec<_> = ds.fields.iter()
                .filter(|f|
                    (size_field.is_none() || f.ident != size_field)
                        && (f.ident.is_none() || !transient_fields.contains(&f.ident.as_ref().unwrap()))
                        && (start_field.is_none() || f.ident != start_field)
                        && (virt_field.is_none() || f.ident != virt_field)
                ).map(|f|{
                    match &f.ident {
                        None => quote!{Parsable::parse(p,alloc)?},
                        Some(id) => {
                            named = true;
                            quote!{#id:Parsable::parse(p,alloc)?}
                        },
                    }
                }).collect();

            match size_field {
                None => {},
                Some(id) => {
                    body.push(quote!{#id:Option::Some(p.index-start)})
                },
            };

            for id in transient_fields {
                body.push(quote!{#id:Option::None})
            };

            match start_field {
                None => {},
                Some(id) => {
                    body.push(quote!{#id:Option::Some(start)})
                },
            };

            match virt_field {
                None => {},
                Some(id) => {
                    body.push(quote!{#id:Option::Some((alloc.allocated_virtual_bytes()-virt_start)+#prefix::SIZE)})
                },
            };

            let fields = body.len();
            let build = if named {
                quote!{ Ok(#prefix{#(#body),*}) }
            } else {
                quote!{ Ok(#prefix(#(#body),*)) }
            };

            if fields == 0 {
                quote!{
                    #pos_fetch
                    #virt_pos_fetch
                    #build
                }
            } else {
                quote!{
                    p.increment_depth()?;
                    #pos_fetch
                    #virt_pos_fetch
                    let res = #build;
                    p.decrement_depth();
                    res
                }
            }

        }
        Data::Enum(ref ed) => {
            let mut cases = Vec::new();
            for (idx,v) in ed.variants.iter().enumerate() {
                let name = &v.ident;
                let num = idx as u8;
                if v.fields.iter().len() == 0 {
                    cases.push(quote!{#num => #prefix::#name})
                } else {
                    let size_field = extract_named_field("ByteSize",v.fields.iter());
                    let virt_field = extract_named_field("VirtualSize",v.fields.iter());
                    let transient_fields = extract_named_fields("Transient",v.fields.iter());
                    let start_field = extract_named_field("StartIndex",v.fields.iter());

                    let pos_fetch = match (&size_field,&start_field) {
                        (None,None) => quote!{},
                        _ => quote!{let start = p.index;},
                    };

                    let virt_pos_fetch = match &virt_field {
                        None => quote!{},
                        _ => quote!{let virt_start = alloc.allocated_virtual_bytes();},
                    };

                    let body = pattern_body_parse(v.fields.iter(),size_field, transient_fields, start_field,virt_field, prefix);
                    cases.push(quote!{#num => {
                            #pos_fetch
                            #virt_pos_fetch
                            #prefix::#name #body
                        }
                    })
                }
            }
            quote!{
                Ok(match p.consume_byte()? {
                    #(#cases,)*
                    x => return error(||"Can not parse unknown enum variant") //panic!("{:?} in {:?}",x, stringify!(#prefix))
                })
            }
        },
        Data::Union(_) => unimplemented!()
    };
    let generics = &ast.generics;
    let alloc_id = extract_alloc_lifetime(generics, auto_alloc);
    let plain = extract_plain_generics(&ast.generics);
    match alloc_id {
        None => {
            let argumented = extract_agumented_generics(&ast.generics, quote!{Parsable<'p>});
            quote!{
                impl<'p, #(#argumented),*> Parsable<'p> for #prefix<#(#plain),*> {
                     fn parse<A:ParserAllocator>(p: &mut Parser, alloc:&'p A) -> Result<Self> {
                        #body
                     }
                }
            }
        },
        Some(id) => {
            let argumented = extract_agumented_generics(&ast.generics, quote!{Parsable<#id>});
            quote!{
                impl<#(#argumented),*> Parsable<#id> for #prefix<#(#plain),*> {
                     fn parse<A:ParserAllocator>(p: &mut Parser, alloc:&#id A) -> Result<Self> {
                        #body
                     }
                }
            }
        }
    }
}

fn impl_virtual_size_macro(ast:&DeriveInput) -> TokenStream {
    let prefix = &ast.ident;
    fn expr<'a>(fields: impl Iterator<Item=&'a Field>) -> TokenStream {
        let parts:Vec<_> = fields.map(|f|{
            let typ = &f.ty;
            quote!{TypeId::<#typ>::SIZE}
        }).collect();
        if parts.is_empty() {
            quote!{0}
        } else {
            quote!{#(#parts)+*}
        }
    }

    fn max_all(variants:&mut Vec<TokenStream>) -> TokenStream {
        match variants.pop() {
            None => quote!{0},
            Some(var) => {
                let rest = max_all(variants);
                quote!{max(#var,#rest)}
            },
        }
    }

    let body = match ast.data {
        Data::Struct(ref ds) => expr(ds.fields.iter()),
        Data::Enum(ref ed) => {
            let res = max_all(&mut ed.variants.iter().map(|v| expr(v.fields.iter())).collect());
            quote!{1+#res} //1Byte for the tag
        },
        Data::Union(ref us) => max_all(&mut us.fields.named.iter().map(|f| {
            let typ = &f.ty;
            quote!{TypeId::<#typ>::SIZE}
        }).collect()),
    };

    let generics = &ast.generics;
    let plain = extract_plain_generics(generics);
    let argumented = extract_agumented_generics(generics, quote!{VirtualSize});
    quote!{
        impl<#(#argumented),*> VirtualSize for #prefix<#(#plain),*> {
             const SIZE:usize = #body;
        }
    }
}


fn pattern_body_serialize<'a>(fs: impl Iterator<Item=&'a Field>, size_field:Option<Ident>, transient_fields:Vec<Ident>, start_field:Option<Ident>, virt_field:Option<Ident>) -> TokenStream {
    let mut body = Vec::new();
    for (idx,f) in fs.filter(|f|
        (size_field.is_none() || f.ident != size_field)
            && (virt_field.is_none() || f.ident != virt_field)
            && (f.ident.is_none() || !transient_fields.contains(&f.ident.as_ref().unwrap()))
            && (start_field.is_none() || f.ident != start_field)
    ).enumerate() {
        body.push(match &f.ident {
            None => {
                let id = Ident::new(&format!("_{}",idx), Span::call_site());
                quote!{#id.serialize(s)?}
            },
            Some(id) => quote!{#id.serialize(s)?},
        });
    }
    quote!{#(#body;)*}
}

fn pattern_body_parse<'a>(fs: impl Iterator<Item=&'a Field>, size_field:Option<Ident>, transient_fields:Vec<Ident>, start_field:Option<Ident>, virt_field:Option<Ident>, prefix:&Ident) -> TokenStream {
    let mut named = false;
    let mut body = Vec::new();
    for f in fs.filter(|f|
        (size_field.is_none() || f.ident != size_field)
            && (f.ident.is_none() || !transient_fields.contains(&f.ident.as_ref().unwrap()))
            && (start_field.is_none() || f.ident != start_field)
            && (virt_field.is_none() || f.ident != virt_field)
    ) {
        body.push(match &f.ident {
            None => quote!{Parsable::parse(p,alloc)?},
            Some(id) => {
                named = true;
                quote!{#id:Parsable::parse(p,alloc)?}
            },
        })
    }
    match size_field {
        None => {},
        Some(id) => {
            body.push(quote!{#id:Option::Some(p.index-start)})
        },
    };

    for id in transient_fields {
        body.push(quote!{#id:Option::None})
    };

    match start_field {
        None => {},
        Some(id) => {
            body.push(quote!{#id:Option::Some(start)})
        },
    };

    match virt_field {
        None => {},
        Some(id) => {
            body.push(quote!{#id:Option::Some((alloc.allocated_virtual_bytes()-virt_start)+#prefix::SIZE)})
        },
    };

    if named {
        quote!{{#(#body),*}}
    } else {
        quote!{(#(#body),*)}
    }
}

fn extract_plain_generics(generics:&Generics) -> Vec<TokenStream> {
    let mut gens = Vec::new();
    gens.extend(generics.lifetimes().map(|f| f.clone().lifetime.into_token_stream()));
    gens.extend(generics.type_params().map(|t| t.clone().ident.into_token_stream()));
    gens
}

fn extract_agumented_generics(generics:&Generics, argument:TokenStream) -> Vec<TokenStream> {
    let mut gens = Vec::new();
    gens.extend(generics.lifetimes().map(|f| {
        let ident = f.clone().lifetime.into_token_stream();
        let bounds:Vec<_> = f.bounds.iter().collect();
        if bounds.is_empty() {
            quote!{#ident}
        } else {
            quote!{#ident:#(#bounds)+*}
        }
    }));
    gens.extend(generics.type_params().map(|t| {
        let ident = &t.ident;
        let bounds:Vec<_> = t.bounds.iter().collect();
        if bounds.is_empty() {
            quote!{#ident:#argument}
        } else {
            quote!{#ident:#argument+#(#bounds)+*}
        }
    }));
    gens
}

fn extract_alloc_lifetime(generics:&Generics, auto_alloc:bool) -> Option<TokenStream> {
    if auto_alloc {
        generics.lifetimes().nth(0).map(|l|l.clone().lifetime.into_token_stream())
    } else {
        for l in generics.lifetimes() {
            for a in &l.attrs {
                match a.path().segments.last() {
                    None => {}
                    Some(p)  => {
                        if p.ident == "AllocLifetime" {
                            return Some(l.clone().lifetime.into_token_stream())
                        }
                    }
                }
            }
        }
        None
    }
}

fn extract_named_field<'a>(name:&str, fs: impl Iterator<Item=&'a Field>) -> Option<Ident> {
    for f in fs {
        for a in &f.attrs {
            match a.path().segments.last() {
                None => {}
                Some(p)  => {
                    if p.ident == name {
                        return f.ident.clone();
                    }
                }
            }
        }
    }
    None
}

fn extract_named_fields<'a>(name:&str, fs: impl Iterator<Item=&'a Field>) -> Vec<Ident> {
    let mut fields:Vec<Ident> = Vec::new();
    for f in fs {
        for a in &f.attrs {
            match a.path().segments.last() {
                None => {}
                Some(p)  => {
                    if p.ident == name {
                        match f.ident {
                            Some(ref ident) =>fields.push(ident.clone()),
                            None => {}
                        }
                    }
                }
            }
        }
    }
    fields
}
