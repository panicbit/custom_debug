use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, Result};
use synstructure::{decl_derive, AddBounds, BindingInfo, Structure, VariantInfo};

use crate::field_attributes::{DebugFormat, FieldAttributes, SkipMode};
use crate::filter_ext::RetainExt;
use crate::result_into_stream_ext::ResultIntoStreamExt;

mod field_attributes;
mod filter_ext;
mod result_into_stream_ext;
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
    structure.try_retain(|binding| {
        let field_attributes = parse_field_attributes(binding)?;

        Ok(field_attributes.skip_mode != SkipMode::Always)
    })?;

    Ok(())
}

fn generate_match_arm_body(variant: &VariantInfo) -> Result<TokenStream> {
    let name = variant.ast().ident.to_string();
    let debug_builder = match variant.ast().fields {
        Fields::Named(_) | Fields::Unit => quote! { debug_struct },
        Fields::Unnamed(_) => quote! { debug_tuple },
    };
    let mut debug_builder_calls = Vec::new();

    for binding in variant.bindings() {
        let field_attributes = parse_field_attributes(binding)?;

        let debug_builder_call = match &field_attributes.skip_mode {
            SkipMode::Default => generate_debug_builder_call(binding, &field_attributes)?,
            SkipMode::Condition(condition) => {
                let debug_builder_call = generate_debug_builder_call(binding, &field_attributes)?;

                quote! {
                    if (!#condition(#binding)) {
                        #debug_builder_call
                    }
                }
            }
            SkipMode::Always => quote! {},
        };

        debug_builder_calls.push(debug_builder_call);
    }

    Ok(quote! {
        let mut debug_builder = fmt.#debug_builder(#name);

        #(#debug_builder_calls)*

        debug_builder.finish()
    })
}

fn generate_debug_builder_call(
    binding: &BindingInfo,
    field_attributes: &FieldAttributes,
) -> Result<TokenStream> {
    let format = generate_debug_impl(binding, &field_attributes.debug_format);

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

fn generate_debug_impl(binding: &BindingInfo, debug_format: &DebugFormat) -> TokenStream {
    match debug_format {
        DebugFormat::Default => quote! { #binding },
        DebugFormat::Format(format) => quote! { &format_args!(#format, #binding) },
        DebugFormat::With(with) => quote! {
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
                    fmt: #with,
                }
            }
        },
    }
}

fn parse_field_attributes(binding: &BindingInfo<'_>) -> Result<FieldAttributes> {
    let mut combined_field_attributes = FieldAttributes::default();

    for attr in &binding.ast().attrs {
        if !attr.path().is_ident("debug") {
            continue;
        }

        let field_attributes = FieldAttributes::from_meta(&attr.meta)?;

        combined_field_attributes = combined_field_attributes.try_combine(field_attributes)?;
    }

    Ok(combined_field_attributes)
}
