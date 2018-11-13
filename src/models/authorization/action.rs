//! Action enum for authorization
use std::fmt;

// All gives all permissions.
// Read - read resource with id,
// ReadUnPublished - read unpublished resources
// Create - create resource with id.
// Update - update resource with id.
// Delete - delete resource with id.
// Moderate - moderation resources
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    All,
    Read,
    ReadUnPublished,
    Create,
    Update,
    Delete,
    Moderate,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Action::All => write!(f, "all"),
            Action::Read => write!(f, "read"),
            Action::ReadUnPublished => write!(f, "read unpublished"),
            Action::Create => write!(f, "create"),
            Action::Update => write!(f, "update"),
            Action::Delete => write!(f, "delete"),
            Action::Moderate => write!(f, "moderate"),
        }
    }
}
