use proc_macro2::{Ident, Span};
use syn::*;

pub const ATTR_STR: &str = "load_dump";

pub fn filter_never(attr: &Attribute) -> Option<String> {
    if let Some(Meta::List(MetaList { ident, nested, .. })) = attr.interpret_meta() {
        if ident != ATTR_STR {
            return None;
        }

        for nm in nested.iter() {
            if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                ref ident, ref lit, ..
            })) = *nm
            {
                if ident == "never" {
                    if let Lit::Str(ref lit_str) = *lit {
                        return Some(lit_str.value());
                    }
                }
            }
        }
    }

    None
}

pub fn has_skip(attr: &Attribute) -> bool {
    if let Some(Meta::List(MetaList { ident, nested, .. })) = attr.interpret_meta() {
        if ident != ATTR_STR {
            return false;
        }

        nested.iter().any(|nm| {
            if let NestedMeta::Meta(Meta::Word(ref attr)) = *nm {
                attr == "skip"
            } else {
                false
            }
        })
    } else {
        false
    }
}

#[inline]
pub fn pnum(n: usize) -> Ident {
    Ident::new(&format!("_p{}", n), Span::call_site())
}
