//! Stores Services, presents CRUD operations with stores
use std::sync::Arc;

use futures::future;
use futures::Future;
use futures_cpupool::CpuPool;


use models::{NewStore, UpdateStore, Store};
use repos::stores::{StoresRepo, StoresRepoImpl};
use super::types::ServiceFuture;
use super::error::Error;
use repos::types::DbPool;

use repos::acl::{ApplicationAcl, RolesCacheImpl, Acl, UnAuthanticatedACL};



pub trait StoresService {
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
pub struct StoresServiceImpl<U: 'static + StoresRepo + Clone> {
    pub stores_repo: U,
    pub user_id: Option<i32>,
}

impl StoresServiceImpl<StoresRepoImpl> {
    pub fn new(db_pool: DbPool,
        cpu_pool: CpuPool,
        roles_cache: RolesCacheImpl,
        user_id: Option<i32>,
    ) -> Self {
        let acl =  user_id.map_or((Arc::new(UnAuthanticatedACL::new()) as Arc<Acl>), |id| (Arc::new(ApplicationAcl::new(roles_cache.clone(), id)) as Arc<Acl>));
        let stores_repo = StoresRepoImpl::new(db_pool, cpu_pool, acl);
        Self {
            stores_repo: stores_repo,
            user_id: user_id,
        }
    }
}

impl<U: StoresRepo + Clone> StoresService for StoresServiceImpl<U> {
    /// Returns store by ID
    fn get(&self, store_id: i32) -> ServiceFuture<Store> {
        Box::new(self.stores_repo.find(store_id).map_err(Error::from))
    }
    
    /// Deactivates specific store
    fn deactivate(&self, store_id: i32) -> ServiceFuture<Store> {
        Box::new(
            self.stores_repo
                .deactivate(store_id)
                .map_err(|e| Error::from(e)),
        )
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Store>> {
        Box::new(
            self.stores_repo
                .list(from, count)
                .map_err(|e| Error::from(e)),
        )
    }

    /// Creates new store
    fn create(&self, payload: NewStore) -> ServiceFuture<Store> {
        let stores_repo = self.stores_repo.clone();
        Box::new(
            stores_repo
                .name_exists(payload.name.to_string())
                .map(move |exists| (payload, exists))
                .map_err(Error::from)
                .and_then(|(payload, exists)| match exists {
                    false => future::ok(payload),
                    true => future::err(Error::Validate(
                        validation_errors!({"name": ["name" => "Name already exists"]}),
                    )),
                })
                .and_then(move |new_store| {
                    stores_repo
                        .create(new_store)
                        .map_err(|e| Error::from(e))
                })
        )
    }

    /// Updates specific store
    fn update(&self, store_id: i32, payload: UpdateStore) -> ServiceFuture<Store> {
        let stores_repo = self.stores_repo.clone();

        Box::new(
            stores_repo
                .find(store_id)
                .and_then(move |_store| stores_repo.update(store_id, payload))
                .map_err(|e| Error::from(e)),
        )
    }
}