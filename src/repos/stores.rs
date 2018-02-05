//! Stores repo, presents CRUD operations with db for users
use std::convert::From;
use std::cell::RefCell;

use diesel;
use diesel::prelude::*;
use diesel::select;
use diesel::dsl::exists;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;

use models::{NewStore, Store, UpdateStore};
use models::store::stores::dsl::*;
use super::error::Error;
use super::types::{DbConnection, RepoResult};
use repos::acl::Acl;
use models::authorization::*;


/// Stores repository, responsible for handling stores
pub struct StoresRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: Box<RefCell<Acl>>,
}

pub trait StoresRepo {
    /// Find specific store by ID
    fn find(&self, store_id: i32) -> RepoResult<Store>;

    /// Verifies store exist
    fn name_exists(&self, name_arg: String) -> RepoResult<bool>;

    /// Find specific store by full name
    fn find_by_name(&self, name_arg: String) -> RepoResult<Store>;

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
    pub fn new(db_conn: &'a DbConnection, acl: Box<RefCell<Acl>>) -> Self {
        Self { db_conn, acl }
    }


    fn execute_query<T: Send + 'static, U: LoadQuery<PgConnection, T> + Send + 'static>(
        &self,
        query: U,
    ) -> Result<T, Error> {
        query
            .get_result::<T>(&**self.db_conn)
            .map_err(|e| Error::from(e))
    }
}

impl<'a> StoresRepo for StoresRepoImpl<'a> {
    /// Find specific store by ID
    fn find(&self, store_id_arg: i32) -> RepoResult<Store> {
        self.execute_query(stores.find(store_id_arg))
            .and_then(|store: Store| {
                let resources = vec![(&store as &WithScope)];
                let mut acl = self.acl.borrow_mut();
                match acl.can(Resource::Stores, Action::Read, resources) {
                    true => Ok(store.clone()),
                    false => Err(Error::ContstaintViolation(
                        "Unauthorized request.".to_string(),
                    )),
                }
            })
    }

    /// Verifies store exist
    fn name_exists(&self, name_arg: String) -> RepoResult<bool> {
        self.execute_query(select(exists(stores.filter(name.eq(name_arg)))))
            .and_then(|exists| {
                let resources = vec![];
                let mut acl = self.acl.borrow_mut();
                match acl.can(Resource::Stores, Action::Read, resources) {
                    true => Ok(exists),
                    false => Err(Error::ContstaintViolation(
                        "Unauthorized request.".to_string(),
                    )),
                }
            })
    }

    /// Find specific store by full name
    fn find_by_name(&self, name_arg: String) -> RepoResult<Store> {
        let query = stores.filter(name.eq(name_arg));

        query
            .first::<Store>(&**self.db_conn)
            .map_err(|e| Error::from(e))
            .and_then(|store: Store| {
                let resources = vec![(&store as &WithScope)];
                let mut acl = self.acl.borrow_mut();
                match acl.can(Resource::Stores, Action::Read, resources) {
                    true => Ok(store.clone()),
                    false => Err(Error::ContstaintViolation(
                        "Unauthorized request.".to_string(),
                    )),
                }
            })
    }


    /// Creates new store
    fn create(&self, payload: NewStore) -> RepoResult<Store> {
        let resources = vec![(&payload as &WithScope)];
        let mut acl = self.acl.borrow_mut();
        match acl.can(Resource::Stores, Action::Write, resources) {
            true => Ok(payload.clone()),
            false => Err(Error::ContstaintViolation(
                "Unauthorized request.".to_string(),
            )),
        }.and_then(|p| {
            let query_store = diesel::insert_into(stores).values(&p);
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
            .map_err(|e| Error::from(e))
            .and_then(|stores_res: Vec<Store>| {
                let resources = stores_res
                    .iter()
                    .map(|store| (store as &WithScope))
                    .collect();
                let mut acl = self.acl.borrow_mut();
                match acl.can(Resource::Stores, Action::Read, resources) {
                    true => Ok(stores_res.clone()),
                    false => Err(Error::ContstaintViolation(
                        "Unauthorized request.".to_string(),
                    )),
                }
            })
    }

    /// Updates specific store
    fn update(&self, store_id_arg: i32, payload: UpdateStore) -> RepoResult<Store> {
        let filter = stores
            .filter(id.eq(store_id_arg))
            .filter(is_active.eq(true));

        let query = diesel::update(filter).set(&payload);
        query
            .get_result::<Store>(&**self.db_conn)
            .map_err(|e| Error::from(e))
            .and_then(|store: Store| {
                let resources = vec![(&store as &WithScope)];
                let mut acl = self.acl.borrow_mut();
                match acl.can(Resource::Stores, Action::Write, resources) {
                    true => Ok(store.clone()),
                    false => Err(Error::ContstaintViolation(
                        "Unauthorized request.".to_string(),
                    )),
                }
            })
    }

    /// Deactivates specific store
    fn deactivate(&self, store_id_arg: i32) -> RepoResult<Store> {
        let filter = stores
            .filter(id.eq(store_id_arg))
            .filter(is_active.eq(true));
        let query = diesel::update(filter).set(is_active.eq(false));
        self.execute_query(query).and_then(|store: Store| {
            let resources = vec![(&store as &WithScope)];
            let mut acl = self.acl.borrow_mut();
            match acl.can(Resource::Stores, Action::Write, resources) {
                true => Ok(store.clone()),
                false => Err(Error::ContstaintViolation(
                    "Unauthorized request.".to_string(),
                )),
            }
        })
    }
}
