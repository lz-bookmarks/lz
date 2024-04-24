pub mod components;
pub mod js;
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
