use proc_macro2::TokenStream;

pub(crate) trait ResultIntoStreamExt {
    fn into_stream(self) -> TokenStream;
}

impl ResultIntoStreamExt for syn::Result<TokenStream> {
    fn into_stream(self) -> TokenStream {
        match self {
            Ok(stream) => stream,
            Err(err) => err.into_compile_error(),
        }
    }
}
