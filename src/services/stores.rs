//! Stores Services, presents CRUD operations with stores

use futures_cpupool::CpuPool;
use futures::prelude::*;
use diesel::Connection;
use serde_json;
use stq_static_resources::Translation;
use stq_http::client::ClientHandle;

use models::{NewStore, SearchStore, Store, UpdateStore};
use elastic::{StoresElastic, StoresElasticImpl};
use super::types::ServiceFuture;
use super::error::ServiceError as Error;
use repos::types::DbPool;
use repos::acl::RolesCacheImpl;
use repos::ReposFactory;

pub trait StoresService {
    /// Find stores by name limited by `count` parameters
    fn find_by_name(&self, search_store: SearchStore, count: i64, offset: i64) -> ServiceFuture<Vec<Store>>;
    /// Find stores auto complete limited by `count` parameters
    fn auto_complete(&self, name: String, count: i64, offset: i64) -> ServiceFuture<Vec<String>>;
    /// Returns store by ID
    fn get(&self, store_id: i32) -> ServiceFuture<Store>;
    /// Deactivates specific store
    fn deactivate(&self, store_id: i32) -> ServiceFuture<Store>;
    /// Creates new store
    fn create(&self, payload: NewStore) -> ServiceFuture<Store>;
    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Store>>;
    /// Updates specific store
    fn update(&self, store_id: i32, payload: UpdateStore) -> ServiceFuture<Store>;
}

/// Stores services, responsible for Store-related CRUD operations
pub struct StoresServiceImpl<F: ReposFactory> {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub roles_cache: RolesCacheImpl,
    pub user_id: Option<i32>,
    pub client_handle: ClientHandle,
    pub elastic_address: String,
    pub repo_factory: F,
}

impl<F: ReposFactory> StoresServiceImpl<F> {
    pub fn new(
        db_pool: DbPool,
        cpu_pool: CpuPool,
        roles_cache: RolesCacheImpl,
        user_id: Option<i32>,
        client_handle: ClientHandle,
        elastic_address: String,
        repo_factory: F,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            roles_cache,
            user_id,
            client_handle,
            elastic_address,
            repo_factory,
        }
    }
}

impl<F: ReposFactory + Send + 'static> StoresService for StoresServiceImpl<F> {
    fn auto_complete(&self, name: String, count: i64, offset: i64) -> ServiceFuture<Vec<String>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let stores_names = {
            let stores_el = StoresElasticImpl::new(client_handle, address);
            stores_el
                .auto_complete(name, count, offset)
                .map_err(Error::from)
        };

        Box::new(stores_names)
    }

    /// Find stores by name
    fn find_by_name(&self, search_store: SearchStore, count: i64, offset: i64) -> ServiceFuture<Vec<Store>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let stores = {
            let stores_el = StoresElasticImpl::new(client_handle, address);
            stores_el
                .find_by_name(search_store, count, offset)
                .map_err(Error::from)
        };

        Box::new(stores.and_then({
            let cpu_pool = self.cpu_pool.clone();
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id;
            let roles_cache = self.roles_cache.clone();
            let repo_factory = self.repo_factory;
            move |el_stores| {
                cpu_pool.spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| Error::Connection(e.into()))
                        .and_then(move |conn| {
                            el_stores
                                .into_iter()
                                .map(|el_store| {
                                    let stores_repo = repo_factory.create_stores_repo(&conn, roles_cache.clone(), user_id);
                                    stores_repo.find(el_store.id).map_err(Error::from)
                                })
                                .collect()
                        })
                })
            }
        }))
    }

    /// Returns store by ID
    fn get(&self, store_id: i32) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let stores_repo = repo_factory.create_stores_repo(&conn, roles_cache, user_id);
                    stores_repo.find(store_id).map_err(Error::from)
                })
        }))
    }

    /// Deactivates specific store
    fn deactivate(&self, store_id: i32) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let stores_repo = repo_factory.create_stores_repo(&conn, roles_cache, user_id);
                    stores_repo.deactivate(store_id).map_err(Error::from)
                })
        }))
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Store>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let stores_repo = repo_factory.create_stores_repo(&conn, roles_cache, user_id);
                    stores_repo.list(from, count).map_err(Error::from)
                })
        }))
    }

    /// Creates new store
    fn create(&self, payload: NewStore) -> ServiceFuture<Store> {
        Box::new({
            let cpu_pool = self.cpu_pool.clone();
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id;
            let roles_cache = self.roles_cache.clone();
            let repo_factory = self.repo_factory;
            cpu_pool.spawn_fn(move || {
                db_pool
                    .get()
                    .map_err(|e| Error::Connection(e.into()))
                    .and_then(move |conn| {
                        let stores_repo = repo_factory.create_stores_repo(&conn, roles_cache, user_id);
                        conn.transaction::<Store, Error, _>(move || {
                            serde_json::from_value::<Vec<Translation>>(payload.name.clone())
                                .map_err(|e| Error::Parse(e.to_string()))
                                .and_then(|translations| {
                                    stores_repo
                                        .name_exists(translations)
                                        .map(move |exists| (payload, exists))
                                        .map_err(Error::from)
                                        .and_then(|(payload, exists)| {
                                            if exists {
                                                Err(Error::Validate(
                                                    validation_errors!({"name": ["name" => "Store with this name already exists"]}),
                                                ))
                                            } else {
                                                Ok(payload)
                                            }
                                        })
                                })
                                .and_then(|payload| {
                                    stores_repo
                                        .slug_exists(payload.slug.to_string())
                                        .map(move |exists| (payload, exists))
                                        .map_err(Error::from)
                                        .and_then(|(new_store, exists)| {
                                            if exists {
                                                Err(Error::Validate(
                                                    validation_errors!({"slug": ["slug" => "Store with this slug already exists"]}),
                                                ))
                                            } else {
                                                Ok(new_store)
                                            }
                                        })
                                })
                                .and_then(move |new_store| stores_repo.create(new_store).map_err(Error::from))
                        })
                    })
            })
        })
    }

    /// Updates specific store
    fn update(&self, store_id: i32, payload: UpdateStore) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let repo_factory = self.repo_factory;

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let stores_repo = repo_factory.create_stores_repo(&conn, roles_cache, user_id);
                    stores_repo
                        .find(store_id.clone())
                        .and_then(move |_user| stores_repo.update(store_id, payload))
                        .map_err(Error::from)
                })
        }))
    }
}
