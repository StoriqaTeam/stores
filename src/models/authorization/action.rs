//! Action enum for authorization
use std::fmt;

// All gives all permissions.
// Read - read resource with id,
// Create - create resource with id.
// Update - update resource with id.
// Delete - delete resource with id.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    All,
    Read,
    Create,
    Update,
    Delete,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Action::All => write!(f, "all"),
            Action::Read => write!(f, "read"),
            Action::Create => write!(f, "create"),
            Action::Update => write!(f, "update"),
            Action::Delete => write!(f, "delete"),
        }
    }
}
