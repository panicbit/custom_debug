# custom_debug_derive

Derive `Debug` with a custom format per field.

# Usage

```rust
    #[macro_use] extern crate custom_debug_derive;

    #[derive(CustomDebug)]
    struct Foo {
        #[debug(format = "{} things")]
        n: i32,
    }
```

Would print something like `Foo { n: 42 things }`
