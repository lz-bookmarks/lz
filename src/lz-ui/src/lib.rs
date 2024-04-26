use bounce::prelude::*;
use yew::prelude::*;
pub mod components;
pub mod route;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}", .0)]
pub(crate) struct GoddamnIt(String);

impl GoddamnIt {
    pub(crate) fn new<E>(error: E) -> Self
    where
        E: ToString,
    {
        GoddamnIt(error.to_string())
    }
}

/// Dispatch a [`Slice`] reducer and return a callback that applies
/// the reducer.
pub(crate) fn dispatch_callback<
    T: Slice + Reducible + 'static,
    U,
    F: Fn(U) -> <T as Slice>::Action + 'static,
>(
    slice: &UseSliceHandle<T>,
    reducer: F,
) -> Callback<U> {
    let slice = slice.clone();
    Callback::from(move |new_value| slice.dispatch(reducer(new_value)))
}
