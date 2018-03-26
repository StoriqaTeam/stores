//! Categories Services, presents CRUD operations with categories

use futures_cpupool::CpuPool;
use stq_acl::RolesCache;

use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use r2d2::{ManageConnection, Pool};

use models::{Category, NewCategory, UpdateCategory};
use models::{Attribute, NewCatAttr, OldCatAttr};
use models::authorization::*;
use super::types::ServiceFuture;
use super::error::ServiceError;
use repos::types::RepoResult;
use repos::categories::CategoryCache;
use repos::ReposFactory;
use repos::error::RepoError;

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

/// Categories services, responsible for Category-related CRUD operations
pub struct CategoriesServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory,
    C: CategoryCache,
    R: RolesCache<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub roles_cache: R,
    pub categories_cache: C,
    pub user_id: Option<i32>,
    pub repo_factory: F,
}

impl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory,
    C: CategoryCache,
    R: RolesCache<T>,
> CategoriesServiceImpl<T, M, F, C, R>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, roles_cache: R, categories_cache: C, user_id: Option<i32>, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            roles_cache,
            categories_cache,
            user_id,
            repo_factory,
        }
    }
}

impl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory,
    C: CategoryCache,
    R: RolesCache<T, Role = Role, Error = RepoError>,
> CategoriesService for CategoriesServiceImpl<T, M, F, C, R>
{
    /// Returns category by ID
    fn get(&self, category_id: i32) -> ServiceFuture<Category> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let categories_repo = repo_factory.create_categories_repo(&*conn, roles_cache, user_id);
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
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let categories_repo = repo_factory.create_categories_repo(&*conn, roles_cache, user_id);
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
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let categories_repo = repo_factory.create_categories_repo(&*conn, roles_cache, user_id);
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
                    categories_cache
                        .get(&*conn, roles_cache, user_id)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Returns all category attributes belonging to category
    fn find_all_attributes(&self, category_id_arg: i32) -> ServiceFuture<Vec<Attribute>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let category_attrs_repo = repo_factory.create_category_attrs_repo(&*conn, roles_cache.clone(), user_id);
                    let attrs_repo = repo_factory.create_attributes_repo(&*conn, roles_cache.clone(), user_id);
                    category_attrs_repo
                        .find_all_attributes(category_id_arg)
                        .and_then(|cat_attrs| {
                            cat_attrs
                                .into_iter()
                                .map(|cat_attr| attrs_repo.find(cat_attr.attr_id))
                                .collect::<RepoResult<Vec<Attribute>>>()
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
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let category_attrs_repo = repo_factory.create_category_attrs_repo(&*conn, roles_cache, user_id);
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
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let category_attrs_repo = repo_factory.create_category_attrs_repo(&*conn, roles_cache, user_id);
                    category_attrs_repo
                        .delete(payload)
                        .map_err(ServiceError::from)
                })
        }))
    }
}
