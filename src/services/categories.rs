//! Categories Services, presents CRUD operations with categories

use futures_cpupool::CpuPool;

use models::{Category, NewCategory, UpdateCategory};
use models::{Attribute, NewCatAttr, OldCatAttr};
use super::types::ServiceFuture;
use super::error::ServiceError;
use repos::types::{DbPool, RepoResult};
use repos::categories::{CategoriesRepo, CategoriesRepoImpl};
use repos::category_attrs::{CategoryAttrsRepo, CategoryAttrsRepoImpl};
use repos::attributes::{AttributesRepo, AttributesRepoImpl};
use repos::acl::{ApplicationAcl, BoxedAcl, RolesCacheImpl, UnauthorizedAcl};
use repos::categories::CategoryCacheImpl;

pub trait CategoriesService {
    /// Returns category by ID
    fn get(&self, category_id: i32) -> ServiceFuture<Category>;
    /// Creates new category
    fn create(&self, payload: NewCategory) -> ServiceFuture<Category>;
    /// Updates specific category
    fn update(&self, category_id: i32, payload: UpdateCategory) -> ServiceFuture<Category>;
    /// Returns all categories as a tree
    fn get_all(&self) -> ServiceFuture<Category>;
    /// Returns all category attributes belonging to category
    fn find_all_attributes(&self, category_id_arg: i32) -> ServiceFuture<Vec<Attribute>>;
    /// Creates new category attribute
    fn add_attribute_to_category(&self, payload: NewCatAttr) -> ServiceFuture<()>;
    /// Deletes category attribute
    fn delete_attribute_from_category(&self, payload: OldCatAttr) -> ServiceFuture<()>;
}

fn acl_for_id(roles_cache: RolesCacheImpl, user_id: Option<i32>) -> BoxedAcl {
    user_id.map_or(Box::new(UnauthorizedAcl::default()) as BoxedAcl, |id| {
        (Box::new(ApplicationAcl::new(roles_cache, id)) as BoxedAcl)
    })
}

/// Categories services, responsible for Category-related CRUD operations
pub struct CategoriesServiceImpl {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub roles_cache: RolesCacheImpl,
    pub categories_cache: CategoryCacheImpl,
    pub user_id: Option<i32>,
}

impl CategoriesServiceImpl {
    pub fn new(
        db_pool: DbPool,
        cpu_pool: CpuPool,
        roles_cache: RolesCacheImpl,
        categories_cache: CategoryCacheImpl,
        user_id: Option<i32>,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            roles_cache,
            categories_cache,
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
                    let categories_repo = CategoriesRepoImpl::new(&conn, acl);
                    categories_repo
                        .find(category_id)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new category
    fn create(&self, new_category: NewCategory) -> ServiceFuture<Category> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let categories_cache = self.categories_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let categories_repo = CategoriesRepoImpl::new(&conn, acl);
                    categories_repo
                        .create(new_category)
                        .and_then(|category| categories_cache.clear().map(|_| category))
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Updates specific category
    fn update(&self, category_id: i32, payload: UpdateCategory) -> ServiceFuture<Category> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let categories_cache = self.categories_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let categories_repo = CategoriesRepoImpl::new(&conn, acl);
                    categories_repo
                        .update(category_id, payload)
                        .and_then(|category| categories_cache.clear().map(|_| category))
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Returns category by ID
    fn get_all(&self) -> ServiceFuture<Category> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let categories_cache = self.categories_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    categories_cache.get(&conn, acl).map_err(ServiceError::from)
                })
        }))
    }

    /// Returns all category attributes belonging to category
    fn find_all_attributes(&self, category_id_arg: i32) -> ServiceFuture<Vec<Attribute>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), user_id);
                    let category_attrs_repo = CategoryAttrsRepoImpl::new(&conn, acl);
                    let acl = acl_for_id(roles_cache, user_id);
                    let attrs_repo = AttributesRepoImpl::new(&conn, acl);
                    category_attrs_repo
                        .find_all_attributes(category_id_arg)
                        .and_then(|cat_attrs| {
                            cat_attrs.into_iter().map(|cat_attr| {
                                attrs_repo
                                    .find(cat_attr.attr_id)
                            }).collect::<RepoResult<Vec<Attribute>>>()
                        })
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new category attribute
    fn add_attribute_to_category(&self, payload: NewCatAttr) -> ServiceFuture<()> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let category_attrs_repo = CategoryAttrsRepoImpl::new(&conn, acl);
                    category_attrs_repo
                        .create(payload)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Deletes category attribute
    fn delete_attribute_from_category(&self, payload: OldCatAttr) -> ServiceFuture<()> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let category_attrs_repo = CategoryAttrsRepoImpl::new(&conn, acl);
                    category_attrs_repo
                        .delete(payload)
                        .map_err(ServiceError::from)
                })
        }))
    }
}
