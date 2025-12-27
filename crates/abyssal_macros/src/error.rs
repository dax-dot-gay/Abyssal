use convert_case::{Case, Casing};
use darling::FromAttributes;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Arm, Fields, Ident, ItemEnum, ItemImpl, Token, Variant, punctuated::Punctuated
};

#[derive(Debug, Clone, FromAttributes)]
#[darling(attributes(error))]
struct Error {
    #[darling(default = || 500)]
    pub status: u16,

    #[darling(default = || String::from("{0:?}"))]
    pub format: String,

    #[darling(default)]
    pub code: Option<String>,

    #[darling(default)]
    pub description: Option<String>,

    #[darling(default)]
    pub from: bool,

    #[darling(default)]
    pub arc: bool,
}

fn process_item(parent: Ident, item: Variant, meta_ident: Ident) -> manyhow::Result<(Variant, Arm, Option<ItemImpl>)> {
    let args = Error::from_attributes(&item.attrs).map_err(|e| syn::Error::new(Span::call_site(), e.to_string()))?;

    let fields = item.fields.clone();
    let Error {
        status: args_status,
        format: args_format,
        code: args_code,
        description: args_description,
        from: args_from,
        arc: args_arc,
    } = args;

    

    let item_ident = item.ident.clone();
    let meta_code = args_code.unwrap_or(item_ident.to_string()).to_case(Case::Snake);
    let item_fields = item.fields.clone();
    let tuple_type = if let Fields::Unnamed(members) = item_fields.clone() {
        if members.unnamed.len() == 1 {
            Some(members.unnamed.first().cloned().unwrap().ty)
        } else {
            None
        }
    } else {
        None
    };

    let new_variant: Variant = if args_arc {
        if tuple_type.is_none() {
            return Err(syn::Error::new(Span::call_site(), "Can only use `arc` on tuple variants with exactly one field.").into());
        }

        let definite_type = tuple_type.clone().unwrap();
        if let Some(desc) = args_description.clone() {
            syn::parse2(quote! {
                #[doc = #desc]
                #[error(#args_format)]
                #item_ident (std::sync::Arc<#definite_type>)
            })?
        } else {
            syn::parse2(quote! {
                #[error(#args_format)]
                #item_ident (std::sync::Arc<#definite_type>)
            })?
        }
    } else if let Some(desc) = args_description.clone() {
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

    let impl_from: Option<ItemImpl> = if args_from {
        if tuple_type.is_none() {
            return Err(syn::Error::new(Span::call_site(), "Can only use `from` on tuple variants with exactly one field.").into());
        }
        let definite_type = tuple_type.clone().unwrap();
        if args_arc {
            Some(syn::parse2(quote! {
                impl From<#definite_type> for #parent {
                    fn from(value: #definite_type) -> Self {
                        Self::#item_ident(std::sync::Arc::new(value))
                    }
                }
            })?)
        } else {
            Some(syn::parse2(quote! {
                impl From<#definite_type> for #parent {
                    fn from(value: #definite_type) -> Self {
                        Self::#item_ident(value)
                    }
                }
            })?)
        }
    } else {
        None
    };

    Ok((new_variant, meta_arm, impl_from))
}

pub fn impl_error(_: TokenStream, item: TokenStream) -> manyhow::Result<TokenStream> {
    let input = syn::parse2::<ItemEnum>(item)?;
    let enum_visibility = input.vis.clone();
    let enum_ident = input.ident.clone();
    let mod_ident = format_ident!("error_mod_{}", enum_ident.to_string().to_case(Case::Snake));
    let metadata_ident = format_ident!("{}Meta", enum_ident.clone());

    let mut error_variants: Punctuated<Variant, Token![,]> = Punctuated::new();
    let mut meta_arms: Punctuated<Arm, Token![,]> = Punctuated::new();
    let mut impl_froms: Vec<ItemImpl> = vec![];
    for variant in input.variants {
        let (new_variant, meta_arm, impl_from) = process_item(enum_ident.clone(), variant, metadata_ident.clone())?;
        error_variants.push(new_variant);
        meta_arms.push(meta_arm);
        if let Some(ifr) = impl_from {
            impl_froms.push(ifr);
        }
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

            #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
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

            #(#impl_froms)*

            use okapi::openapi3::{Responses, Response as OkResponse};
            use rocket_okapi::{r#gen::OpenApiGenerator, response::OpenApiResponderInner};
            impl OpenApiResponderInner for Error {
                fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
                    let mut responses = Responses::default();
                    responses.responses.entry("default".to_owned()).or_insert_with(|| OkResponse::default().into());
                    Ok(responses)
                }
            }
        }

        #enum_visibility use #mod_ident::{#enum_ident, #metadata_ident};
    })
}
