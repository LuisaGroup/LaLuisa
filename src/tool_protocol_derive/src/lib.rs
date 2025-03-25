use anyhow::Result;
use proc_macro::TokenStream;
use quote::quote;
use serde_json;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Expr, Fields, Lit, Meta, Token, parse_macro_input};

fn report_error(attr: &Attribute, name: &str) -> proc_macro::TokenStream {
    syn::Error::new(attr.span(), format!("Invalid {} attribute", name))
        .to_compile_error()
        .into()
}

fn create_field_error<T: Spanned>(meta: &T, msg: &str) -> Result<FieldMeta, syn::Error> {
    Err(syn::Error::new(meta.span(), msg))
}

fn parse_json_value(expr: Option<Expr>) -> Result<serde_json::Value, syn::Error> {
    match expr {
        Some(expr) => match serde_json::from_str(&quote!(#expr).to_string()) {
            Ok(value) => Ok(value),
            Err(err) => Err(syn::Error::new(expr.span(), err)),
        },
        None => Ok(serde_json::Value::Null),
    }
}

struct FieldMeta {
    help: String,
    required: bool,
    default: Option<Expr>,
    example: Option<Expr>,
}

fn parse_protocol_attr_list(attr: &Attribute) -> Result<FieldMeta, syn::Error> {
    let mut help = String::new();
    let mut required = false;
    let mut default = None;
    let mut example = None;

    let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    for meta in nested {
        match meta {
            Meta::Path(path) => {
                if path.is_ident("required") {
                    required = true;
                } else {
                    return create_field_error(&path, "expected one of `required`");
                }
            }
            Meta::NameValue(name_value) => {
                if name_value.path.is_ident("help") {
                    match &name_value.value {
                        Expr::Lit(lit) => match &lit.lit {
                            Lit::Str(s) => help = s.value(),
                            _ => {
                                return create_field_error(
                                    &name_value.value,
                                    "expected string literal",
                                );
                            }
                        },
                        _ => {
                            return create_field_error(
                                &name_value.value,
                                "expected string literal",
                            );
                        }
                    }
                } else if name_value.path.is_ident("required") {
                    match &name_value.value {
                        Expr::Lit(lit) => match &lit.lit {
                            Lit::Bool(b) => required = b.value,
                            _ => {
                                return create_field_error(
                                    &name_value.value,
                                    "expected boolean literal",
                                );
                            }
                        },
                        _ => {
                            return create_field_error(
                                &name_value.value,
                                "expected boolean literal",
                            );
                        }
                    }
                } else if name_value.path.is_ident("default") {
                    default = Some(name_value.value);
                } else if name_value.path.is_ident("example") {
                    example = Some(name_value.value);
                } else {
                    return create_field_error(
                        &name_value.path,
                        "expected one of `help`, `required`, `default`, `example`",
                    );
                }
            }
            _ => {
                return create_field_error(
                    &meta,
                    "expected one of `help`, `required`, `default`, `example`",
                );
            }
        }
    }
    Ok(FieldMeta {
        help,
        required,
        default,
        example,
    })
}

#[proc_macro_derive(ToolProtocol, attributes(tool_protocol))]
pub fn derive_tool_protocol(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new(input.span(), "Only named fields are supported")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new(input.span(), "Only structs are supported")
                .to_compile_error()
                .into();
        }
    };

    let mut arguments = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type = &field.ty;

        let FieldMeta {
            help,
            required,
            default,
            example,
        } = match field
            .attrs
            .iter()
            .find(|a| a.path().is_ident("tool_protocol"))
        {
            Some(attr) => match parse_protocol_attr_list(attr) {
                Ok(meta) => meta,
                Err(err) => return err.to_compile_error().into(),
            },
            None => {
                return syn::Error::new(field.span(), "Missing `tool_protocol` attribute")
                    .to_compile_error()
                    .into();
            }
        };

        let type_str = quote!(#field_type).to_string();
        let default_value = match parse_json_value(default) {
            Ok(value) => value.to_string(),
            Err(err) => return err.to_compile_error().into(),
        };
        let example_value = match parse_json_value(example) {
            Ok(value) => value.to_string(),
            Err(err) => return err.to_compile_error().into(),
        };

        arguments.push(quote! {
            ToolArgument {
                name: #field_name_str.to_string(),
                help: #help.to_string(),
                type_: #type_str.to_string(),
                required: #required,
                default: serde_json::from_str(#default_value).unwrap(),
                example: serde_json::from_str(#example_value).unwrap(),
            }
        });
    }

    let struct_help = match input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("tool_protocol"))
    {
        Some(attr) => match parse_protocol_attr_list(attr) {
            Ok(meta) => meta.help,
            Err(err) => return err.to_compile_error().into(),
        },
        None => String::new(),
    };

    let expanded = quote! {
        impl ToolProtocol for #name {
            fn get_schema() -> ToolSchema {
                ToolSchema {
                    name: stringify!(#name).to_string(),
                    help: #struct_help.to_string(),
                    arguments: vec![#(#arguments),*],
                }
            }
        }
    };

    TokenStream::from(expanded)
}
