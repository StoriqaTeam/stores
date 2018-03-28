//! Attributes Services, presents CRUD operations with attributes

use futures_cpupool::CpuPool;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use models::{Attribute, NewAttribute, UpdateAttribute};
use services::types::ServiceFuture;
use services::error::ServiceError;
use r2d2::{ManageConnection, Pool};
use repos::attributes::AttributeCacheImpl;
use repos::ReposFactory;

pub trait AttributesService {
    /// Returns attribute by ID
    fn get(&self, attribute_id: i32) -> ServiceFuture<Attribute>;
    /// Creates new attribute
    fn create(&self, payload: NewAttribute) -> ServiceFuture<Attribute>;
    /// Updates specific attribute
    fn update(&self, attribute_id: i32, payload: UpdateAttribute) -> ServiceFuture<Attribute>;
}

/// Attributes services, responsible for Attribute-related CRUD operations
pub struct AttributesServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    F: ReposFactory<T>,
    M: ManageConnection<Connection = T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub attributes_cache: AttributeCacheImpl,
    pub user_id: Option<i32>,
    pub repo_factory: F,
}

impl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    F: ReposFactory<T>,
    M: ManageConnection<Connection = T>,
> AttributesServiceImpl<T, F, M>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, attributes_cache: AttributeCacheImpl, user_id: Option<i32>, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            attributes_cache,
            user_id,
            repo_factory,
        }
    }
}

impl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    F: ReposFactory<T>,
    M: ManageConnection<Connection = T>,
> AttributesService for AttributesServiceImpl<T, F, M>
{
    /// Returns attribute by ID
    fn get(&self, attribute_id: i32) -> ServiceFuture<Attribute> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let attributes_cache = self.attributes_cache.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    if attributes_cache.contains(attribute_id) {
                        attributes_cache
                            .get(attribute_id)
                            .map_err(ServiceError::from)
                    } else {
                        let attributes_repo = repo_factory.create_attributes_repo(&*conn, user_id);
                        attributes_repo
                            .find(attribute_id)
                            .map_err(ServiceError::from)
                            .and_then(|attr| {
                                attributes_cache.add_attribute(attribute_id, attr.clone());
                                Ok(attr)
                            })
                    }
                })
        }))
    }

    /// Creates new attribute
    fn create(&self, new_attribute: NewAttribute) -> ServiceFuture<Attribute> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let attributes_repo = repo_factory.create_attributes_repo(&*conn, user_id);
                    conn.transaction::<(Attribute), ServiceError, _>(move || {
                        attributes_repo
                            .create(new_attribute)
                            .map_err(ServiceError::from)
                    })
                })
        }))
    }

    /// Updates specific attribute
    fn update(&self, attribute_id: i32, payload: UpdateAttribute) -> ServiceFuture<Attribute> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let attributes_cache = self.attributes_cache.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let attributes_repo = repo_factory.create_attributes_repo(&*conn, user_id);
                    attributes_repo
                        .update(attribute_id, payload)
                        .map_err(ServiceError::from)
                        .and_then(|attribute| {
                            attributes_cache.remove(attribute_id);
                            Ok(attribute)
                        })
                })
        }))
    }
}
