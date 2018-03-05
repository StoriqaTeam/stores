//! Authorization module contains authorization logic for the repo layer app
use std::rc::Rc;
use std::collections::HashMap;

use stq_acl::{Acl, RolesCache, WithScope};
use models::authorization::*;
use repos::error::RepoError;
use repos::types::DbConnection;

pub fn check(
    acl: &Acl<Resource, Action, Scope, RepoError>,
    resource: &Resource,
    action: &Action,
    resources_with_scope: &[&WithScope<Scope>],
    conn: Option<&DbConnection>,
) -> Result<(), RepoError> {
    acl.allows(resource, action, resources_with_scope, conn)
        .and_then(|allowed| {
            if allowed {
                Ok(())
            } else {
                Err(RepoError::Unauthorized(*resource, *action))
            }
        })
}

pub type BoxedAcl = Box<Acl<Resource, Action, Scope, RepoError>>;

/// ApplicationAcl contains main logic for manipulation with recources
// TODO: remove info about deleted user from cache
#[derive(Clone)]
pub struct ApplicationAcl<R: RolesCache> {
    acls: Rc<HashMap<Role, Vec<Permission>>>,
    roles_cache: R,
    user_id: i32,
}

impl<R: RolesCache> ApplicationAcl<R> {
    pub fn new(roles_cache: R, user_id: i32) -> Self {
        let mut hash = ::std::collections::HashMap::new();
        hash.insert(
            Role::Superuser,
            vec![
                permission!(Resource::Stores),
                permission!(Resource::Products),
                permission!(Resource::UserRoles),
                permission!(Resource::ProductAttrs),
                permission!(Resource::Attributes)
            ],
        );
        hash.insert(
            Role::User,
            vec![
                permission!(Resource::Stores, Action::Read),
                permission!(Resource::Stores, Action::All, Scope::Owned),
                permission!(Resource::Products, Action::Read),
                permission!(Resource::Products, Action::All, Scope::Owned),
                permission!(Resource::UserRoles, Action::Read, Scope::Owned),
                permission!(Resource::ProductAttrs, Action::Read),
                permission!(Resource::ProductAttrs, Action::All, Scope::Owned),
            ],
        );

        ApplicationAcl {
            acls: Rc::new(hash),
            roles_cache: roles_cache,
            user_id: user_id,
        }
    }
}

impl<R: RolesCache<Role = Role, Error = RepoError>> Acl<Resource, Action, Scope, RepoError> for ApplicationAcl<R> {
    fn allows(
        &self,
        resource: &Resource,
        action: &Action,
        resources_with_scope: &[&WithScope<Scope>],
        conn: Option<&DbConnection>,
    ) -> Result<bool, RepoError> {
        let empty: Vec<Permission> = Vec::new();
        let user_id = &self.user_id;
        let hashed_acls = self.acls.clone();
        self.roles_cache.get(*user_id, conn).and_then(|vec| {
            let acls = vec.into_iter()
                .flat_map(|role| hashed_acls.get(&role).unwrap_or(&empty))
                .filter(|permission| {
                    (permission.resource == *resource) && ((permission.action == *action) || (permission.action == Action::All))
                })
                .filter(|permission| {
                    resources_with_scope
                        .into_iter()
                        .all(|res| res.is_in_scope(&permission.scope, *user_id, conn))
                });

            Ok(acls.count() > 0)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use models::authorization::*;
    use repos::*;
    use models::*;

    struct CacheRolesMock {}

    impl RolesCache for CacheRolesMock {
        fn get(&mut self, id: i32, _con: Option<&DbConnection>) -> RepoResult<Vec<Role>> {
            match id {
                1 => Ok(vec![Role::Superuser]),
                _ => Ok(vec![Role::User]),
            }
        }
    }

    const MOCK_USER_ROLE: CacheRolesMock = CacheRolesMock {};

    fn create_store() -> Store {
        Store {
            id: 1,
            user_id: 1,
            name: "name".to_string(),
            is_active: true,
            currency_id: 1,
            short_description: "short description".to_string(),
            long_description: None,
            slug: "myname".to_string(),
            cover: None,
            logo: None,
            phone: "1234567".to_string(),
            email: "example@mail.com".to_string(),
            address: "town city street".to_string(),
            facebook_url: None,
            twitter_url: None,
            instagram_url: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }

    #[test]
    fn test_super_user_for_users() {
        let mut acl = ApplicationAcl::new(MOCK_USER_ROLE, 1);

        let resource = create_store();

        let resources = vec![&resource as &WithScope];

        assert_eq!(
            acl.can(Resource::Products, Action::All, resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.can(Resource::Products, Action::Read, resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.can(Resource::Products, Action::Create, resources.clone(), None)
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_ordinary_user_for_users() {
        let mut acl = ApplicationAcl::new(MOCK_USER_ROLE, 2);

        let resource = create_store();
        let resources = vec![&resource as &WithScope];

        assert_eq!(
            acl.can(Resource::Products, Action::All, resources.clone(), None)
                .unwrap(),
            false
        );
        assert_eq!(
            acl.can(Resource::Products, Action::Read, resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.can(Resource::Products, Action::Create, resources.clone(), None)
                .unwrap(),
            false
        );
    }

    #[test]
    fn test_super_user_for_user_roles() {
        let mut acl = ApplicationAcl::new(MOCK_USER_ROLE, 1);

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };
        let resources = vec![&resource as &WithScope];

        assert_eq!(
            acl.can(Resource::UserRoles, Action::All, resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.can(Resource::UserRoles, Action::Read, resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.can(Resource::UserRoles, Action::Create, resources.clone(), None)
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_user_for_user_roles() {
        let mut acl = ApplicationAcl::new(MOCK_USER_ROLE, 2);

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };
        let resources = vec![&resource as &WithScope];

        assert_eq!(
            acl.can(Resource::UserRoles, Action::All, resources.clone(), None)
                .unwrap(),
            false
        );
        assert_eq!(
            acl.can(Resource::UserRoles, Action::Read, resources.clone(), None)
                .unwrap(),
            false
        );
        assert_eq!(
            acl.can(Resource::UserRoles, Action::Create, resources.clone(), None)
                .unwrap(),
            false
        );
    }

}
