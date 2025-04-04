use anyhow::Result;
use proc_macro::TokenStream;
use quote::quote;
use serde_json;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DeriveInput, Expr, Fields, Lit, Meta, MetaNameValue, Token, parse_macro_input,
};

fn create_attr_error<Meta, T: Spanned>(meta: &T, msg: &str) -> Result<Meta, syn::Error> {
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

struct StructMeta {
    name: String,
    help: String,
}

fn parse_name_value_string_attr(name_value: &MetaNameValue) -> Result<String, syn::Error> {
    match &name_value.value {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Str(s) => Ok(s.value()),
            _ => create_attr_error(&name_value.value, "expected string literal"),
        },
        _ => create_attr_error(&name_value.value, "expected string literal"),
    }
}

fn parse_protocol_field_attr_list(attr: &Attribute) -> Result<FieldMeta, syn::Error> {
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
                    return create_attr_error(&path, "expected one of `required`");
                }
            }
            Meta::NameValue(name_value) => {
                if name_value.path.is_ident("help") {
                    help = parse_name_value_string_attr(&name_value)?;
                } else if name_value.path.is_ident("required") {
                    match &name_value.value {
                        Expr::Lit(lit) => match &lit.lit {
                            Lit::Bool(b) => required = b.value,
                            _ => {
                                return create_attr_error(
                                    &name_value.value,
                                    "expected boolean literal",
                                );
                            }
                        },
                        _ => {
                            return create_attr_error(
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
                    return create_attr_error(
                        &name_value.path,
                        "expected one of `help`, `required`, `default`, `example`",
                    );
                }
            }
            _ => {
                return create_attr_error(
                    &meta,
                    "expected one of `help`, `required`, `default`, `example`",
                );
            }
        }
    }
    if help.is_empty() {
        Err(syn::Error::new(attr.span(), "Missing `help` attribute"))
    } else if required && default.is_none() && example.is_none() {
        Err(syn::Error::new(
            attr.span(),
            "Missing `default` or `example` attribute for required field",
        ))
    } else if !required && default.is_none() {
        Err(syn::Error::new(
            attr.span(),
            "Missing `default` attribute for optional field",
        ))
    } else {
        Ok(FieldMeta {
            help,
            required,
            default,
            example,
        })
    }
}

fn parse_protocol_struct_attr_list(attr: &Attribute) -> Result<StructMeta, syn::Error> {
    let mut name = String::new();
    let mut help = String::new();

    let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    for meta in nested {
        match meta {
            // name: string, help: string
            Meta::NameValue(name_value) => {
                if name_value.path.is_ident("name") {
                    name = parse_name_value_string_attr(&name_value)?;
                } else if name_value.path.is_ident("help") {
                    help = parse_name_value_string_attr(&name_value)?;
                } else {
                    return create_attr_error(&name_value.path, "expected one of `name`, `help`");
                }
            }
            _ => {
                return create_attr_error(&meta, "expected one of `name`, `help`");
            }
        }
    }
    if name.is_empty() {
        Err(syn::Error::new(attr.span(), "Missing `name` attribute"))
    } else if help.is_empty() {
        Err(syn::Error::new(attr.span(), "Missing `help` attribute"))
    } else {
        Ok(StructMeta { name, help })
    }
}

#[proc_macro_derive(ToolProtocol, attributes(tool_protocol))]
pub fn derive_tool_protocol(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

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
            Some(attr) => match parse_protocol_field_attr_list(attr) {
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

    let StructMeta {
        help: struct_help,
        name: mut struct_name,
    } = match input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("tool_protocol"))
    {
        Some(attr) => match parse_protocol_struct_attr_list(attr) {
            Ok(meta) => meta,
            Err(err) => return err.to_compile_error().into(),
        },
        None => {
            return syn::Error::new(input.span(), "Missing `tool_protocol` attribute")
                .to_compile_error()
                .into();
        }
    };

    let name = input.ident;
    let expanded = quote! {
        impl ToolProtocol<#name> for #name {
            fn create_schema() -> ToolSchema {
                ToolSchema {
                    name: #struct_name.to_string(),
                    help: #struct_help.to_string(),
                    arguments: vec![#(#arguments),*],
                }
            }
            fn parse_args(schema: &ToolSchema, args: &serde_json::Value) -> Result<#name> {
                let args = canonicalize_tool_args(schema, args)?;
                serde_json::from_value(args).map_err(Into::into)
            }
        }
    };

    TokenStream::from(expanded)
}

// #[tool(SomeProtocol)]
// pub struct SomeTool {
//     schema: ToolSchema,
// }
// expends to:
// impl Tool for SomeTool {
//     fn get_schema() -> &ToolSchema { &self.schema }
//     fn invoke(&mut self, args: &serde_json::Value) -> Result<String> {
//         self.invoke(parse_args::<SomeProtocol>(self.get_schema(), args)?)
//     }
// }
#[proc_macro_attribute]
pub fn tool(protocol: TokenStream, item: TokenStream) -> TokenStream {
    let protocol = parse_macro_input!(protocol as syn::Ident);
    let item = parse_macro_input!(item as syn::ItemStruct);

    let name = &item.ident;
    let schema = quote! {
        create_schema::<#protocol>()
    };

    let expanded = quote! {
        #item

        impl Tool for #name {
            fn get_schema(&self) -> &ToolSchema {
                &self.schema
            }

            fn invoke(&mut self, args: &serde_json::Value) -> Result<String> {
                let args = parse_args::<#protocol>(self.get_schema(), args)?;
                println!("\n\nCalling {:?} with {:?}", stringify!(#name), args);
                self.invoke(args)
            }
        }
    };

    TokenStream::from(expanded)
}
