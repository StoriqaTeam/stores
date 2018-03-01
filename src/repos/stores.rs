//! Stores repo, presents CRUD operations with db for users
use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;

use models::{NewStore, Store, UpdateStore};
use models::store::stores::dsl::*;
use super::error::RepoError as Error;
use super::types::{DbConnection, RepoResult};
use repos::acl::Acl;
use models::authorization::*;

/// Stores repository, responsible for handling stores
pub struct StoresRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: &'a Acl,
}

pub trait StoresRepo {
    /// Find specific store by ID
    fn find(&self, store_id: i32) -> RepoResult<Store>;

    /// Returns list of stores, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<Store>>;

    /// Creates new store
    fn create(&self, payload: NewStore) -> RepoResult<Store>;

    /// Updates specific store
    fn update(&self, store_id: i32, payload: UpdateStore) -> RepoResult<Store>;

    /// Deactivates specific store
    fn deactivate(&self, store_id: i32) -> RepoResult<Store>;
}

impl<'a> StoresRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl: &'a Acl) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<T: Send + 'static, U: LoadQuery<PgConnection, T> + Send + 'static>(&self, query: U) -> Result<T, Error> {
        query.get_result::<T>(&**self.db_conn).map_err(Error::from)
    }
}

impl<'a> StoresRepo for StoresRepoImpl<'a> {
    /// Find specific store by ID
    fn find(&self, store_id_arg: i32) -> RepoResult<Store> {
        self.execute_query(stores.find(store_id_arg))
            .and_then(|store: Store| {
                acl!(
                    [store],
                    self.acl,
                    Resource::Stores,
                    Action::Read,
                    Some(self.db_conn)
                ).and_then(|_| Ok(store))
            })
    }

    /// Creates new store
    fn create(&self, payload: NewStore) -> RepoResult<Store> {
        acl!(
            [payload],
            self.acl,
            Resource::Stores,
            Action::Create,
            Some(self.db_conn)
        ).and_then(|_| {
            let query_store = diesel::insert_into(stores).values(&payload);
            query_store
                .get_result::<Store>(&**self.db_conn)
                .map_err(Error::from)
        })
    }

    /// Returns list of stores, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<Store>> {
        let query = stores
            .filter(is_active.eq(true))
            .filter(id.gt(from))
            .order(id)
            .limit(count);

        query
            .get_results(&**self.db_conn)
            .map_err(Error::from)
            .and_then(|stores_res: Vec<Store>| {
                let resources = stores_res
                    .iter()
                    .map(|store| (store as &WithScope))
                    .collect();
                acl!(
                    resources,
                    self.acl,
                    Resource::Stores,
                    Action::Read,
                    Some(self.db_conn)
                ).and_then(|_| Ok(stores_res.clone()))
            })
    }

    /// Updates specific store
    fn update(&self, store_id_arg: i32, payload: UpdateStore) -> RepoResult<Store> {
        self.execute_query(stores.find(store_id_arg))
            .and_then(|store: Store| {
                acl!(
                    [store],
                    self.acl,
                    Resource::Stores,
                    Action::Update,
                    Some(self.db_conn)
                )
            })
            .and_then(|_| {
                let filter = stores
                    .filter(id.eq(store_id_arg))
                    .filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query
                    .get_result::<Store>(&**self.db_conn)
                    .map_err(Error::from)
            })
    }

    /// Deactivates specific store
    fn deactivate(&self, store_id_arg: i32) -> RepoResult<Store> {
        self.execute_query(stores.find(store_id_arg))
            .and_then(|store: Store| {
                acl!(
                    [store],
                    self.acl,
                    Resource::Stores,
                    Action::Delete,
                    Some(self.db_conn)
                )
            })
            .and_then(|_| {
                let filter = stores
                    .filter(id.eq(store_id_arg))
                    .filter(is_active.eq(true));
                let query = diesel::update(filter).set(is_active.eq(false));
                self.execute_query(query)
            })
    }
}
