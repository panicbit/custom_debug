extern crate proc_macro2;
extern crate syn;
#[macro_use] extern crate synstructure;

use proc_macro2::TokenStream;
use synstructure::Structure;
use syn::{Fields, Meta, NestedMeta, parse_str};

decl_derive!([CustomDebug, attributes(debug)] => custom_debug_derive);

fn custom_debug_derive(s: Structure) -> TokenStream {
    let name = s.ast().ident.to_string();
    let debug_attr = parse_str("debug").unwrap();

    let variants = s.each_variant(|variant| {
        let debug_helper = match variant.ast().fields {
            | Fields::Named(_)
            | Fields::Unit => quote! { debug_struct },
            | Fields::Unnamed(_) => quote! { debug_tuple },
        };

        let variant_body = variant.bindings().iter().map(|b| {
            let mut custom_format = None;

            b.ast().attrs.iter()
            .filter(|attr| attr.path == debug_attr)
            .flat_map(|attr| attr.interpret_meta())
            .flat_map(|meta| match meta {
                Meta::List(list) => list.nested,
                _ => panic!("Invalid debug attribute"),
            })
            .for_each(|meta| match meta {
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    match &*nv.ident.to_string() {
                        "format" => custom_format = Some(nv.lit),
                        name => panic!("Unknown key '{}'", name),
                    }
                },
                _ => panic!("Invalid debug attribute"),
            });

            let value = match custom_format {
                None => quote! { #b },
                Some(format) => quote! { &format_args!(#format, #b) },
            };

            if let Some(ref name) = b.ast().ident.as_ref().map(<_>::to_string) {
                quote! {
                    s.field(#name, #value);
                }
            } else {
                quote! {
                    s.field(#value);
                }
            }
        });

        quote! {
            let mut s = f.#debug_helper(#name);
            #(#variant_body)*
            s.finish()
        }
    });

    s.gen_impl(quote! {
        gen impl ::std::fmt::Debug for @Self {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match self {
                    #variants
                }
            }
        }
    })
}

#[test]
fn test_default_struct() {
    test_derive! {
        custom_debug_derive {
            struct Point {
                x: f32,
                y: f32,
            }
        }

        expands to {
            #[allow(non_upper_case_globals)]
            const _DERIVE_std_fmt_Debug_FOR_Point: () = {
                impl ::std::fmt::Debug for Point {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        match self {
                            Point { x: ref __binding_0, y: ref __binding_1, } => {
                                let mut s = f.debug_struct("Point");
                                s.field("x", __binding_0);
                                s.field("y", __binding_1);
                                s.finish()
                            }
                        }
                    }
                }
            };
        }
    }
}

#[test]
fn test_format() {
    test_derive! {
        custom_debug_derive {
            struct Point {
                #[debug(format = "{:.02}")]
                x: f32,
                y: f32,
            }
        }

        expands to {
            #[allow(non_upper_case_globals)]
            const _DERIVE_std_fmt_Debug_FOR_Point: () = {
                impl ::std::fmt::Debug for Point {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        match self {
                            Point { x: ref __binding_0, y: ref __binding_1, } => {
                                let mut s = f.debug_struct("Point");
                                s.field("x", &format_args!("{:.02}", __binding_0));
                                s.field("y", __binding_1);
                                s.finish()
                            }
                        }
                    }
                }
            };
        }

        no_build
    }
}

#[test]
fn test_enum() {
    test_derive! {
        custom_debug_derive {
            enum Foo {
                Bar(#[debug(format = "{}i32")] i32, String),
                Quux { x: f32, y: f32 },
            }
        }

        expands to {
            #[allow(non_upper_case_globals)]
            const _DERIVE_std_fmt_Debug_FOR_Foo: () = {
                impl ::std::fmt::Debug for Foo {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        match self {
                            Foo::Bar(ref __binding_0, ref __binding_1,) => {
                                let mut s = f.debug_tuple("Foo");
                                s.field(&format_args!("{}i32", __binding_0));
                                s.field(__binding_1);
                                s.finish()
                            }
                            Foo::Quux { x: ref __binding_0, y: ref __binding_1, } => {
                                let mut s = f.debug_struct("Foo");
                                s.field("x", __binding_0);
                                s.field("y", __binding_1);
                                s.finish()
                            }
                        }
                    }
                }
            };
        }

        no_build
    }
}
