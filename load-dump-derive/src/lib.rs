extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

mod attrs;
mod dump;
mod load;

#[proc_macro_derive(Dump, attributes(load_dump))]
pub fn dump_impl(input: TokenStream) -> TokenStream {
    // Parse the input token stream
    let input = syn::parse(input).unwrap();

    // Build the impl
    let gen = dump::gen(input);

    // Return the generated impl
    gen.into()
}

#[proc_macro_derive(Load, attributes(load_dump))]
pub fn load_impl(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let input = syn::parse(input).unwrap();

    // Build the impl
    let gen = load::gen(input);

    // Return the generated impl
    gen.into()
}
