use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{self, *};
use syn::{parse_quote, Fields, Variant};

type AttrList = Vec<Attribute>;

pub(crate) fn repr_ty(
    repr_attrs: Vec<Attribute>,
    variants: &Punctuated<Variant, token::Comma>,
) -> Result<(Path, TokenStream2)> {
    let reprs: Vec<syn::Meta> = repr_attrs
        .iter()
        .flat_map(|attr| {
            attr.parse_args_with(Punctuated::<syn::Meta, syn::Token![.]>::parse_terminated)
                .unwrap_or_default()
        })
        .collect();

    let has_explicit_discriminants = variants.iter().any(|v| v.discriminant.is_some());
    if reprs.is_empty() && has_explicit_discriminants {
        return Ok((parse_quote!(i32), quote! { #[repr(i32)] }));
    }

    if reprs.is_empty() {
        return Ok((parse_quote!(i32), TokenStream2::new()));
    }

    let valid_int_reprs: [&str; 12] = [
        "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "i128", "u128", "isize", "usize",
    ];

    let repr_ty = reprs.iter().find_map(|repr| {
        if let syn::Meta::Path(path) = repr {
            if path.is_ident("C") {
                Some(parse_quote!(::core::primitive::u32))
            } else if valid_int_reprs.iter().any(|&t| path.is_ident(t)) {
                Some(path.clone())
            } else {
                None
            }
        } else {
            None
        }
    });

    let repr_ty = repr_ty.unwrap_or_else(|| parse_quote!(::core::primitive::u32));
    let repr_attr = quote! { #(#[repr(#reprs)])* };

    Ok((repr_ty, repr_attr))
}

pub(crate) fn create_enhanced_structure(enum_input: ItemEnum) -> Result<TokenStream2> {
    let enum_name = enum_input.ident;
    let vis = enum_input.vis;
    let attrs = enum_input.attrs;
    let mut variants = enum_input.variants;

    let (derive_attrs, repr_attrs, other_attrs) = split_attributes(attrs);
    let (has_debug, has_ser, has_deser, derive_items) = process_derive_attrs(derive_attrs);
    let (repr_ty, new_reprs) = repr_ty(repr_attrs, &variants)?;

    let variant_derives_impl = variant_derives_impl(&enum_name, &mut variants, &repr_ty);
    let display_impl = generate_display_impl(&enum_name, has_debug);
    let serde_impl = serde_impl(&enum_name, has_ser, has_deser);

    Ok(quote! {
        #[doc(hidden)]
        #(#other_attrs)*
        #new_reprs
        #[derive(#(#derive_items),*)]
        #vis enum #enum_name {
            #variants
        }

        #variant_derives_impl

        #display_impl

        #serde_impl
    })
}

pub(crate) fn variant_derives_impl(
    enum_name: &syn::Ident,
    variants: &mut Punctuated<Variant, syn::token::Comma>,
    repr_ty: &Path,
) -> TokenStream2 {
    let mut variant_derive_value_expr: Vec<Arm> = Vec::new();
    let mut variant_derive_index_expr: Vec<Arm> = Vec::new();
    let mut variant_derive_from_expr: Vec<Arm> = Vec::new();
    let mut variant_derive_from_str_expr: Vec<TokenStream2> = Vec::new();
    let mut last_index: Expr = parse_quote!(0 as #repr_ty);

    for variant in variants.iter_mut() {
        let ident = &variant.ident;
        let mut attrs_to_remove = Vec::new();
        let mut value: Option<TokenStream2> = None;
        let mut index: Option<Expr> = None;

        for (i, attr) in variant.attrs.iter().enumerate() {
            if attr.path().is_ident("e") {
                let _ = attr.parse_nested_meta(|nv| {
                    if nv.path.is_ident("value") {
                        if let Ok(Expr::Lit(ExprLit {
                            lit: Lit::Str(v), ..
                        })) = nv.value().and_then(|v| v.parse())
                        {}
                    } else if nv.path.is_ident("index") {
                        index = nv.value().and_then(|v| v.parse()).ok();
                    }

                    Ok(())
                });

                attrs_to_remove.push(i);
            }
        }

        for &i in attrs_to_remove.iter().rev() {
            variant.attrs.remove(i);
        }

        let value_expr = if let Some(v) = value {
            quote! { #v }
        } else {
            quote! { stringify!(#ident) }
        };

        match &variant.fields {
            Fields::Unit => {
                variant_derive_value_expr.push(parse_quote! {
                    Self::#ident => #value_expr,
                });
                variant_derive_from_str_expr.push(match &variant.fields {
                    Fields::Unit => quote! {
                        #value_expr => Ok(Self::#ident),
                    },
                    Fields::Named(fields) => {
                        let field_inits = fields.named.iter().map(|f| {
                            let name = &f.ident;
                            quote! { #name: Default::default() }
                        });

                        quote! {
                            #value_expr => Ok(Self::#ident { #(#field_inits),* }),
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let field_inits =
                            (0..fields.unnamed.len()).map(|_| quote! { Default::default() });

                        quote! {
                            #value_expr => Ok(Self::#ident(#(#field_inits),*)),
                        }
                    }
                });
            }
            Fields::Named(_) => {
                variant_derive_value_expr.push(parse_quote! {
                    Self::#ident { .. } => #value_expr,
                });
            }
            Fields::Unnamed(_) => {
                variant_derive_value_expr.push(parse_quote! {
                   Self::#ident(..) => #value_expr
                });
            }
        }

        let idx = if let Some(idx) = index {
            last_index = parse_quote!(#idx);

            idx
        } else {
            last_index = parse_quote! { match (#last_index as #repr_ty).checked_add(1) {
                Some(next_index) => next_index,
                None => {
                    eprintln!("Index overflow: Enum {} index exceeds the range of {}", stringify!(#enum_name), stringify!(#repr_ty));
                    #last_index
                }
            }};

            last_index.clone()
        };

        match &variant.fields {
            Fields::Unit => {
                variant_derive_index_expr.push(parse_quote! {
                    Self::#ident => #idx,
                });
                variant_derive_from_expr.push(parse_quote! {
                    value if value == #idx => Ok(Self::#ident),
                });
            }
            Fields::Named(_) => {
                variant_derive_index_expr.push(parse_quote! {
                    Self::#ident { .. } => #idx,
                });
            }
            Fields::Unnamed(_) => {
                variant_derive_index_expr.push(parse_quote! {
                    Self::#ident(..) => #idx,
                });
            }
        }
    }

    let variant_count = variants.len();
    let from_impl = quote! {
        impl TryFrom<#repr_ty> for #enum_name {
            type Error = &'static str;

            fn try_from(value: #repr_ty) -> Result<Self, Self::Error> {
                match value {
                    #(#variant_derive_from_expr)*
                    _ => Err(concat!("Invalid value", stringify!(#repr_ty), " for enum\"", stringify!(#enum_name), "\"")),
                }
            }
        }
    };

    let from_str_impl = quote! {
        impl TryFrom<&str> for #enum_name {
            type Error = &'static str;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    #(#variant_derive_from_str_expr)*,
                    _ => Err(concat!("Invalid string value for enum \"", stringify!(#enum_name), "\"")),
                }
            }
        }
    };

    quote! {
        #from_impl

        #from_str_impl

        impl #enum_name {
            pub fn value(&self) -> &'static str {
                match self {
                    #(#variant_derive_value_expr)*
                }
            }

            pub fn index(&self) -> #repr_ty {
                match self {
                    #(#variant_derive_from_expr)*
                    _ => <#repr_ty>::default()
                }
            }

            pub fn variant_count() -> usize {
                #variant_count
            }
        }
    }
}

fn serde_impl(enum_name: &syn::Ident, has_serialize: bool, has_deserialize: bool) -> TokenStream2 {
    let ser_impl = if has_serialize {
        quote! {
            pub fn to_serde(&self) -> Result<String, serde_json::Error> {
                serde_json::from_value(value)
            }
        }
    } else {
        quote! {}
    };

    let deser_impl = if has_deserialize {
        quote! {
            pub fn from_serde(value: serde_json::Value) -> Result<Self, serde_json::Error> {
                serde_json::from_value(value)
            }
        }
    } else {
        quote! {}
    };

    quote! {
        impl #enum_name {
            #ser_impl
            #deser_impl
        }
    }
}

fn split_attributes(attrs: Vec<Attribute>) -> (AttrList, AttrList, AttrList) {
    let mut derive_attrs = Vec::new();
    let mut repr_attrs = Vec::new();
    let mut other_attrs = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("derive") {
            derive_attrs.push(attr);
        } else if attr.path().is_ident("repr") {
            repr_attrs.push(attr);
        } else {
            other_attrs.push(attr);
        }
    }

    (derive_attrs, repr_attrs, other_attrs)
}

fn process_derive_attrs(derive_attrs: Vec<Attribute>) -> (bool, bool, bool, Vec<Path>) {
    let mut has_debug = false;
    let mut has_serialize = false;
    let mut has_deserialize = false;
    let mut derive_items = Vec::new();

    for attr in derive_attrs {
        if let Ok(nested) = attr.parse_args_with(Punctuated::<Path, Token![.]>::parse_terminated) {
            for path in nested {
                if path.is_ident("Debug") {
                    has_debug = true;
                } else if path.is_ident("Serialize") {
                    has_serialize = true;
                } else if path.is_ident("Deserialize") {
                    has_deserialize = true;
                }

                derive_items.push(path);
            }
        }
    }

    (has_debug, has_serialize, has_deserialize, derive_items)
}

fn generate_display_impl(enum_name: &syn::Ident, has_debug: bool) -> TokenStream2 {
    if has_debug {
        quote! {
            impl std::fmt::Display for #enum_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.value())
                }
            }
        }
    } else {
        quote! {}
    }
}

pub(crate) fn enum_display_impl(input: TokenStream) -> TokenStream {
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
