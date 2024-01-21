use std::cell::OnceCell;

use darling::util::Flag;
use darling::FromMeta;
use syn::ExprPath;

#[derive(Default)]
pub struct FieldAttributes {
    pub skip: bool,
    pub debug_format: DebugFormat,
}

impl FieldAttributes {
    fn new(internal: InternalFieldAttributes) -> darling::Result<Self> {
        let skip = internal.skip.is_present();
        let debug_format = OnceCell::new();

        if let Some(format) = internal.format {
            debug_format
                .set(DebugFormat::Format(format))
                .map_err(|_| conflicting_format_options_error())?;
        }

        if let Some(with) = internal.with {
            debug_format
                .set(DebugFormat::With(with))
                .map_err(|_| conflicting_format_options_error())?;
        }

        let debug_format = debug_format.into_inner().unwrap_or(DebugFormat::Default);

        Ok(Self { skip, debug_format })
    }

    pub fn try_combine(self, other: Self) -> darling::Result<Self> {
        let skip = self.skip || other.skip;
        let debug_format = self.debug_format.try_combine(other.debug_format)?;

        Ok(Self { skip, debug_format })
    }
}

impl FromMeta for FieldAttributes {
    fn from_nested_meta(item: &darling::ast::NestedMeta) -> darling::Result<Self> {
        InternalFieldAttributes::from_nested_meta(item).and_then(FieldAttributes::new)
    }

    fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
        InternalFieldAttributes::from_meta(item).and_then(FieldAttributes::new)
    }

    fn from_none() -> Option<Self> {
        InternalFieldAttributes::from_none().and_then(|attrs| FieldAttributes::new(attrs).ok())
    }

    fn from_word() -> darling::Result<Self> {
        InternalFieldAttributes::from_word().and_then(FieldAttributes::new)
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        InternalFieldAttributes::from_list(items).and_then(FieldAttributes::new)
    }
}

#[derive(FromMeta, Debug, PartialEq, Eq, Default)]
pub enum DebugFormat {
    #[default]
    Default,
    Format(String),
    With(ExprPath),
}

impl DebugFormat {
    fn try_combine(self, other: Self) -> darling::Result<Self> {
        match (&self, &other) {
            (DebugFormat::Default, _) => Ok(other),
            (_, DebugFormat::Default) => Ok(self),
            _ => Err(conflicting_format_options_error()),
        }
    }
}

#[derive(FromMeta)]
struct InternalFieldAttributes {
    skip: Flag,
    format: Option<String>,
    with: Option<ExprPath>,
}

fn conflicting_format_options_error() -> darling::Error {
    darling::Error::custom("Conflicting format options")
}
