use crate::filter_ext::FilterExt;
use crate::macros::{bail, error};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_str, Fields, Ident, Lit, Meta, NestedMeta, Path, Result};
use synstructure::{decl_derive, AddBounds, BindingInfo, Structure, VariantInfo};

mod filter_ext;
mod macros;
#[cfg(test)]
mod tests;

decl_derive!([Debug, attributes(debug)] => custom_debug_derive);

fn custom_debug_derive(mut structure: Structure) -> Result<TokenStream> {
    filter_out_skipped_fields(&mut structure)?;

    structure.add_bounds(AddBounds::Fields);

    let match_arms =
        structure.each_variant(|variant| generate_match_arm_body(variant).into_stream());

    Ok(structure.gen_impl(quote! {
        gen impl ::core::fmt::Debug for @Self {
            fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    #match_arms
                }
            }
        }
    }))
}

fn filter_out_skipped_fields(structure: &mut Structure) -> Result<()> {
    let skip_ident: Ident = parse_str("skip").unwrap();

    structure.try_filter(|binding| {
        for meta in get_metas(binding) {
            let meta = meta?;

            if let NestedMeta::Meta(Meta::Path(ref path)) = meta {
                if path
                    .get_ident()
                    .map(|ident| ident == &skip_ident)
                    .unwrap_or(false)
                {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    })?;

    Ok(())
}

fn generate_match_arm_body(variant: &VariantInfo) -> Result<TokenStream> {
    let name = variant.ast().ident.to_string();
    let debug_builder = match variant.ast().fields {
        Fields::Named(_) | Fields::Unit => quote! { debug_struct },
        Fields::Unnamed(_) => quote! { debug_tuple },
    };
    let debug_builder_calls = variant
        .bindings()
        .iter()
        .map(generate_debug_builder_call)
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        let mut debug_builder = fmt.#debug_builder(#name);

        #(#debug_builder_calls)*

        debug_builder.finish()
    })
}

fn generate_debug_builder_call(binding: &BindingInfo) -> Result<TokenStream> {
    let mut format = None;

    for meta in get_metas(binding) {
        let meta = meta?;

        match meta {
            NestedMeta::Meta(Meta::NameValue(nv)) => {
                format = Some(generate_name_value_builder_call(binding, nv)?)
            }
            _ => bail!(meta.span(), "Unsupported attribute"),
        }
    }

    let format = format.unwrap_or_else(|| quote! { #binding });

    let debug_builder_call =
        if let Some(ref name) = binding.ast().ident.as_ref().map(<_>::to_string) {
            quote! {
                debug_builder.field(#name, #format);
            }
        } else {
            quote! {
                debug_builder.field(#format);
            }
        };

    Ok(debug_builder_call)
}

fn generate_name_value_builder_call(
    binding: &BindingInfo,
    nv: syn::MetaNameValue,
) -> Result<TokenStream> {
    let key_span = nv.path.span();
    let value_span = nv.lit.span();
    let value = nv.lit;
    let ident = nv
        .path
        .get_ident()
        .map(Ident::to_string)
        .ok_or_else(|| error!(key_span, "Unsupported attribute"))?;

    match &*ident {
        "format" => Ok(quote! { &format_args!(#value, #binding) }),
        "with" => match value {
            Lit::Str(fun) => {
                let fun = fun
                    .parse::<Path>()
                    .map_err(|_| error!(fun.span(), "Invalid path to function"))?;

                Ok(quote! {
                    {
                        struct DebugWith<'a, T: 'a> {
                            data: &'a T,
                            fmt: fn(&T, &mut ::core::fmt::Formatter) -> ::core::fmt::Result,
                        }

                        impl<'a, T: 'a> ::core::fmt::Debug for DebugWith<'a, T> {
                            fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                                (self.fmt)(self.data, fmt)
                            }
                        }

                        &DebugWith {
                            data: #binding,
                            fmt: #fun,
                        }
                    }
                })
            }
            _ => bail!(value_span, "Invalid `with` value"),
        },
        _ => bail!(key_span, "Unsupported attribute"),
    }
}

fn get_metas<'a>(binding: &BindingInfo<'a>) -> impl Iterator<Item = Result<NestedMeta>> + 'a {
    let debug_attr = parse_str::<Path>("debug").unwrap();

    binding
        .ast()
        .attrs
        .iter()
        .filter(move |attr| attr.path == debug_attr)
        .map(|attr| {
            let meta = attr.parse_meta()?;

            match meta {
                Meta::List(list) => Ok(list.nested),
                _ => bail!(meta.span(), "Unsupported attribute style, use `debug(…)`"),
            }
        })
        .flatten_ok()
}

trait ResultIntoStreamExt {
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
