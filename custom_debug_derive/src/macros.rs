macro_rules! error {
    ($span:expr, $message:expr $(, $($rest:tt),*)?) => {{
        let message = format!(concat!("custom_debug: ", $message) $(, $($rest),*)?);

        syn::Error::new($span, message)
    }};
}

pub(crate) use error;

macro_rules! bail {
    ($span:expr, $message:expr $(, $($rest:tt),*)?) => {
        return Err($crate::error!($span, $message $(, $($rest),*)?))
    };
}

pub(crate) use bail;
