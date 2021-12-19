use std::{error::Error, fmt::Display};
use syn::{parse_macro_input, parse_quote, DeriveInput, Expr, LitStr, Stmt};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};

use syn_helpers::{
    build_implementation_over_structure, BuildPair, Field, Fields, NamedField, NamedOrUnnamedField,
    PrefixAndPostfix, Trait, TraitMethod,
};

const IGNORE_DEBUG: &str = "debug_ignore";
const SINGLE_TUPLE_INLINE: &str = "debug_single_tuple_inline";
const DEBUG_AS_DISPLAY: &str = "debug_as_display";

#[proc_macro_derive(
    DebugExtras,
    attributes(debug_single_tuple_inline, debug_as_display, debug_ignore)
)]
pub fn debug_extras(input: TokenStream) -> TokenStream {
    let structure = parse_macro_input!(input as DeriveInput);

    // Debug trait
    let debug_trait = Trait {
        name: parse_quote!(::std::fmt::Debug),
        generic_parameters: vec![],
        methods: vec![TraitMethod {
            method_name: Ident::new("fmt", Span::call_site()),
            method_parameters: vec![
                parse_quote!(&self),
                parse_quote!(f: &mut ::std::fmt::Formatter<'_>),
            ],
            method_generics: vec![],
            return_type: Some(parse_quote!(::std::fmt::Result)),
            build_pair: BuildPair::NoPairing,
        }],
    };

    build_implementation_over_structure(
        &structure,
        debug_trait,
        |_, _| Ok(PrefixAndPostfix::default()),
        |method_name, fields| {
            if method_name == "fmt" {
                debug_extras_impl(fields)
            } else {
                unreachable!()
            }
        },
    )
    .into()
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

fn debug_extras_impl(fields: &mut Fields) -> Result<Vec<Stmt>, Box<dyn Error>> {
    let auto_debug_tuple_inline = cfg!(feature = "auto-debug-single-tuple-inline")
        && fields.fields_iterator().len() == 1
        && matches!(
            fields.fields_iterator().next().unwrap(),
            NamedOrUnnamedField::Unnamed(..)
        );

    let debug_single_tuple_inline = auto_debug_tuple_inline
        || fields
            .get_structure()
            .all_attributes()
            .any(|attr| attr.path.is_ident(SINGLE_TUPLE_INLINE));

    let expr: Expr = if debug_single_tuple_inline {
        if let Fields::Unnamed {
            fields,
            on_structure,
        } = fields
        {
            if let [field] = fields.as_mut_slice() {
                let read_expr = field.get_reference();
                let formatting_pattern = LitStr::new(
                    &format!("{}({{:?}})", on_structure.full_name()),
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
            Fields::Named {
                fields,
                on_structure,
            } => {
                let name = LitStr::new(&on_structure.full_name(), Span::call_site());
                let mut top = parse_quote! {
                    f.debug_struct(#name)
                };
                for field in fields.iter_mut() {
                    let mut expr = field.get_reference();
                    let NamedField { attrs, name, .. } = &field;
                    if attrs.iter().any(|attr| attr.path.is_ident(IGNORE_DEBUG)) {
                        continue;
                    }
                    if attrs
                        .iter()
                        .any(|attr| attr.path.is_ident(DEBUG_AS_DISPLAY))
                    {
                        expr = parse_quote! { format_args!("{}", &#expr) };
                    }

                    let field_name = LitStr::new(&name.to_string(), Span::call_site());
                    top = parse_quote! { #top.field(#field_name, &#expr) }
                }
                top
            }
            Fields::Unnamed {
                fields,
                on_structure,
            } => {
                let name = LitStr::new(&on_structure.full_name(), Span::call_site());
                let mut top = parse_quote! {
                    f.debug_tuple(#name)
                };
                for field in fields.iter_mut() {
                    let mut expr = field.get_reference();
                    if field
                        .attrs
                        .iter()
                        .any(|attr| attr.path.is_ident(IGNORE_DEBUG))
                    {
                        continue;
                    }
                    if field
                        .attrs
                        .iter()
                        .any(|attr| attr.path.is_ident(DEBUG_AS_DISPLAY))
                    {
                        expr = parse_quote! { format_args!("{}", &#expr) };
                    }
                    top = parse_quote! { #top.field(&#expr) }
                }
                top
            }
            Fields::Unit { on_structure } => {
                let name = LitStr::new(&on_structure.full_name(), Span::call_site());
                parse_quote! {
                    f.debug_struct(#name)
                }
            }
        };
        parse_quote! {
            #builder.finish()
        }
    };
    Ok(vec![Stmt::Expr(expr)])
}
