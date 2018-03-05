//! Stores Services, presents CRUD operations with stores

use future;
use futures_cpupool::CpuPool;
use futures::prelude::*;
use diesel::Connection;
use serde_json;
use stq_acl::UnauthorizedACL;

use models::{NewStore, SearchStore, Store, Translation, UpdateStore};
use repos::{StoresRepo, StoresRepoImpl, StoresSearchRepo, StoresSearchRepoImpl};
use super::types::ServiceFuture;
use super::error::ServiceError as Error;

use repos::types::DbPool;
use stq_http::client::ClientHandle;

use repos::acl::{ApplicationAcl, BoxedAcl, RolesCacheImpl};

pub trait StoresService {
    /// Find stores by name limited by `count` parameters
    fn find_by_name(&self, search_store: SearchStore, count: i64, offset: i64) -> ServiceFuture<Vec<Store>>;
    /// Find stores full name by name part limited by `count` parameters
    fn find_full_names_by_name_part(&self, search_store: SearchStore, count: i64, offset: i64) -> ServiceFuture<Vec<String>>;
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
    user_id.map_or((Box::new(UnauthorizedACL::default()) as BoxedAcl), |id| {
        (Box::new(ApplicationAcl::new(roles_cache, id)) as BoxedAcl)
    })
}

impl StoresService for StoresServiceImpl {
    fn find_full_names_by_name_part(&self, search_store: SearchStore, count: i64, offset: i64) -> ServiceFuture<Vec<String>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let stores_names = {
            let stores_el = StoresSearchRepoImpl::new(client_handle, address);
            let name = search_store.name.clone();
            stores_el
                .find_by_name(search_store, count, offset)
                .map_err(Error::from)
                .and_then(|el_stores| {
                    el_stores
                        .into_iter()
                        .map(move |el_store| {
                            serde_json::from_value::<Vec<Translation>>(el_store.name)
                                .map_err(|e| Error::Parse(e.to_string()))
                                .and_then(|translations| {
                                    translations
                                        .into_iter()
                                        .find(|transl| transl.text.contains(&name))
                                        .ok_or(Error::NotFound)
                                        .map(|t| t.text)
                                })
                        })
                        .collect::<Result<Vec<String>, Error>>()
                        .into_future()
                })
        };

        Box::new(stores_names)
    }

    /// Find stores by name
    fn find_by_name(&self, search_store: SearchStore, count: i64, offset: i64) -> ServiceFuture<Vec<Store>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let stores = {
            let stores_el = StoresSearchRepoImpl::new(client_handle, address);
            stores_el
                .find_by_name(search_store, count, offset)
                .map_err(Error::from)
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
                        .map_err(|e| Error::Connection(e.into()))
                        .and_then(move |conn| {
                            el_stores
                                .into_iter()
                                .map(|el_store| {
                                    let acl = acl_for_id(roles_cache.clone(), user_id);
                                    let stores_repo = StoresRepoImpl::new(&conn, acl);
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

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);

                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo.find(store_id).map_err(Error::from)
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
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo.deactivate(store_id).map_err(Error::from)
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
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo.list(from, count).map_err(Error::from)
                })
        }))
    }

    /// Creates new store
    fn create(&self, payload: NewStore) -> ServiceFuture<Store> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let check_store_name_exists = {
            let stores_el = StoresSearchRepoImpl::new(client_handle, address);
            stores_el
                .name_exists(payload.name.to_string())
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
        };

        Box::new(check_store_name_exists.and_then({
            let cpu_pool = self.cpu_pool.clone();
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id;
            let roles_cache = self.roles_cache.clone();
            let client_handle = self.client_handle.clone();
            let address = self.elastic_address.clone();
            move |new_store| {
                cpu_pool
                    .spawn_fn(move || {
                        db_pool
                            .get()
                            .map_err(|e| Error::Connection(e.into()))
                            .and_then(move |conn| {
                                let acl = acl_for_id(roles_cache, user_id);
                                let stores_repo = StoresRepoImpl::new(&conn, acl);
                                conn.transaction::<Store, Error, _>(move || stores_repo.create(new_store).map_err(Error::from))
                            })
                    })
                    .and_then({
                        move |store| {
                            let fut = {
                                let stores_el = StoresSearchRepoImpl::new(client_handle, address);
                                stores_el
                                    .create(store.clone().into())
                                    .map_err(Error::from)
                                    .and_then(|_| future::ok(store))
                            };
                            cpu_pool.spawn(fut)
                        }
                    })
            }
        }))
    }

    /// Updates specific store
    fn update(&self, store_id: i32, payload: UpdateStore) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| Error::Connection(e.into()))
                        .and_then(move |conn| {
                            let acl = acl_for_id(roles_cache, user_id);

                            let stores_repo = StoresRepoImpl::new(&conn, acl);
                            stores_repo
                                .find(store_id.clone())
                                .and_then(move |_user| stores_repo.update(store_id, payload))
                                .map_err(Error::from)
                        })
                })
                .and_then({
                    let cpu_pool = self.cpu_pool.clone();
                    let client_handle = self.client_handle.clone();
                    let address = self.elastic_address.clone();
                    move |store| {
                        let fut = {
                            let stores_el = StoresSearchRepoImpl::new(client_handle, address);
                            stores_el
                                .update(store.clone().into())
                                .map_err(Error::from)
                                .and_then(|_| future::ok(store))
                        };
                        cpu_pool.spawn(fut)
                    }
                }),
        )
    }
}
