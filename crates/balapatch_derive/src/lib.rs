#![allow(non_snake_case)]

use proc_macro::*;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(EnumChoice)]
pub fn derive_enum_choice(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let ident = &derive_input.ident;

    let expanded = quote! {
        impl std::fmt::Display for #ident {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(f, "{self:?}")
            }
        }
    };

    TokenStream::from(expanded)
}

