//! UserRoles Services, presents CRUD operations with user_roles

use futures_cpupool::CpuPool;

use models::{NewUserRole, OldUserRole, UserRole};
use super::types::ServiceFuture;
use super::error::Error;
use repos::types::DbPool;
use repos::acl::{RolesCache, SystemACL};
use repos::user_roles::{UserRolesRepo, UserRolesRepoImpl};

pub trait UserRolesService {
    /// Returns user_role by ID
    fn get(&self, user_role_id: i32) -> ServiceFuture<Vec<UserRole>>;
    /// Delete specific user role
    fn delete(&self, payload: OldUserRole) -> ServiceFuture<()>;
    /// Creates new user_role
    fn create(&self, payload: NewUserRole) -> ServiceFuture<UserRole>;
}

/// UserRoles services, responsible for UserRole-related CRUD operations
pub struct UserRolesServiceImpl<R: RolesCache + Clone + Send + 'static> {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub roles_cache: R,
}

impl<R: RolesCache + Clone + Send + 'static> UserRolesServiceImpl<R> {
    pub fn new(db_pool: DbPool, cpu_pool: CpuPool, roles_cache: R) -> Self {
        Self {
            db_pool,
            cpu_pool,
            roles_cache,
        }
    }
}

impl<R: RolesCache + Clone + Send + 'static> UserRolesService for UserRolesServiceImpl<R> {
    /// Returns user_role by ID
    fn get(&self, user_role_id: i32) -> ServiceFuture<Vec<UserRole>> {
        let db_pool = self.db_pool.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = SystemACL::new();
                    let user_roles_repo = UserRolesRepoImpl::new(&conn, &acl);
                    user_roles_repo
                        .list_for_user(user_role_id)
                        .map_err(Error::from)
                })
        }))
    }

    /// Deletes specific user role
    fn delete(&self, payload: OldUserRole) -> ServiceFuture<()> {
        let db_pool = self.db_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let user_id = payload.user_id;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = SystemACL::new();
                    let user_roles_repo = UserRolesRepoImpl::new(&conn, &acl);
                    user_roles_repo.delete(payload).map_err(Error::from)
                })
                .and_then(|_| roles_cache.remove(user_id).map_err(Error::from))
        }))
    }

    /// Creates new user_role
    fn create(&self, new_user_role: NewUserRole) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let user_id = new_user_role.user_id;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = SystemACL::new();
                    let user_roles_repo = UserRolesRepoImpl::new(&conn, &acl);
                    user_roles_repo.create(new_user_role).map_err(Error::from)
                })
                .and_then(|user_role| {
                    roles_cache
                        .remove(user_id)
                        .map_err(Error::from)
                        .and_then(|_| Ok(user_role))
                })
        }))
    }
}
