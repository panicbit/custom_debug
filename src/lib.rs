#![no_std]
use core::fmt;

pub use custom_debug_derive::*;

/// Formats a buffer as hex using \xNN notation.
pub fn hexbuf(v: &impl AsRef<[u8]>, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "b\"")?;

    for x in v.as_ref() {
        write!(f, "\\x{:02x}", x)?;
    }

    write!(f, "\"")?;

    Ok(())
}

/// Formats a buffer as hex using \xNN notation,
/// except for printable ascii characters.
pub fn hexbuf_str(v: &impl AsRef<[u8]>, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "b\"")?;

    for x in v.as_ref() {
        match x {
            b'\\' => write!(f, "\\\\")?,
            b'"' => write!(f, "\\\"")?,
            b if b.is_ascii_graphic() => write!(f, "{}", *x as char)?,
            _ => write!(f, "\\x{:02x}", x)?,
        }
    }

    write!(f, "\"")?;

    Ok(())
}
