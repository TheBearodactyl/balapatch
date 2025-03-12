#![allow(non_snake_case, unused)]

use crate::enum_macros::{enum_choice_impl, enum_display_impl};
use proc_macro::*;
use syn::parse_macro_input;

mod enum_macros;
mod struct_macros;
mod utils;

#[proc_macro_derive(EnumDisplay)]
pub fn derive_enum_choice(input: TokenStream) -> TokenStream {
    enum_display_impl(input)
}

#[proc_macro_derive(EnumChoice)]
pub fn derive_variants(input: TokenStream) -> TokenStream {
    enum_choice_impl(input)
}

#[proc_macro_attribute]
pub fn enhanced_enum(_: TokenStream, item: TokenStream) -> TokenStream {
    let enum_input = parse_macro_input!(item as syn::ItemEnum);

    enum_macros::create_enhanced_structure(enum_input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
