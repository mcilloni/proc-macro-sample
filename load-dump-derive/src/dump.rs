use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, *};

use crate::attrs::*;

fn struct_field(field: &Field) -> Ident {
    let Field {
        ident: idopt,
        attrs,
        ..
    } = field;

    if attrs.iter().any(has_skip) {
        Ident::new("_", Span::call_site())
    } else {
        idopt.as_ref().expect("nameless field detected").clone()
    }
}

fn render_enum(name: &Ident, vars: &Punctuated<Variant, Token![,]>) -> TokenStream {
    use std::iter::repeat;

    let vmatches = repeat(name)
        .zip(vars.iter().enumerate())
        .map(render_enum_match);

    quote! {
        match self {
            #(
                #vmatches,
            )*
        }
    }
}

fn render_enum_dumps((pos, v): (Index, &Variant)) -> TokenStream {
    use syn::Fields::*;
    let fnames: Vec<Ident> = match v.fields {
        Named(FieldsNamed { ref named, .. }) => named.iter().map(struct_field).collect(),
        Unnamed(FieldsUnnamed { ref unnamed, .. }) => {
            unnamed.iter().enumerate().map(tuple_field).collect()
        }
        Unit => vec![],
    };

    quote! {
        crate::Dump::dump(&(#pos as u32), write)?;
        #(crate::Dump::dump(&#fnames, write)?;)*
    }
}

fn render_enum_match((name, (pos, var)): (&Ident, (usize, &Variant))) -> TokenStream {
    if var.attrs.iter().any(has_skip) {
        let vname = &var.ident;
        return quote! {&#name::#vname{..} => {}};
    }

    let vhead = render_enum_variant(var);

    let vdump = if let Some(msg) = var.attrs.iter().filter_map(filter_never).nth(0) {
        let name_str = name.to_string();
        let var_str = var.ident.to_string();
        quote! {panic!("{}::{} cannot be dumped: {}", #name_str, #var_str, #msg)}
    } else {
        render_enum_dumps((Index::from(pos), var))
    };

    quote! {&#name::#vhead => {
        #vdump
    }}
}

fn render_enum_variant(
    &Variant {
        ident: ref name,
        ref fields,
        ..
    }: &Variant,
) -> TokenStream {
    use syn::Fields::*;
    match *fields {
        Named(FieldsNamed { ref named, .. }) => {
            let fnames = named.iter().map(struct_field);
            quote! { #name { #(ref #fnames),* } }
        }

        Unnamed(FieldsUnnamed { ref unnamed, .. }) => {
            let pnames = (0..unnamed.len()).map(pnum);
            quote! { #name ( #(ref #pnames),* ) }
        }

        Unit => quote! {#name},
    }
}

fn render_struct_field_dump(field: &Field) -> TokenStream {
    if let Field {
        ident: Some(ref id),
        ref attrs,
        ..
    } = *field
    {
        if !attrs.iter().any(has_skip) {
            quote! {
                crate::Dump::dump(&self.#id, write)?;
            }
        } else {
            quote! {}
        }
    } else {
        panic!("nameless field detected")
    }
}

fn render_tuple_field_dump((pos, field): (usize, &Field)) -> TokenStream {
    let Field { ref attrs, .. } = *field;
    let pos = Index::from(pos);

    if !attrs.iter().any(has_skip) {
        quote! {
            crate::Dump::dump(&self.#pos, write)?;
        }
    } else {
        quote! {}
    }
}

fn render_struct_fields(vfields: &Fields) -> TokenStream {
    use syn::Fields::*;
    let dumps: Box<dyn Iterator<Item = TokenStream>> = match *vfields {
        Named(FieldsNamed { ref named, .. }) => {
            Box::new(named.iter().map(render_struct_field_dump))
        }
        Unnamed(FieldsUnnamed { ref unnamed, .. }) => {
            Box::new(unnamed.iter().enumerate().map(render_tuple_field_dump))
        }
        Unit => return quote! {},
    };

    quote! {
        #(
            #dumps
        )*
    }
}

fn tuple_field((num, &Field { ref attrs, .. }): (usize, &Field)) -> Ident {
    if attrs.iter().any(has_skip) {
        Ident::new("_", Span::call_site())
    } else {
        pnum(num)
    }
}

pub fn gen(ast: DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let dump_body = match ast.data {
        Data::Struct(DataStruct { fields, .. }) => render_struct_fields(&fields),
        Data::Enum(DataEnum { variants, .. }) => render_enum(name, &variants),
        Data::Union(_) => panic!("tagged unions are not supported"),
    };

    quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics crate::Dump for #name #ty_generics #where_clause {
            fn dump(&self, write: &mut (impl ::std::io::Write + ?Sized)) -> crate::err::Result<()> {
                {
                    #dump_body
                }

                Ok(())
            }
        }
    }
}
