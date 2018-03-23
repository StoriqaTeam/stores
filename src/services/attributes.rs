//! Attributes Services, presents CRUD operations with attributes

use futures_cpupool::CpuPool;

use models::{Attribute, NewAttribute, UpdateAttribute};
use services::types::ServiceFuture;
use services::error::ServiceError;
use repos::types::DbPool;
use repos::attributes::AttributeCacheImpl;
use repos::acl::{ApplicationAcl, BoxedAcl, RolesCacheImpl, UnauthorizedAcl};
use repos::ReposFactory;

pub trait AttributesService {
    /// Returns attribute by ID
    fn get(&self, attribute_id: i32) -> ServiceFuture<Attribute>;
    /// Creates new attribute
    fn create(&self, payload: NewAttribute) -> ServiceFuture<Attribute>;
    /// Updates specific attribute
    fn update(&self, attribute_id: i32, payload: UpdateAttribute) -> ServiceFuture<Attribute>;
}

fn acl_for_id(roles_cache: RolesCacheImpl, user_id: Option<i32>) -> BoxedAcl {
    user_id.map_or(Box::new(UnauthorizedAcl::default()) as BoxedAcl, |id| {
        (Box::new(ApplicationAcl::new(roles_cache, id)) as BoxedAcl)
    })
}

/// Attributes services, responsible for Attribute-related CRUD operations
pub struct AttributesServiceImpl<F: ReposFactory> {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub roles_cache: RolesCacheImpl,
    pub attributes_cache: AttributeCacheImpl,
    pub user_id: Option<i32>,
    pub repo_factory: F,
}

impl<F: ReposFactory> AttributesServiceImpl<F> {
    pub fn new(
        db_pool: DbPool,
        cpu_pool: CpuPool,
        roles_cache: RolesCacheImpl,
        attributes_cache: AttributeCacheImpl,
        user_id: Option<i32>,
        repo_factory: F,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            roles_cache,
            attributes_cache,
            user_id,
            repo_factory,
        }
    }
}

impl<F: ReposFactory + Send + 'static> AttributesService for AttributesServiceImpl<F> {
    /// Returns attribute by ID
    fn get(&self, attribute_id: i32) -> ServiceFuture<Attribute> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let attributes_cache = self.attributes_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    attributes_cache
                        .get(attribute_id, &conn, acl)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new attribute
    fn create(&self, new_attribute: NewAttribute) -> ServiceFuture<Attribute> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let attributes_repo = repo_factory.create_attributes_repo(&conn, roles_cache, user_id);
                    attributes_repo
                        .create(new_attribute)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Updates specific attribute
    fn update(&self, attribute_id: i32, payload: UpdateAttribute) -> ServiceFuture<Attribute> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let attributes_cache = self.attributes_cache.clone();
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let attributes_repo = repo_factory.create_attributes_repo(&conn, roles_cache, user_id);
                    attributes_repo
                        .update(attribute_id, payload)
                        .and_then(|attribute| attributes_cache.remove(attribute_id).map(|_| attribute))
                        .map_err(ServiceError::from)
                })
        }))
    }
}
