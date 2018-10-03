# custom_debug_derive

Derive `Debug` with a custom format per field.

# Usage

```rust
    #[macro_use] extern crate custom_debug_derive;
    use std::fmt;

    #[derive(CustomDebug)]
    struct Foo {
        #[debug(format = "{} things")]
        n: i32,
        #[debug(with = "hex_fmt")]
        m: i32,
    }

    fn hex_fmt<T: fmt::Debug>(n: &T, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:02X?}", n)
    }
```

Would print something like

```
Foo {
    n: 42 things,
    m: 0xAB
}
```
