//! Repo for user_roles table. UserRole is an entity that connects
//! users and roles. I.e. this table is for user has-many roles
//! relationship

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;

use stq_types::{StoresRole, UserId};

use repos::legacy_acl::*;

use super::types::RepoResult;
use models::authorization::*;
use models::{NewUserRole, OldUserRole, UserRole};
use repos::RolesCacheImpl;
use schema::user_roles::dsl::*;

/// UserRoles repository for handling UserRoles
pub trait UserRolesRepo {
    /// Returns list of user_roles for a specific user
    fn list_for_user(&self, user_id: UserId) -> RepoResult<Vec<StoresRole>>;

    /// Create a new user role
    fn create(&self, payload: NewUserRole) -> RepoResult<UserRole>;

    /// Delete role of a user
    fn delete(&self, payload: OldUserRole) -> RepoResult<UserRole>;

    /// Delete user roles by user id
    fn delete_by_user_id(&self, user_id_arg: UserId) -> RepoResult<UserRole>;
}

/// Implementation of UserRoles trait
pub struct UserRolesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, UserRole>>,
    pub cached_roles: RolesCacheImpl,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserRolesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, UserRole>>, cached_roles: RolesCacheImpl) -> Self {
        Self {
            db_conn,
            acl,
            cached_roles,
        }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserRolesRepo for UserRolesRepoImpl<'a, T> {
    fn list_for_user(&self, user_id_value: UserId) -> RepoResult<Vec<StoresRole>> {
        debug!("list user roles for id {}.", user_id_value);
        if self.cached_roles.contains(user_id_value) {
            let roles = self.cached_roles.get(user_id_value);
            Ok(roles)
        } else {
            let query = user_roles.filter(user_id.eq(user_id_value));
            query
                .get_results::<UserRole>(self.db_conn)
                .and_then(|user_roles_arg| {
                    let roles = user_roles_arg
                        .into_iter()
                        .map(|user_role| user_role.role)
                        .collect::<Vec<StoresRole>>();
                    Ok(roles)
                })
                .and_then(|roles| {
                    if !roles.is_empty() {
                        self.cached_roles.add_roles(user_id_value, &roles);
                    }
                    Ok(roles)
                })
                .map_err(|e| e.context(format!("List user roles for id {} error occured.", user_id_value)).into())
        }
    }

    fn create(&self, payload: NewUserRole) -> RepoResult<UserRole> {
        debug!("create new user role {:?}.", payload);
        self.cached_roles.remove(payload.user_id);
        let query = diesel::insert_into(user_roles).values(&payload);
        query
            .get_result(self.db_conn)
            .map_err(|e| e.context(format!("create new user role {:?}.", payload)).into())
    }

    fn delete(&self, payload: OldUserRole) -> RepoResult<UserRole> {
        debug!("delete user role {:?}.", payload);
        self.cached_roles.remove(payload.user_id);
        let filtered = user_roles.filter(user_id.eq(payload.user_id)).filter(role.eq(payload.role));
        let query = diesel::delete(filtered);
        query
            .get_result(self.db_conn)
            .map_err(move |e| e.context(format!("delete user role {:?}.", payload)).into())
    }

    fn delete_by_user_id(&self, user_id_arg: UserId) -> RepoResult<UserRole> {
        debug!("delete user role by id {}.", user_id_arg);
        self.cached_roles.remove(user_id_arg);
        let filtered = user_roles.filter(user_id.eq(user_id_arg));
        let query = diesel::delete(filtered);
        query
            .get_result(self.db_conn)
            .map_err(|e| e.context(format!("delete user role by id {}.", user_id_arg)).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, UserRole>
    for UserRolesRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&UserRole>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(user_role) = obj {
                    user_role.user_id == user_id_arg
                } else {
                    false
                }
            }
        }
    }
}
