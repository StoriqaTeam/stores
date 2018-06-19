//! Repos is a module responsible for interacting with access control lists
//! Authorization module contains authorization logic for the repo layer app

#[macro_use]
pub mod macros;
pub mod legacy_acl;
pub mod roles_cache;

pub use self::roles_cache::RolesCacheImpl;

use std::collections::HashMap;
use std::rc::Rc;

use errors::Error;
use failure::Error as FailureError;

use self::legacy_acl::{Acl, CheckScope};

use models::authorization::*;

pub fn check<T>(
    acl: &Acl<Resource, Action, Scope, FailureError, T>,
    resource: &Resource,
    action: &Action,
    scope_checker: &CheckScope<Scope, T>,
    obj: Option<&T>,
) -> Result<(), FailureError> {
    acl.allows(resource, action, scope_checker, obj).and_then(|allowed| {
        if allowed {
            Ok(())
        } else {
            Err(format_err!("Denied request to do {:?} on {:?}", action, resource)
                .context(Error::Forbidden)
                .into())
        }
    })
}

/// ApplicationAcl contains main logic for manipulation with recources
#[derive(Clone)]
pub struct ApplicationAcl {
    acls: Rc<HashMap<Role, Vec<Permission>>>,
    roles: Vec<Role>,
    user_id: i32,
}

impl ApplicationAcl {
    pub fn new(roles: Vec<Role>, user_id: i32) -> Self {
        let mut hash = ::std::collections::HashMap::new();
        hash.insert(
            Role::Superuser,
            vec![
                permission!(Resource::Stores),
                permission!(Resource::Products),
                permission!(Resource::BaseProducts),
                permission!(Resource::UserRoles),
                permission!(Resource::ProductAttrs),
                permission!(Resource::Attributes),
                permission!(Resource::Categories),
                permission!(Resource::CategoryAttrs),
                permission!(Resource::CurrencyExchange),
                permission!(Resource::WizardStores),
                permission!(Resource::ModeratorProductComments),
                permission!(Resource::ModeratorStoreComments),
            ],
        );
        hash.insert(
            Role::User,
            vec![
                permission!(Resource::Stores, Action::Read),
                permission!(Resource::Stores, Action::All, Scope::Owned),
                permission!(Resource::Products, Action::Read),
                permission!(Resource::Products, Action::All, Scope::Owned),
                permission!(Resource::BaseProducts, Action::Read),
                permission!(Resource::BaseProducts, Action::All, Scope::Owned),
                permission!(Resource::UserRoles, Action::Read, Scope::Owned),
                permission!(Resource::ProductAttrs, Action::Read),
                permission!(Resource::ProductAttrs, Action::All, Scope::Owned),
                permission!(Resource::WizardStores, Action::Read),
                permission!(Resource::WizardStores, Action::All, Scope::Owned),
                permission!(Resource::ModeratorProductComments, Action::Read),
                permission!(Resource::ModeratorProductComments, Action::All, Scope::Owned),
                permission!(Resource::ModeratorStoreComments, Action::Read),
                permission!(Resource::ModeratorStoreComments, Action::All, Scope::Owned),
                permission!(Resource::Attributes, Action::Read),
                permission!(Resource::Categories, Action::Read),
                permission!(Resource::CategoryAttrs, Action::Read),
                permission!(Resource::CurrencyExchange, Action::Read),
            ],
        );

        ApplicationAcl {
            acls: Rc::new(hash),
            roles,
            user_id,
        }
    }
}
impl<T> Acl<Resource, Action, Scope, FailureError, T> for ApplicationAcl {
    fn allows(
        &self,
        resource: &Resource,
        action: &Action,
        scope_checker: &CheckScope<Scope, T>,
        obj: Option<&T>,
    ) -> Result<bool, FailureError> {
        let empty: Vec<Permission> = Vec::new();
        let user_id = &self.user_id;
        let hashed_acls = self.acls.clone();
        let acls = self.roles
            .iter()
            .flat_map(|role| hashed_acls.get(role).unwrap_or(&empty))
            .filter(|permission| {
                (permission.resource == *resource) && ((permission.action == *action) || (permission.action == Action::All))
            })
            .filter(|permission| scope_checker.is_in_scope(*user_id, &permission.scope, obj));

        if acls.count() > 0 {
            Ok(true)
        } else {
            error!("Denied request from user {} to do {} on {}.", user_id, action, resource);
            Ok(false)
        }
    }
}

/// UnauthorizedAcl contains main logic for manipulation with recources
#[derive(Clone, Default)]
pub struct UnauthorizedAcl;

impl<T> Acl<Resource, Action, Scope, FailureError, T> for UnauthorizedAcl {
    fn allows(
        &self,
        resource: &Resource,
        action: &Action,
        _scope_checker: &CheckScope<Scope, T>,
        _obj: Option<&T>,
    ) -> Result<bool, FailureError> {
        if *action == Action::Read {
            match *resource {
                Resource::Categories
                | Resource::Stores
                | Resource::Products
                | Resource::BaseProducts
                | Resource::ProductAttrs
                | Resource::Attributes
                | Resource::CurrencyExchange
                | Resource::WizardStores
                | Resource::ModeratorProductComments
                | Resource::ModeratorStoreComments
                | Resource::CategoryAttrs => Ok(true),
                _ => Ok(false),
            }
        } else {
            error!("Denied unauthorized request to do {} on {}.", action, resource);
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use repos::legacy_acl::{Acl, CheckScope};
    use serde_json;

    use models::*;
    use repos::*;

    fn create_store() -> Store {
        Store {
            id: 1,
            user_id: 1,
            name: serde_json::from_str("{}").unwrap(),
            is_active: true,
            short_description: serde_json::from_str("{}").unwrap(),
            long_description: None,
            slug: "myname".to_string(),
            cover: None,
            logo: None,
            phone: Some("1234567".to_string()),
            email: Some("example@mail.com".to_string()),
            address: Some("town city street".to_string()),
            default_language: "en".to_string(),
            slogan: Some("fdsf".to_string()),
            facebook_url: None,
            twitter_url: None,
            instagram_url: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            country: None,
            rating: 0f64,
            product_categories: Some(serde_json::from_str("{}").unwrap()),
            status: Status::Published,
            administrative_area_level_1: None,
            administrative_area_level_2: None,
            locality: None,
            political: None,
            postal_code: None,
            route: None,
            street_number: None,
            place_id: None,
        }
    }

    #[derive(Default)]
    struct ScopeChecker;

    impl CheckScope<Scope, Store> for ScopeChecker {
        fn is_in_scope(&self, user_id: i32, scope: &Scope, obj: Option<&Store>) -> bool {
            match *scope {
                Scope::All => true,
                Scope::Owned => {
                    if let Some(store) = obj {
                        store.user_id == user_id
                    } else {
                        false
                    }
                }
            }
        }
    }

    impl CheckScope<Scope, UserRole> for ScopeChecker {
        fn is_in_scope(&self, user_id: i32, scope: &Scope, obj: Option<&UserRole>) -> bool {
            match *scope {
                Scope::All => true,
                Scope::Owned => {
                    if let Some(user_role) = obj {
                        user_role.user_id == user_id
                    } else {
                        false
                    }
                }
            }
        }
    }

    #[test]
    fn test_super_user_for_stores() {
        let acl = ApplicationAcl::new(vec![Role::Superuser], 1232);
        let s = ScopeChecker::default();
        let resource = create_store();
        assert_eq!(acl.allows(&Resource::Stores, &Action::All, &s, Some(&resource)).unwrap(), true);
        assert_eq!(acl.allows(&Resource::Stores, &Action::Read, &s, Some(&resource)).unwrap(), true);
        assert_eq!(acl.allows(&Resource::Stores, &Action::Create, &s, Some(&resource)).unwrap(), true);
    }

    #[test]
    fn test_ordinary_user_for_store() {
        let acl = ApplicationAcl::new(vec![Role::User], 2);
        let s = ScopeChecker::default();
        let resource = create_store();

        assert_eq!(acl.allows(&Resource::Stores, &Action::All, &s, Some(&resource)).unwrap(), false);
        assert_eq!(acl.allows(&Resource::Stores, &Action::Read, &s, Some(&resource)).unwrap(), true);
        assert_eq!(acl.allows(&Resource::Stores, &Action::Create, &s, Some(&resource)).unwrap(), false);
    }

    #[test]
    fn test_super_user_for_user_roles() {
        let acl = ApplicationAcl::new(vec![Role::Superuser], 1232);
        let s = ScopeChecker::default();

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };

        assert_eq!(acl.allows(&Resource::UserRoles, &Action::All, &s, Some(&resource)).unwrap(), true);
        assert_eq!(acl.allows(&Resource::UserRoles, &Action::Read, &s, Some(&resource)).unwrap(), true);
        assert_eq!(
            acl.allows(&Resource::UserRoles, &Action::Create, &s, Some(&resource)).unwrap(),
            true
        );
    }

    #[test]
    fn test_user_for_user_roles() {
        let acl = ApplicationAcl::new(vec![Role::User], 2);
        let s = ScopeChecker::default();

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };

        assert_eq!(acl.allows(&Resource::UserRoles, &Action::All, &s, Some(&resource)).unwrap(), false);
        assert_eq!(acl.allows(&Resource::UserRoles, &Action::Read, &s, Some(&resource)).unwrap(), false);
        assert_eq!(
            acl.allows(&Resource::UserRoles, &Action::Create, &s, Some(&resource)).unwrap(),
            false
        );
    }

}
