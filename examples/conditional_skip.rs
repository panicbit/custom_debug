#![allow(clippy::disallowed_names)]
use core::fmt;

use custom_debug::Debug;

#[derive(Debug)]
struct Foo {
    x: i32,
    #[debug(
        skip_if = Option::is_none,
        with = strip_some,
    )]
    y: Option<i32>,
    z: i32,
}

fn main() {
    let mut foo = Foo {
        x: 42,
        y: None,
        z: 171,
    };

    println!("With `y = None`:");
    println!("{:#?}", foo);

    foo.y = Some(123);
    println!("With `y = Some(123)`:");
    println!("{:#?}", foo);
}

fn strip_some<T: fmt::Debug>(value: &Option<T>, f: &mut fmt::Formatter) -> fmt::Result {
    if let Some(value) = value {
        value.fmt(f)?;
    }

    Ok(())
}
