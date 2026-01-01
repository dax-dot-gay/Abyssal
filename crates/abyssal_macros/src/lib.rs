use proc_macro2::TokenStream;
mod error;

#[manyhow::manyhow(proc_macro_attribute)]
pub fn make_error(args: TokenStream, item: TokenStream) -> manyhow::Result<TokenStream> {
    error::impl_error(args, item)
}
