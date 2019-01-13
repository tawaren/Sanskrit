
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
use syn::punctuated::Pair;

#[proc_macro_derive(Serializable)]
pub fn serialize_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast:DeriveInput  = syn::parse(input).unwrap();

    // Build the trait implementation
    //println!("{}", impl_serialize_macro(&ast));
    impl_serialize_macro(&ast).into()
}

#[proc_macro_derive(Parsable, attributes(AllocLifetime))]
pub fn parse_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast:DeriveInput  = syn::parse(input).unwrap();

    // Build the trait implementation
    //println!("{}",impl_parsable_macro(&ast));
    impl_parsable_macro(&ast).into()
}

#[proc_macro_derive(VirtualSize)]
pub fn virtual_size_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
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
            let mut body = Vec::new();
            for (idx,f) in ds.fields.iter().enumerate() {
                body.push(match &f.ident {
                    None => quote!{self.#idx.serialize(s)},
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
                    let body = pattern_body_serialize(v.fields.iter());
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
        Data::Union(ref du) => unimplemented!(),
    };

    let generics = &ast.generics;
    let plain = extract_plain_generics(&ast.generics);
    let argumented = extract_agumented_generics(&ast.generics, quote!{Serializable});
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
            let mut named = false;
            let body:Vec<_> = ds.fields.iter().map(|f|{
                match &f.ident {
                    None => quote!{Parsable::parse(p,alloc)?},
                    Some(id) => {
                        named = true;
                        quote!{#id:Parsable::parse(p,alloc)?}
                    },
                }
            }).collect();
            if named {
                quote!{Ok(#prefix{#(#body),*})}
            } else {
                quote!{Ok(#prefix(#(#body),*))}
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
                    let body = pattern_body_parse(v.fields.iter());
                    cases.push(quote!{#num => #prefix::#name #body})
                }
            }
            quote!{
                Ok(match p.consume_byte()? {
                    #(#cases,)*
                    _ => return parsing_case_failure()
                })
            }
        },
        Data::Union(ref du) => unimplemented!()
    };
    let generics = &ast.generics;
    let alloc_id = extract_alloc_lifetime(generics);
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
            let res = max_all(&mut ed.variants.iter().map(|v|expr(v.fields.iter())).collect());
            quote!{1+#res} //1Byte for the tag
        },
        Data::Union(ref du) => unimplemented!(),
    };

    let generics = &ast.generics;
    let plain = extract_plain_generics(&ast.generics);
    let argumented = extract_agumented_generics(&ast.generics, quote!{VirtualSize});

    quote!{
        impl<#(#argumented),*> VirtualSize for #prefix<#(#plain),*> {
             const SIZE:usize = #body;
        }
    }
}


fn pattern_body_serialize<'a>(fs: impl Iterator<Item=&'a Field>) -> TokenStream {
    let mut body = Vec::new();
    for (idx,f) in fs.enumerate() {
        body.push(match &f.ident {
            None => {
                let id = Ident::new(&format!("_{}",idx), Span::call_site());
                quote!{#id.serialize(s)}
            }
            Some(id) => quote!{#id.serialize(s)},
        })
    }
    quote!{#(#body;)*}
}


fn pattern_body_parse<'a>(fs: impl Iterator<Item=&'a Field>) -> TokenStream {
    let mut named = false;
    let mut body = Vec::new();
    for (idx,f) in fs.enumerate() {
        body.push(match &f.ident {
            None => quote!{Parsable::parse(p,alloc)?},
            Some(id) => {
                named = true;
                quote!{#id:Parsable::parse(p,alloc)?}
            },
        })
    }
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

fn extract_alloc_lifetime(generics:&Generics) -> Option<TokenStream> {
    for l in generics.lifetimes() {
        for a in &l.attrs {
            match  a.path.segments.last() {
                None => {}
                Some(Pair::Punctuated(p,_)) | Some(Pair::End(p))  => {
                    if p.ident.to_string() == "AllocLifetime" {
                        return Some(l.clone().lifetime.into_token_stream())
                    }
                }
            }
        }
    }
    None
}