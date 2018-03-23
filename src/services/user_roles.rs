//! UserRoles Services, presents CRUD operations with user_roles

use futures_cpupool::CpuPool;

use stq_acl::RolesCache;

use models::{NewUserRole, OldUserRole, Role, UserRole};
use super::types::ServiceFuture;
use super::error::ServiceError;
use repos::types::DbPool;
use repos::acl::RolesCacheImpl;
use repos::ReposFactory;

pub trait UserRolesService {
    /// Returns role by user ID
    fn get_roles(&self, user_id: i32) -> ServiceFuture<Vec<Role>>;
    /// Delete specific user role
    fn delete(&self, payload: OldUserRole) -> ServiceFuture<UserRole>;
    /// Creates new user_role
    fn create(&self, payload: NewUserRole) -> ServiceFuture<UserRole>;
    /// Deletes default roles for user
    fn delete_default(&self, user_id: i32) -> ServiceFuture<UserRole>;
    /// Creates default roles for user
    fn create_default(&self, user_id: i32) -> ServiceFuture<UserRole>;
}

/// UserRoles services, responsible for UserRole-related CRUD operations
pub struct UserRolesServiceImpl<F: ReposFactory> {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub cached_roles: RolesCacheImpl,
    pub repo_factory: F,
}

impl<F: ReposFactory> UserRolesServiceImpl<F> {
    pub fn new(db_pool: DbPool, cpu_pool: CpuPool, cached_roles: RolesCacheImpl, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            cached_roles,
            repo_factory,
        }
    }
}

impl<F: ReposFactory + Send + 'static> UserRolesService for UserRolesServiceImpl<F> {
    /// Returns role by user ID
    fn get_roles(&self, user_id: i32) -> ServiceFuture<Vec<Role>> {
        let db_pool = self.db_pool.clone();
        let cached_roles = self.cached_roles.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    cached_roles
                        .get(user_id, Some(&conn))
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Deletes specific user role
    fn delete(&self, payload: OldUserRole) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let cached_roles = self.cached_roles.clone();
        let user_id = payload.user_id;
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let user_roles_repo = repo_factory.create_user_roles_repo(&conn);
                    user_roles_repo.delete(payload).map_err(ServiceError::from)
                })
                .and_then(|user_role| {
                    cached_roles
                        .remove(user_id)
                        .map(|_| user_role)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new user_role
    fn create(&self, new_user_role: NewUserRole) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let cached_roles = self.cached_roles.clone();
        let user_id = new_user_role.user_id;
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let user_roles_repo = repo_factory.create_user_roles_repo(&conn);
                    user_roles_repo
                        .create(new_user_role)
                        .map_err(ServiceError::from)
                })
                .and_then(|user_role| {
                    cached_roles
                        .remove(user_id)
                        .map(|_| user_role)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Deletes default roles for user
    fn delete_default(&self, user_id_arg: i32) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let cached_roles = self.cached_roles.clone();
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let user_roles_repo = repo_factory.create_user_roles_repo(&conn);
                    user_roles_repo
                        .delete_by_user_id(user_id_arg)
                        .map_err(ServiceError::from)
                })
                .and_then(|user_role| {
                    cached_roles
                        .remove(user_id_arg)
                        .map(|_| user_role)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Creates default roles for user
    fn create_default(&self, user_id_arg: i32) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let cached_roles = self.cached_roles.clone();
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let defaul_role = NewUserRole {
                        user_id: user_id_arg,
                        role: Role::User,
                    };
                    let user_roles_repo = repo_factory.create_user_roles_repo(&conn);
                    user_roles_repo
                        .create(defaul_role)
                        .map_err(ServiceError::from)
                })
                .and_then(|user_role| {
                    cached_roles
                        .remove(user_id_arg)
                        .map(|_| user_role)
                        .map_err(ServiceError::from)
                })
        }))
    }
}
