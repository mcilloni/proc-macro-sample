use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

use crate::attrs::*;

fn render_enum_match((name, var): (&Ident, &Variant)) -> TokenStream {
    let vname = &var.ident;

    if let Some(msg) = var.attrs.iter().filter_map(filter_never).nth(0) {
        let name = name.to_string();
        let vname = vname.to_string();

        quote! {panic!("{}::{} cannot be loaded: {}", #name, #vname, #msg)}
    } else {
        let loads = render_fields(&var.fields);

        quote! {#name::#vname{#loads}}
    }
}

fn render_fields(fields: &Fields) -> TokenStream {
    #[inline]
    fn gen_loads<T: quote::ToTokens>(skip: bool, id: &T) -> TokenStream {
        if skip {
            quote! {#id: std::default::Default::default()}
        } else {
            quote! {#id : ::proc_macro_sample::Load::load(read)?}
        }
    }

    use syn::Fields::*;
    let loads: Box<dyn Iterator<Item = TokenStream>> = match *fields {
        Named(FieldsNamed { ref named, .. }) => Box::new(named.iter().map(|field| {
            if let Field {
                ident: Some(ref id),
                ref attrs,
                ..
            } = *field
            {
                gen_loads(attrs.iter().any(has_skip), id)
            } else {
                panic!("nameless field detected")
            }
        })),

        Unnamed(FieldsUnnamed { ref unnamed, .. }) => Box::new(unnamed.iter().enumerate().map(
            |(n, &Field { ref attrs, .. })| gen_loads(attrs.iter().any(has_skip), &Index::from(n)),
        )),

        Unit => return quote! {},
    };

    quote! {
        #(
            #loads,
        )*
    }
}

pub fn gen(ast: DeriveInput) -> TokenStream {
    use std::iter::repeat;

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let load_body = match ast.data {
        Data::Struct(DataStruct { fields, .. }) => {
            let inside = render_fields(&fields);

            quote! {
                Self {
                    #inside
                }
            }
        }

        Data::Enum(DataEnum { variants, .. }) => {
            let int_vals = (0..variants.len())
                .map(Index::from)
                .map(|idx| quote! {#idx});

            let loads = repeat(name).zip(&variants).map(render_enum_match);

            quote! {
                {
                    let pos : u32 = ::proc_macro_sample::Load::load(read)?;

                    match pos {
                        #(
                            #int_vals => #loads,
                        )*
                        _ => panic!("{} is out of enum values range", pos),
                    }
                }
            }
        }

        Data::Union(..) => panic!("unions cannot implement Load"),
    };

    quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics ::proc_macro_sample::Load for #name #ty_generics #where_clause {
            fn load(read: &mut impl std::io::Read) -> ::proc_macro_sample::Result<Self> {
                Ok(#load_body)
            }
        }
    }
}
