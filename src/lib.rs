extern crate proc_macro2;
extern crate syn;
#[macro_use] extern crate synstructure;

use proc_macro2::TokenStream;
use synstructure::Structure;
use syn::{Fields, Meta, NestedMeta, Lit, Path, Ident, parse_str};

decl_derive!([CustomDebug, attributes(debug)] => custom_debug_derive);

fn custom_debug_derive(s: Structure) -> TokenStream {
    let name = s.ast().ident.to_string();
    let debug_attr = parse_str::<Path>("debug").unwrap();
    let skip_ident = parse_str::<Ident>("skip").unwrap();

    let variants = s.each_variant(|variant| {
        let debug_helper = match variant.ast().fields {
            | Fields::Named(_)
            | Fields::Unit => quote! { debug_struct },
            | Fields::Unnamed(_) => quote! { debug_tuple },
        };

        let variant_body = variant.bindings().iter().map(|b| {
            let mut format = None;

            let metas = b.ast().attrs.iter()
            .filter(|attr| attr.path == debug_attr)
            .flat_map(|attr| attr.interpret_meta())
            .flat_map(|meta| match meta {
                Meta::List(list) => list.nested,
                _ => panic!("Invalid debug attribute"),
            });

            for meta in metas {
                match meta {
                    NestedMeta::Meta(Meta::Word(ref ident)) if ident == &skip_ident => return quote! {},
                    NestedMeta::Meta(Meta::NameValue(nv)) => {
                        let value = nv.lit;
                        format = Some(match &*nv.ident.to_string() {
                            "format" => quote! { &format_args!(#value, #b) },
                            "with" => match value {
                                Lit::Str(fun) => {
                                    let fun = fun.parse::<Path>().unwrap();
                                    quote! {
                                        &{
                                            struct DebugWith<'a, T: 'a> {
                                                data: &'a T,
                                                fmt: fn(&T, &mut ::std::fmt::Formatter) -> ::std::fmt::Result,
                                            }

                                            impl<'a, T: 'a> ::std::fmt::Debug for DebugWith<'a, T> {
                                                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                                    (self.fmt)(self.data, f)
                                                }
                                            }

                                            DebugWith {
                                                data: #b,
                                                fmt: #fun,
                                            }
                                        }
                                    }
                                },
                                _ => panic!("Invalid 'with' value"),
                            },
                            name => panic!("Unknown key '{}'", name),
                        })
                    },
                    _ => panic!("Invalid debug attribute"),
                }
            }

            let format = format.unwrap_or_else(|| quote! { #b });

            if let Some(ref name) = b.ast().ident.as_ref().map(<_>::to_string) {
                quote! {
                    s.field(#name, #format);
                }
            } else {
                quote! {
                    s.field(#format);
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
fn test_with() {
    test_derive! {
        custom_debug_derive {
            struct Point {
                #[debug(with = "my_fmt")]
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
                                s.field("x", &{
                                    struct DebugWith<'a, T: 'a> {
                                        data: &'a T,
                                        fmt: fn(&T, &mut ::std::fmt::Formatter) -> ::std::fmt::Result,
                                    }

                                    impl<'a, T: 'a> ::std::fmt::Debug for DebugWith<'a, T> {
                                        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                            (self.fmt)(self.data, f)
                                        }
                                    }

                                    DebugWith {
                                        data: __binding_0,
                                        fmt: my_fmt,
                                    }
                                });
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
fn test_skip() {
    test_derive! {
        custom_debug_derive {
            struct Point {
                x: f32,
                #[debug(skip)]
                y: f32,
                z: f32,
            }
        }

        expands to {
            #[allow(non_upper_case_globals)]
            const _DERIVE_std_fmt_Debug_FOR_Point: () = {
                impl ::std::fmt::Debug for Point {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        match self {
                            Point { x: ref __binding_0, y: ref __binding_1, z: ref __binding_2, } => {
                                let mut s = f.debug_struct("Point");
                                s.field("x", __binding_0);
                                s.field("z", __binding_2);
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
