use proc_macro2::TokenStream;
mod error;
mod permission;

#[manyhow::manyhow(proc_macro_attribute)]
pub fn make_error(args: TokenStream, item: TokenStream) -> manyhow::Result<TokenStream> {
    error::impl_error(args, item)
}

#[manyhow::manyhow(proc_macro)]
pub fn make_permissions(input: TokenStream) -> manyhow::Result<TokenStream> {
    permission::impl_permissions(input)
}
