use darling::util::Flag;
use darling::FromMeta;
use syn::ExprPath;

#[derive(Default)]
pub struct FieldAttributes {
    pub skip_mode: SkipMode,
    pub debug_format: DebugFormat,
}

impl FieldAttributes {
    fn new(internal: InternalFieldAttributes) -> darling::Result<Self> {
        let mut skip_mode = SkipMode::Default;
        let mut debug_format = DebugFormat::Default;

        if internal.skip.is_present() {
            skip_mode = skip_mode.try_combine(SkipMode::Always)?;
        }

        if let Some(skip_if) = internal.skip_if {
            skip_mode = skip_mode.try_combine(SkipMode::Condition(skip_if))?;
        }

        if let Some(format) = internal.format {
            debug_format = debug_format.try_combine(DebugFormat::Format(format))?;
        }

        if let Some(with) = internal.with {
            debug_format = debug_format.try_combine(DebugFormat::With(with))?;
        }

        Ok(Self {
            skip_mode,
            debug_format,
        })
    }

    pub fn try_combine(self, other: Self) -> darling::Result<Self> {
        let skip_mode = self.skip_mode.try_combine(other.skip_mode)?;
        let debug_format = self.debug_format.try_combine(other.debug_format)?;

        Ok(Self {
            skip_mode,
            debug_format,
        })
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

#[derive(Default, PartialEq, Eq)]
pub enum SkipMode {
    #[default]
    Default,
    Condition(ExprPath),
    Always,
}

impl SkipMode {
    fn try_combine(self, other: Self) -> darling::Result<Self> {
        match (&self, &other) {
            (SkipMode::Default, _) => Ok(other),
            (_, SkipMode::Default) => Ok(self),
            _ => Err(conflicting_skip_options_error()),
        }
    }
}

#[derive(FromMeta)]
struct InternalFieldAttributes {
    skip: Flag,
    skip_if: Option<ExprPath>,
    format: Option<String>,
    with: Option<ExprPath>,
}

fn conflicting_skip_options_error() -> darling::Error {
    darling::Error::custom("Conflicting skip options")
}

fn conflicting_format_options_error() -> darling::Error {
    darling::Error::custom("Conflicting format options")
}
