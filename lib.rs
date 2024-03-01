use proc_macro::TokenStream;
use std::{error::Error, fmt::Display};

use syn_helpers::{
    derive_trait, path_to_string,
    proc_macro2::{Ident, Span},
    syn::{parse_macro_input, parse_quote, DeriveInput, Expr, LitStr, Stmt},
    Constructable, FieldMut, Fields, NamedField, NamedOrUnnamedField, Trait, TraitItem,
};

const IGNORE_DEBUG: &str = "debug_ignore";
const SINGLE_TUPLE_INLINE: &str = "debug_single_tuple_inline";
const DEBUG_AS_DISPLAY: &str = "debug_as_display";

#[proc_macro_derive(
    DebugExtras,
    attributes(debug_single_tuple_inline, debug_as_display, debug_ignore)
)]
pub fn debug_extras(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let fmt_item = TraitItem::new_method(
        Ident::new("fmt", Span::call_site()),
        None,
        syn_helpers::TypeOfSelf::Reference,
        vec![parse_quote!(f: &mut ::std::fmt::Formatter<'_>)],
        Some(parse_quote!(::std::fmt::Result)),
        |mut item| {
            item.map_constructable(|mut constructable| {
                let full_path_name_string = path_to_string(constructable.get_constructor_path());
                let name = LitStr::new(&full_path_name_string, Span::call_site());

                let inline_tuple_attribute = constructable
                    .all_attributes()
                    .any(|attr| attr.path().is_ident(SINGLE_TUPLE_INLINE));

                let fields = &mut constructable.get_fields_mut();

                let auto_debug_tuple_inline = cfg!(feature = "auto-debug-single-tuple-inline")
                    && fields.fields_iterator().len() == 1
                    && matches!(
                        fields.fields_iterator().next().unwrap(),
                        NamedOrUnnamedField::Unnamed(..)
                    );

                let debug_single_tuple_inline = inline_tuple_attribute || auto_debug_tuple_inline;

                let expr: Expr = if debug_single_tuple_inline {
                    if let Fields::Unnamed(fields, _) = fields
                    {
                        if let [field] = fields.as_mut_slice() {
                            let read_expr = field.get_reference();
                            let formatting_pattern = LitStr::new(
                                &format!("{}({{:?}})", full_path_name_string),
                                Span::call_site(),
                            );
                            parse_quote! { f.write_fmt(format_args!(#formatting_pattern, #read_expr)) }
                        } else {
                            return Err(Box::new(
                                DebugExtrasErrors::DebugSingleTupleInlineInvalidStructure,
                            ));
                        }
                    } else {
                        return Err(Box::new(
                            DebugExtrasErrors::DebugSingleTupleInlineInvalidStructure,
                        ));
                    }
                } else {
                    let builder: Expr = match fields {
                        Fields::Named(fields, _) => {
                            let mut top = parse_quote! {
                                f.debug_struct(#name)
                            };
                            for field in fields.iter_mut() {
                                let mut expr = field.get_reference();
                                let NamedField { attrs, name, .. } = &field;
                                if attrs.iter().any(|attr| attr.path().is_ident(IGNORE_DEBUG)) {
                                    continue;
                                }
                                if attrs
                                    .iter()
                                    .any(|attr| attr.path().is_ident(DEBUG_AS_DISPLAY))
                                {
                                    expr = parse_quote! { format_args!("{}", &#expr) };
                                }

                                let field_name = LitStr::new(&name.to_string(), Span::call_site());
                                top = parse_quote! { #top.field(#field_name, &#expr) }
                            }
                            top
                        }
                        Fields::Unnamed(fields, _) => {
                            let mut top = parse_quote! {
                                f.debug_tuple(#name)
                            };
                            for field in fields.iter_mut() {
                                let mut expr = field.get_reference();
                                if field
                                    .attrs
                                    .iter()
                                    .any(|attr| attr.path().is_ident(IGNORE_DEBUG))
                                {
                                    continue;
                                }
                                if field
                                    .attrs
                                    .iter()
                                    .any(|attr| attr.path().is_ident(DEBUG_AS_DISPLAY))
                                {
                                    expr = parse_quote! { format_args!("{}", &#expr) };
                                }
                                top = parse_quote! { #top.field(&#expr) }
                            }
                            top
                        }
                        Fields::Unit(..) => {
                            parse_quote! {
                                f.debug_struct(#name)
                            }
                        }
                    };
                    parse_quote! {
                        #builder.finish()
                    }
                };
                Ok(vec![Stmt::Expr(expr, None)])
            })
        },
    );

    // Debug trait
    let debug_trait = Trait {
        name: parse_quote!(::std::fmt::Debug),
        generic_parameters: None,
        items: vec![fmt_item],
    };

    derive_trait(input, debug_trait).into()
}

#[derive(Debug)]
enum DebugExtrasErrors {
    DebugSingleTupleInlineInvalidStructure,
}

impl Display for DebugExtrasErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugExtrasErrors::DebugSingleTupleInlineInvalidStructure => {
                f.write_str("Must be tuple struct with one item")
            }
        }
    }
}

impl Error for DebugExtrasErrors {}
