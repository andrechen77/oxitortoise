use crate::value::{ContainedInValue, PolyValue};

pub(crate) fn unwrap_option<T>(option: Option<T>) -> T {
    #[cfg(debug_assertions)]
    return option.unwrap();

    #[cfg(not(debug_assertions))]
    return option.unwrap_unchecked();
}

pub(crate) fn unwrap_polyvalue<T: ContainedInValue>(polyvalue: PolyValue) -> T {
    #[cfg(debug_assertions)]
    return polyvalue.into().unwrap();

    #[cfg(not(debug_assertions))]
    return polyvalue.into().unwrap_unchecked(); // TODO or use into_unchecked
}
