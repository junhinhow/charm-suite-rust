//! Derive macros for [`charm-devtools`](https://docs.rs/charm-devtools).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, parse_quote, DeriveInput, GenericArgument, Meta, Token, Type};

/// Derive an explicit component tree for a named struct.
///
/// Fields are ignored unless marked with `#[inspect]` or
/// `#[devtools(inspect)]`. Use `#[inspect(skip)]` to explicitly omit a field.
/// Marked `Vec<T>` fields add one child per element.
#[proc_macro_derive(DevInspect, attributes(inspect, devtools))]
pub fn derive_dev_inspect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_dev_inspect(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_dev_inspect(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let mut generics = input.generics;
    let mut children = Vec::new();

    let syn::Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "DevInspect can only be derived for structs",
        ));
    };
    if let syn::Fields::Named(fields) = data.fields {
        for field in fields.named {
            let Some(ident) = field.ident else {
                continue;
            };
            let (inspect, skip) = field_inspection(&field.attrs)?;
            if !inspect || skip {
                continue;
            }

            if let Some(element_type) = vec_element_type(&field.ty) {
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote!(#element_type: ::charm_devtools::DevInspect));
                children.push(quote! {
                    for (index, value) in self.#ident.iter().enumerate() {
                        let mut child = ::charm_devtools::DevInspect::inspect(value);
                        child.name = ::std::format!("{}[{}]", stringify!(#ident), index);
                        children.push(child);
                    }
                });
            } else {
                let field_type = &field.ty;
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote!(#field_type: ::charm_devtools::DevInspect));
                children.push(quote! {
                    let mut child = ::charm_devtools::DevInspect::inspect(&self.#ident);
                    child.name = ::std::string::ToString::to_string(stringify!(#ident));
                    children.push(child);
                });
            }
        }
    }

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::charm_devtools::DevInspect for #name #type_generics #where_clause {
            fn inspect(&self) -> ::charm_devtools::ComponentNode {
                let mut children = ::std::vec::Vec::new();
                #(#children)*
                ::charm_devtools::ComponentNode::new(
                    stringify!(#name),
                    ::std::any::type_name::<Self>(),
                    children,
                )
            }
        }
    })
}

fn field_inspection(attributes: &[syn::Attribute]) -> syn::Result<(bool, bool)> {
    let mut inspect = false;
    let mut skip = false;

    for attribute in attributes {
        if attribute.path().is_ident("inspect") {
            match &attribute.meta {
                Meta::Path(_) => inspect = true,
                Meta::List(list) => {
                    let entries = Punctuated::<Meta, Token![,]>::parse_terminated
                        .parse2(list.tokens.clone())?;
                    for entry in entries {
                        if entry.path().is_ident("skip") {
                            skip = true;
                        } else {
                            return Err(syn::Error::new_spanned(
                                entry,
                                "expected `skip` inside `#[inspect(...)]`",
                            ));
                        }
                    }
                }
                Meta::NameValue(_) => {
                    return Err(syn::Error::new_spanned(
                        attribute,
                        "expected `#[inspect]` or `#[inspect(skip)]`",
                    ));
                }
            }
        } else if attribute.path().is_ident("devtools") {
            let Meta::List(list) = &attribute.meta else {
                return Err(syn::Error::new_spanned(
                    attribute,
                    "expected `#[devtools(inspect)]`",
                ));
            };
            let entries =
                Punctuated::<Meta, Token![,]>::parse_terminated.parse2(list.tokens.clone())?;
            for entry in entries {
                if entry.path().is_ident("inspect") {
                    inspect = true;
                } else if entry.path().is_ident("skip") {
                    skip = true;
                } else {
                    return Err(syn::Error::new_spanned(
                        entry,
                        "expected `inspect` or `skip` inside `#[devtools(...)]`",
                    ));
                }
            }
        }
    }

    Ok((inspect, skip))
}

fn vec_element_type(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Vec" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    arguments.args.iter().find_map(|argument| match argument {
        GenericArgument::Type(element_type) => Some(element_type),
        _ => None,
    })
}
