# custom_debug

Derive `Debug` with a custom format per field.

# Example usage

Here is a showcase of `custom_debug`s features:

```rust
    use custom_debug::Debug;
    use std::fmt;

    #[derive(Debug)]
    struct Foo {
        #[debug(format = "{} things")]
        x: i32,
        #[debug(skip)]
        y: i32,
        #[debug(with = hex_fmt)]
        z: i32,
        #[debug(skip_if = Option::is_none)]
        label: Option<String>,
    }

    fn hex_fmt<T: fmt::Debug>(n: &T, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:02X?}", n)
    }
```

The resulting debug output would look something like this:

```
Foo {
    x: 42 things,
    z: 0xAB
}
```

# Field attributes reference

Attributes within a section below are considered mutually exclusive.

## Skip attributes

| | |
|-|-|
| `skip` | Unconditionally skips a field. |
| `skip_if = path::to::function` | Skips a field if `path::to::function(&field)` returns `true`. |

## Format attributes

| | |
|-|-|
| `format = "format string {}"` | Formats a field using a format string. Must contain a placeholder (`{}`) with modifiers of your choice. |
| `with = path::to::formatter` | Formats a field using `path::to::formatter`. The required signature is `fn(&T, &mut std::fmt::Formatter) -> std::fmt::Result` where `T` is a type compatible with the field's type (i.e. the function can be generic and coercions apply). |
