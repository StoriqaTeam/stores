use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::select;
use diesel::dsl::exists;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;
use futures::future;
use futures_cpupool::CpuPool;

use models::store::{UpdateStore, Store, NewStore};
use models::store::stores::dsl::*;
use super::error::Error;
use super::types::{DbConnection, DbPool, RepoFuture};

/// Stores repository, responsible for handling stores
#[derive(Clone)]
pub struct StoresRepoImpl {
    // Todo - no need for Arc, since pool is itself an ARC-like structure
    pub r2d2_pool: DbPool,
    pub cpu_pool: CpuPool
}

pub trait StoresRepo {
    /// Find specific store by ID
    fn find(&self, store_id: i32) -> RepoFuture<Store>;

    /// Verifies store exist
    fn name_exists(&self, name_arg: String) -> RepoFuture<bool>;

    /// Find specific store by full name
    fn find_by_name(&self, name_arg: String) -> RepoFuture<Store>;

    /// Returns list of stores, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoFuture<Vec<Store>>;

    /// Creates new store
    fn create(&self, payload: NewStore) -> RepoFuture<Store>;

    /// Updates specific store
    fn update(&self, store_id: i32, payload: UpdateStore) -> RepoFuture<Store>;

    /// Deactivates specific store
    fn deactivate(&self, store_id: i32) -> RepoFuture<Store>;
}

impl StoresRepoImpl {
    pub fn new(r2d2_pool: DbPool, cpu_pool: CpuPool) -> Self {
        Self {
            r2d2_pool,
            cpu_pool
        }
    }

    fn get_connection(&self) -> DbConnection {
        match self.r2d2_pool.get() {
            Ok(connection) => connection,
            Err(e) => panic!("Error obtaining connection from pool: {}", e),
        }
    }

    fn execute_query<T: Send + 'static, U: LoadQuery<PgConnection, T> + Send + 'static>(
        &self,
        query: U,
    ) -> RepoFuture<T> {
        let conn = match self.r2d2_pool.get() {
            Ok(connection) => connection,
            Err(_) => {
                return Box::new(future::err(
                    Error::Connection("Cannot connect to stores db".to_string()),
                ))
            }
        };

        Box::new(self.cpu_pool.spawn_fn(move || {
            query.get_result::<T>(&*conn).map_err(|e| Error::from(e))
        }))
    }
}

impl StoresRepo for StoresRepoImpl {
    /// Find specific store by ID
    fn find(&self, store_id_arg: i32) -> RepoFuture<Store> {
        self.execute_query(stores.find(store_id_arg))
    }

    /// Verifies store exist
    fn name_exists(&self, name_arg: String) -> RepoFuture<bool> {
        self.execute_query(select(exists(
            stores
                .filter(name.eq(name_arg))
        )))
    }

    /// Find specific store by full name
    fn find_by_name(&self, name_arg: String) -> RepoFuture<Store>{
        let conn = self.get_connection();
        let query = stores
            .filter(name.eq(name_arg));

        Box::new(self.cpu_pool.spawn_fn(move || {
            query.first::<Store>(&*conn).map_err(|e| Error::from(e))
        }))
    }


    /// Creates new store
    fn create(&self, payload: NewStore) -> RepoFuture<Store> {
        let conn = self.get_connection();

        Box::new(self.cpu_pool.spawn_fn(move || {
            let query_store = diesel::insert_into(stores).values(&payload);
            query_store
                .get_result::<Store>(&*conn)
                .map_err(Error::from)
        }))
    }

     /// Returns list of stores, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoFuture<Vec<Store>> {
        let conn = self.get_connection();
        let query = stores
            .filter(is_active.eq(true))
            .filter(id.gt(from))
            .order(id)
            .limit(count);

        Box::new(self.cpu_pool.spawn_fn(move || {
            query.get_results(&*conn).map_err(|e| Error::from(e))
        }))
    }

    /// Updates specific store
    fn update(&self, store_id_arg: i32, payload: UpdateStore) -> RepoFuture<Store> {
        let conn = self.get_connection();
        let filter = stores.filter(id.eq(store_id_arg)).filter(is_active.eq(true));

        Box::new(self.cpu_pool.spawn_fn(move || {
            let query = diesel::update(filter).set(&payload);
            query.get_result::<Store>(&*conn).map_err(|e| Error::from(e))
        }))
    }

    /// Deactivates specific store
    fn deactivate(&self, store_id_arg: i32) -> RepoFuture<Store> {
        let filter = stores.filter(id.eq(store_id_arg)).filter(is_active.eq(true));
        let query = diesel::update(filter).set(is_active.eq(false));
        self.execute_query(query)
    }
}
