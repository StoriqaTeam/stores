//! Permission is a tuple for describing permissions

use models::{Action, Resource, Rule, Scope};

pub struct Permission {
    pub resource: Resource,
    pub action: Action,
    pub scope: Scope,
    pub rule: Rule,
}
