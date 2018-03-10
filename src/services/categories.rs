//! Categories Services, presents CRUD operations with categorys

use futures_cpupool::CpuPool;

use stq_acl::UnauthorizedACL;

use models::{Category, NewCategory, UpdateCategory, CategoryTree};
use super::types::ServiceFuture;
use super::error::ServiceError;
use repos::types::DbPool;
use repos::categories::{CategoriesRepo, CategoriesRepoImpl};

use repos::acl::{ApplicationAcl, BoxedAcl, RolesCacheImpl};

pub trait CategoriesService {
    /// Returns category by ID
    fn get(&self, category_id: i32) -> ServiceFuture<Category>;
    /// Creates new category
    fn create(&self, payload: NewCategory) -> ServiceFuture<Category>;
    /// Updates specific category
    fn update(&self, category_id: i32, payload: UpdateCategory) -> ServiceFuture<Category>;
    /// Returns all categories as a tree
    fn get_all(&self) -> ServiceFuture<Vec<CategoryTree>>;
}

fn acl_for_id(roles_cache: RolesCacheImpl, user_id: Option<i32>) -> BoxedAcl {
    user_id.map_or(Box::new(UnauthorizedACL::default()) as BoxedAcl, |id| {
        (Box::new(ApplicationAcl::new(roles_cache, id)) as BoxedAcl)
    })
}

/// Categories services, responsible for Category-related CRUD operations
pub struct CategoriesServiceImpl {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub roles_cache: RolesCacheImpl,
    pub user_id: Option<i32>,
}

impl CategoriesServiceImpl {
    pub fn new(db_pool: DbPool, cpu_pool: CpuPool, roles_cache: RolesCacheImpl, user_id: Option<i32>) -> Self {
        Self {
            db_pool,
            cpu_pool,
            roles_cache,
            user_id,
        }
    }
}

impl CategoriesService for CategoriesServiceImpl {
    /// Returns category by ID
    fn get(&self, category_id: i32) -> ServiceFuture<Category> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let categorys_repo = CategoriesRepoImpl::new(&conn, acl);
                    categorys_repo.find(category_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new category
    fn create(&self, new_category: NewCategory) -> ServiceFuture<Category> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let categorys_repo = CategoriesRepoImpl::new(&conn, acl);
                    categorys_repo
                        .create(new_category)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Updates specific category
    fn update(&self, category_id: i32, payload: UpdateCategory) -> ServiceFuture<Category> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let categorys_repo = CategoriesRepoImpl::new(&conn, acl);
                    categorys_repo
                        .update(category_id, payload)
                        .map_err(ServiceError::from)
                })
        }))
    }

     /// Returns category by ID
    fn get_all(&self) -> ServiceFuture<Vec<CategoryTree>>{
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let categorys_repo = CategoriesRepoImpl::new(&conn, acl);
                    categorys_repo.get_all().map_err(ServiceError::from)
                })
        }))
    }
}
