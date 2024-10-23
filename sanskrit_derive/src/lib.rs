
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

#[proc_macro_derive(Serializable, attributes(ByteSize, Transient, StartIndex))]
pub fn serialize_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust sys as a syntax tree
    // that we can manipulate
    let ast:DeriveInput  = syn::parse(input).unwrap();

    // Build the trait implementation
    //println!("{}", impl_serialize_macro(&ast));
    impl_serialize_macro(&ast).into()
}

#[proc_macro_derive(Parsable, attributes(ByteSize, Transient, StartIndex))]
pub fn parse_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust sys as a syntax tree
    // that we can manipulate
    let ast:DeriveInput  = syn::parse(input).unwrap();
    // Build the trait implementation
    //println!("{:?}",impl_parsable_macro(&ast));
    impl_parsable_macro(&ast).into()
}

fn impl_serialize_macro(ast:&DeriveInput) -> TokenStream {
    let prefix = &ast.ident;
    let body = match ast.data {
        Data::Struct(ref ds) => {
            let size_field = extract_named_field("ByteSize",ds.fields.iter());
            let transient_fields = extract_named_fields("Transient",ds.fields.iter());
            let start_field = extract_named_field("StartIndex",ds.fields.iter());

            let mut body = Vec::new();
            for (idx,f) in ds.fields.iter().filter(|f|
                (size_field.is_none() || f.ident != size_field)
                    && (f.ident.is_none() || !transient_fields.contains(&f.ident.as_ref().unwrap()))
                    && (start_field.is_none() || f.ident != start_field)
            ).enumerate() {
                body.push(match &f.ident {
                    None => {
                        let idx = syn::Index::from(idx);
                        quote!{self.#idx.serialize(s)}
                    },
                    Some(id) => quote!{self.#id.serialize(s)},
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
                    let transient_fields = extract_named_fields("Transient",v.fields.iter());
                    let start_field = extract_named_field("StartIndex",v.fields.iter());

                    let body = pattern_body_serialize(v.fields.iter(),size_field,transient_fields,start_field);
                    if named {
                        cases.push(quote!{#prefix::#name{#(#fields),*} => {
                            s.produce_byte(#num);
                            #body
                        }})
                    } else {
                        cases.push(quote!{#prefix::#name(#(#fields),*) => {
                            s.produce_byte(#num);
                            #body
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
             fn serialize(&self, s:&mut Serializer) {
                #body
             }
        }
    }
}

fn impl_parsable_macro(ast:&DeriveInput) -> TokenStream {
    let prefix = &ast.ident;
    let body = match ast.data {
        Data::Struct(ref ds) => {
            let size_field = extract_named_field("ByteSize",ds.fields.iter());
            let transient_fields = extract_named_fields("Transient",ds.fields.iter());
            let start_field = extract_named_field("StartIndex",ds.fields.iter());

            let pos_fetch = match (&size_field,&start_field) {
                (None,None) => quote!{},
                _ => quote!{let start = p.index;},
            };

            let mut named = false;
            let mut body:Vec<_> = ds.fields.iter()
                .filter(|f|
                    (size_field.is_none() || f.ident != size_field)
                        && (f.ident.is_none() || !transient_fields.contains(&f.ident.as_ref().unwrap()))
                        && (start_field.is_none() || f.ident != start_field)
                ).map(|f|{
                    match &f.ident {
                        None => quote!{Parsable::parse(p)},
                        Some(id) => {
                            named = true;
                            quote!{#id:Parsable::parse(p)}
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
                body.push(quote!{#id:Default::default()})
            };

            match start_field {
                None => {},
                Some(id) => {
                    body.push(quote!{#id:Option::Some(start)})
                },
            };

            let build = if named {
                quote!{ #prefix{#(#body),*} }
            } else {
                quote!{ #prefix(#(#body),*) }
            };

            quote!{
                #pos_fetch
                #build
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
                    let transient_fields = extract_named_fields("Transient",v.fields.iter());
                    let start_field = extract_named_field("StartIndex",v.fields.iter());

                    let pos_fetch = match (&size_field,&start_field) {
                        (None,None) => quote!{},
                        _ => quote!{let start = p.index;},
                    };

                    let body = pattern_body_parse(v.fields.iter(),size_field, transient_fields, start_field);
                    cases.push(quote!{#num => {
                            #pos_fetch
                            #prefix::#name #body
                        }
                    })
                }
            }
            quote!{
                match p.consume_byte() {
                    #(#cases,)*
                    x => panic!("{:?} in {:?}",x, stringify!(#prefix))
                }
            }
        },
        Data::Union(_) => unimplemented!()
    };
    let plain = extract_plain_generics(&ast.generics);
    let argumented = extract_agumented_generics(&ast.generics, quote!{Parsable});
    quote!{
        impl<#(#argumented),*> Parsable for #prefix<#(#plain),*> {
             fn parse(p: &mut Parser) -> Self {
                #body
             }
        }
    }
}

fn pattern_body_serialize<'a>(fs: impl Iterator<Item=&'a Field>, size_field:Option<Ident>, transient_fields:Vec<Ident>, start_field:Option<Ident>) -> TokenStream {
    let mut body = Vec::new();
    for (idx,f) in fs.filter(|f|
        (size_field.is_none() || f.ident != size_field)
            && (f.ident.is_none() || !transient_fields.contains(&f.ident.as_ref().unwrap()))
            && (start_field.is_none() || f.ident != start_field)
    ).enumerate() {
        body.push(match &f.ident {
            None => {
                let id = Ident::new(&format!("_{}",idx), Span::call_site());
                quote!{#id.serialize(s)}
            },
            Some(id) => quote!{#id.serialize(s)},
        });
    }
    quote!{#(#body;)*}
}

fn pattern_body_parse<'a>(fs: impl Iterator<Item=&'a Field>, size_field:Option<Ident>, transient_fields:Vec<Ident>, start_field:Option<Ident>) -> TokenStream {
    let mut named = false;
    let mut body = Vec::new();
    for f in fs.filter(|f|
        (size_field.is_none() || f.ident != size_field)
            && (f.ident.is_none() || !transient_fields.contains(&f.ident.as_ref().unwrap()))
            && (start_field.is_none() || f.ident != start_field)
    ) {
        body.push(match &f.ident {
            None => quote!{Parsable::parse(p)},
            Some(id) => {
                named = true;
                quote!{#id:Parsable::parse(p)}
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
        body.push(quote!{#id:Default::default()})
    };

    match start_field {
        None => {},
        Some(id) => {
            body.push(quote!{#id:Option::Some(start)})
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
