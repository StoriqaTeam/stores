//! enum for authorization
use std::fmt;

use stq_static_resources::ModerationStatus;

// Any - gives all permissions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rule {
    Any,
    ModerationStatus(ModerationStatus),
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Rule::Any => write!(f, "any"),
            Rule::ModerationStatus(status) => write!(f, "status: {}", status),
        }
    }
}
