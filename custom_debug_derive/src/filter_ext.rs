use synstructure::BindingInfo;

use synstructure::Structure;

use syn::Result;

pub(crate) trait FilterExt {
    type Item<'a>;

    fn filter<F>(&mut self, f: F) -> &mut Self
    where
        F: for<'a> FnMut(Self::Item<'a>) -> bool;

    fn try_filter<F>(&mut self, mut f: F) -> Result<&mut Self>
    where
        F: for<'a> FnMut(Self::Item<'a>) -> Result<bool>,
    {
        let mut filter_err = None;

        let result = self.filter(|value| {
            if filter_err.is_some() {
                return false;
            }

            f(value).unwrap_or_else(|err| {
                filter_err = Some(err);
                false
            })
        });

        filter_err.map(Err).unwrap_or(Ok(result))
    }
}

impl FilterExt for Structure<'_> {
    type Item<'a> = &'a BindingInfo<'a>;

    fn filter<F>(&mut self, mut f: F) -> &mut Self
    where
        F: for<'a> FnMut(&'a BindingInfo<'a>) -> bool,
    {
        self.filter(|value| f(value))
    }
}
