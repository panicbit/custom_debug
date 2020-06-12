use custom_debug::{Debug, hexbuf, hexbuf_str};

#[derive(Debug)]
struct Foo {
    #[debug(with = "hexbuf")]
    buf1: &'static [u8],
    #[debug(with = "hexbuf_str")]
    buf2: Vec<u8>,
}

fn main() {
    let foo = Foo {
        buf1: b"\0test1\0test2\0",
        buf2: b"\0test1\0test2\0".to_vec(),
    };

    println!("{:#?}", foo);
}
