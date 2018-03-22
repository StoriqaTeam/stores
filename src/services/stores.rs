//! Stores Services, presents CRUD operations with stores

use futures_cpupool::CpuPool;
use futures::prelude::*;
use diesel::Connection;
use serde_json;
use stq_static_resources::Translation;
use stq_http::client::ClientHandle;

use models::{NewStore, SearchStore, Store, UpdateStore};
use repos::{StoresRepo, StoresRepoImpl};
use elastic::{StoresElastic, StoresElasticImpl};
use super::types::ServiceFuture;
use super::error::ServiceError;
use repos::types::DbPool;
use repos::acl::{ApplicationAcl, BoxedAcl, RolesCacheImpl, UnauthorizedAcl};

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
pub struct StoresServiceImpl {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub roles_cache: RolesCacheImpl,
    pub user_id: Option<i32>,
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

impl StoresServiceImpl {
    pub fn new(
        db_pool: DbPool,
        cpu_pool: CpuPool,
        roles_cache: RolesCacheImpl,
        user_id: Option<i32>,
        client_handle: ClientHandle,
        elastic_address: String,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            roles_cache,
            user_id,
            client_handle,
            elastic_address,
        }
    }
}

fn acl_for_id(roles_cache: RolesCacheImpl, user_id: Option<i32>) -> BoxedAcl {
    user_id.map_or(Box::new(UnauthorizedAcl::default()) as BoxedAcl, |id| {
        (Box::new(ApplicationAcl::new(roles_cache, id)) as BoxedAcl)
    })
}

impl StoresService for StoresServiceImpl {
    fn auto_complete(&self, name: String, count: i64, offset: i64) -> ServiceFuture<Vec<String>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let stores_names = {
            let stores_el = StoresElasticImpl::new(client_handle, address);
            stores_el
                .auto_complete(name, count, offset)
                .map_err(ServiceError::from)
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
                .map_err(ServiceError::from)
        };

        Box::new(stores.and_then({
            let cpu_pool = self.cpu_pool.clone();
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id;
            let roles_cache = self.roles_cache.clone();
            move |el_stores| {
                cpu_pool.spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| {
                            error!(
                                "Could not get connection to db from pool! {}",
                                e.to_string()
                            );
                            ServiceError::Connection(e.into())
                        })
                        .and_then(move |conn| {
                            el_stores
                                .into_iter()
                                .map(|el_store| {
                                    let acl = acl_for_id(roles_cache.clone(), user_id);
                                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                                    stores_repo.find(el_store.id).map_err(ServiceError::from)
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

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);

                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo.find(store_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Deactivates specific store
    fn deactivate(&self, store_id: i32) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo.deactivate(store_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Store>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo.list(from, count).map_err(ServiceError::from)
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
            cpu_pool.spawn_fn(move || {
                db_pool
                    .get()
                    .map_err(|e| {
                        error!(
                            "Could not get connection to db from pool! {}",
                            e.to_string()
                        );
                        ServiceError::Connection(e.into())
                    })
                    .and_then(move |conn| {
                        let acl = acl_for_id(roles_cache, user_id);
                        let stores_repo = StoresRepoImpl::new(&conn, acl);
                        conn.transaction::<Store, ServiceError, _>(move || {
                            serde_json::from_value::<Vec<Translation>>(payload.name.clone())
                                .map_err(|e| ServiceError::Parse(e.to_string()))
                                .and_then(|translations| {
                                    stores_repo
                                        .name_exists(translations)
                                        .map(move |exists| (payload, exists))
                                        .map_err(ServiceError::from)
                                        .and_then(|(payload, exists)| {
                                            if exists {
                                                Err(ServiceError::Validate(
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
                                        .map_err(ServiceError::from)
                                        .and_then(|(new_store, exists)| {
                                            if exists {
                                                Err(ServiceError::Validate(
                                                    validation_errors!({"slug": ["slug" => "Store with this slug already exists"]}),
                                                ))
                                            } else {
                                                Ok(new_store)
                                            }
                                        })
                                })
                                .and_then(move |new_store| stores_repo.create(new_store).map_err(ServiceError::from))
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

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);

                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo
                        .find(store_id.clone())
                        .and_then(move |_user| stores_repo.update(store_id, payload))
                        .map_err(ServiceError::from)
                })
        }))
    }
}
