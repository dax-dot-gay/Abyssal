use std::collections::HashMap;

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Arm, Expr, ExprLit, Fields, Ident, ItemEnum, Lit, MetaNameValue, Pat, PatStruct, Token,
    Variant, parse::Parser, punctuated::Punctuated,
};

#[derive(Debug, Clone)]
struct ErrorItemArgs {
    pub status: u16,
    pub format: String,
    pub description: Option<String>,
}

fn process_item(item: Variant, meta_ident: Ident) -> manyhow::Result<(Variant, Arm)> {
    if let Some(attribute) = item
        .attrs
        .clone()
        .into_iter()
        .filter(|a| a.path().is_ident("error"))
        .next()
    {
        let args_list =
            attribute.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;
        let kvs = args_list
            .into_iter()
            .filter_map(|v| {
                if let Some(ident) = v.path.get_ident().clone() {
                    Some((ident.to_string(), v.value.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();

        let args = ErrorItemArgs {
            status: kvs
                .get("status")
                .cloned()
                .and_then(|v| {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Int(lit), ..
                    }) = v
                    {
                        Some(lit.base10_parse::<u16>().unwrap())
                    } else {
                        None
                    }
                })
                .unwrap_or(500),
            format: kvs
                .get("format")
                .cloned()
                .and_then(|v| {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit), ..
                    }) = v
                    {
                        Some(lit.value())
                    } else {
                        None
                    }
                })
                .unwrap_or(String::from("{0:?}")),
            description: kvs.get("description").cloned().and_then(|v| {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit), ..
                }) = v
                {
                    Some(lit.value())
                } else {
                    None
                }
            }),
        };

        let fields = item.fields.clone();
        let args_format = args.format.clone();
        let args_status = args.status;
        let args_description = args.description;

        let item_ident = item.ident.clone();
        let meta_code = item_ident.to_string().to_case(Case::Snake);
        let item_fields = item.fields.clone();

        let new_variant: Variant = if let Some(desc) = args_description.clone() {
            syn::parse2(quote! {
                #[doc = #desc]
                #[error(#args_format)]
                #item_ident #item_fields
            })?
        } else {
            syn::parse2(quote! {
                #[error(#args_format)]
                #item_ident #item_fields
            })?
        };
        let pattern: TokenStream = match fields {
            Fields::Named(_) => quote! {Self::#item_ident {..}},
            Fields::Unnamed(_) => quote! {Self::#item_ident (..)},
            Fields::Unit => quote! {Self::#item_ident},
        };

        let meta_arm: Arm = if let Some(desc) = args_description {
            syn::parse2(quote! {
                #pattern => #meta_ident {
                    status: #args_status,
                    code: String::from(#meta_code),
                    message: format!("{self}"),
                    description: String::from(#desc)
                }
            })?
        } else {
            syn::parse2(quote! {
                #pattern => #meta_ident {
                    status: #args_status,
                    code: String::from(#meta_code),
                    message: format!("{}", self.clone()),
                    description: String::new()
                }
            })?
        };

        Ok((new_variant, meta_arm))
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            "Missing #[error(...)] attribute!",
        ))?
    }
}

pub fn impl_error(_: TokenStream, item: TokenStream) -> manyhow::Result<TokenStream> {
    let input = syn::parse2::<ItemEnum>(item)?;
    let enum_visibility = input.vis.clone();
    let enum_ident = input.ident.clone();
    let mod_ident = format_ident!("error_mod_{}", enum_ident.to_string().to_case(Case::Snake));
    let metadata_ident = format_ident!("{}Meta", enum_ident.clone());

    let mut error_variants: Punctuated<Variant, Token![,]> = Punctuated::new();
    let mut meta_arms: Punctuated<Arm, Token![,]> = Punctuated::new();
    for variant in input.variants {
        let (new_variant, meta_arm) = process_item(variant, metadata_ident.clone())?;
        error_variants.push(new_variant);
        meta_arms.push(meta_arm);
    }

    Ok(quote! {
        mod #mod_ident {
            use super::*;
            use rocket::{
                response::{self, Response, Responder},
                request::Request,
                http::{ContentType, Status},
                serde::json::Json
            };

            #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Eq, PartialEq)]
            pub struct #metadata_ident {
                pub status: u16,
                pub code: String,
                pub message: String,
                pub description: String
            }

            #[derive(Clone, Debug, thiserror::Error)]
            pub enum #enum_ident {
                #error_variants
            }

            impl #enum_ident {
                pub fn metadata(&self) -> #metadata_ident {
                    match self.clone() {
                        #meta_arms
                    }
                }
            }

            impl<'r> Responder<'r, 'static> for #enum_ident {
                fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
                    Response::build_from(Json(self.metadata()).respond_to(req)?).status(Status::new(self.metadata().status)).ok()
                }
            }
        }

        #enum_visibility use #mod_ident::{#enum_ident, #metadata_ident};
    })
}
