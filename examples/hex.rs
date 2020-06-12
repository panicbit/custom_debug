use custom_debug::Debug;
use std::fmt;

#[derive(Debug)]
struct Foo {
    #[debug(format = "{} things")]
    x: i32,
    #[debug(skip)]
    y: i32,
    #[debug(with = "hex_fmt")]
    z: i32,
}

fn hex_fmt<T: fmt::Debug>(n: &T, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "0x{:02X?}", n)
}

fn main() {
    let foo = Foo {
        x: 42,
        y: 123,
        z: 171,
    };

    println!("{:#?}", foo);
    println!("Hidden field 'y': {}", foo.y);
}
