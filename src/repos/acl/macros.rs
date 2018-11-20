//! ACL Macroses

/// Macros used adding permissions to user.
#[macro_export]
macro_rules! permission {
    ($resource:expr) => {
        Permission {
            resource: $resource,
            action: Action::All,
            scope: Scope::All,
            rule: None,
        }
    };
    ($resource:expr, $action:expr) => {
        Permission {
            resource: $resource,
            action: $action,
            scope: Scope::All,
            rule: None,
        }
    };
    ($resource:expr, $action:expr, $scope:expr) => {
        Permission {
            resource: $resource,
            action: $action,
            scope: $scope,
            rule: None,
        }
    };
    ($resource:expr, $action:expr, $scope:expr, $rules:expr) => {
        Permission {
            resource: $resource,
            action: $action,
            scope: $scope,
            rule: Some($rules),
        }
    };
}
