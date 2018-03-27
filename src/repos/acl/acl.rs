//! Authorization module contains authorization logic for the repo layer app
use std::rc::Rc;
use std::collections::HashMap;

use stq_acl::{Acl, CheckScope};
use models::authorization::*;
use repos::error::RepoError;

pub fn check<T>(
    acl: &Acl<Resource, Action, Scope, RepoError, T>,
    resource: &Resource,
    action: &Action,
    scope_checker: &CheckScope<Scope, T>,
    obj: Option<&T>,
) -> Result<(), RepoError> {
    acl.allows(resource, action, scope_checker, obj)
        .and_then(|allowed| {
            if allowed {
                Ok(())
            } else {
                Err(RepoError::Unauthorized(*resource, *action))
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
                permission!(Resource::Attributes, Action::Read),
                permission!(Resource::Categories, Action::Read),
                permission!(Resource::CategoryAttrs, Action::Read),
            ],
        );

        ApplicationAcl {
            acls: Rc::new(hash),
            roles,
            user_id,
        }
    }
}
impl<T> Acl<Resource, Action, Scope, RepoError, T> for ApplicationAcl {
    fn allows(
        &self,
        resource: &Resource,
        action: &Action,
        scope_checker: &CheckScope<Scope, T>,
        obj: Option<&T>,
    ) -> Result<bool, RepoError> {
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

        Ok(acls.count() > 0)
    }
}

/// UnauthorizedAcl contains main logic for manipulation with recources
#[derive(Clone, Default)]
pub struct UnauthorizedAcl;

impl<T> Acl<Resource, Action, Scope, RepoError, T> for UnauthorizedAcl {
    fn allows(
        &self,
        resource: &Resource,
        action: &Action,
        _scope_checker: &CheckScope<Scope, T>,
        _obj: Option<&T>,
    ) -> Result<bool, RepoError> {
        if *action == Action::Read {
            match *resource {
                Resource::Categories
                | Resource::Stores
                | Resource::Products
                | Resource::BaseProducts
                | Resource::ProductAttrs
                | Resource::Attributes
                | Resource::CategoryAttrs => Ok(true),
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use stq_acl::{Acl, RolesCache, WithScope};
    use serde_json;

    use repos::error::RepoError as Error;
    use repos::*;
    use models::*;

    #[derive(Clone)]
    struct CacheRolesMock;

    impl RolesCache for CacheRolesMock {
        type Role = Role;
        type Error = Error;

        fn get(&self, user_id: i32, _db_conn: Option<&DbConnection>) -> Result<Vec<Self::Role>, Self::Error> {
            match user_id {
                1 => Ok(vec![Role::Superuser]),
                _ => Ok(vec![Role::User]),
            }
        }

        fn clear(&self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn remove(&self, _id: i32) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    const MOCK_USER_ROLE: CacheRolesMock = CacheRolesMock {};

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
        }
    }

    #[test]
    fn test_super_user_for_users() {
        let acl = ApplicationAcl::new(MOCK_USER_ROLE, 1);

        let resource = create_store();

        let resources = vec![&resource as &WithScope<Scope>];

        assert_eq!(
            acl.allows(&Resource::Stores, &Action::All, &resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.allows(&Resource::Stores, &Action::Read, &resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.allows(&Resource::Stores, &Action::Create, &resources.clone(), None)
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_ordinary_user_for_users() {
        let acl = ApplicationAcl::new(MOCK_USER_ROLE, 2);

        let resource = create_store();
        let resources = vec![&resource as &WithScope<Scope>];

        assert_eq!(
            acl.allows(&Resource::Stores, &Action::All, &resources.clone(), None)
                .unwrap(),
            false
        );
        assert_eq!(
            acl.allows(&Resource::Stores, &Action::Read, &resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.allows(&Resource::Stores, &Action::Create, &resources.clone(), None)
                .unwrap(),
            false
        );
    }

    #[test]
    fn test_super_user_for_user_roles() {
        let acl = ApplicationAcl::new(MOCK_USER_ROLE, 1);

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };
        let resources = vec![&resource as &WithScope<Scope>];

        assert_eq!(
            acl.allows(&Resource::UserRoles, &Action::All, &resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.allows(
                &Resource::UserRoles,
                &Action::Read,
                &resources.clone(),
                None
            ).unwrap(),
            true
        );
        assert_eq!(
            acl.allows(
                &Resource::UserRoles,
                &Action::Create,
                &resources.clone(),
                None
            ).unwrap(),
            true
        );
    }

    #[test]
    fn test_user_for_user_roles() {
        let acl = ApplicationAcl::new(MOCK_USER_ROLE, 2);

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };
        let resources = vec![&resource as &WithScope<Scope>];

        assert_eq!(
            acl.allows(&Resource::UserRoles, &Action::All, &resources.clone(), None)
                .unwrap(),
            false
        );
        assert_eq!(
            acl.allows(
                &Resource::UserRoles,
                &Action::Read,
                &resources.clone(),
                None
            ).unwrap(),
            false
        );
        assert_eq!(
            acl.allows(
                &Resource::UserRoles,
                &Action::Create,
                &resources.clone(),
                None
            ).unwrap(),
            false
        );
    }

}
