//! Stores Services, presents CRUD operations with stores

use futures_cpupool::CpuPool;
use diesel::Connection;


use models::{NewStore, UpdateStore, Store};
use repos::stores::{StoresRepo, StoresRepoImpl};
use super::types::ServiceFuture;
use super::error::Error;
use repos::types::DbPool;

use repos::acl::{ApplicationAcl, RolesCache, Acl, UnauthorizedACL};



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
pub struct StoresServiceImpl<R: RolesCache + Clone + Send + 'static> {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub roles_cache: R,
    pub user_id: Option<i32>,
}

impl<R: RolesCache + Clone + Send + 'static> StoresServiceImpl<R> {
    pub fn new(
        db_pool: DbPool,
        cpu_pool: CpuPool,
        roles_cache: R,
        user_id: Option<i32>,
    ) -> Self {
        
        Self {
            db_pool,
            cpu_pool,
            roles_cache,
            user_id
        }
    }
}

impl<R: RolesCache + Clone + Send + 'static> StoresService for StoresServiceImpl<R> {
    /// Returns store by ID
    fn get(&self, store_id: i32) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                    });
                    let mut stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo.find(store_id).map_err(Error::from)
                })
        }))
    }

    /// Deactivates specific store
    fn deactivate(&self, store_id: i32) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                    });
                    let mut stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo
                        .deactivate(store_id)
                        .map_err(|e| Error::from(e))
                })
        }))
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Store>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                    });
                    let mut stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo.list(from, count).map_err(|e| Error::from(e))
                })
        }))
    }

    /// Creates new store
    fn create(&self, payload: NewStore) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                    });
                    let mut stores_repo = StoresRepoImpl::new(&conn, acl);
                    conn.transaction::<Store, Error, _>(move || {
                        stores_repo
                            .name_exists(payload.name.to_string())
                            .map(move |exists| (payload, exists))
                            .map_err(Error::from)
                            .and_then(|(payload, exists)| match exists {
                                false => Ok(payload),
                                true => Err(Error::Database("Store already exists".into())),
                            })
                            .and_then(move |new_store| {
                                stores_repo
                                    .create(new_store)
                                    .map_err(|e| Error::from(e))
                            })
                            //rollback if error
                    })
                })
        }))
    }

    /// Updates specific store
    fn update(&self, store_id: i32, payload: UpdateStore) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                    });
                    let mut stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo
                        .find(store_id.clone())
                        .and_then(move |_user| stores_repo.update(store_id, payload))
                        .map_err(|e| Error::from(e))
                })
                            //rollback if error
        }))
    }
}