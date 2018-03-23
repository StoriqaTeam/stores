//! Stores repo, presents CRUD operations with db for users
use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::dsl::exists;

use stq_acl::*;
use stq_static_resources::Translation;

use models::{NewStore, Store, UpdateStore};
use models::store::stores::dsl::*;
use super::error::RepoError as Error;
use super::types::{DbConnection, RepoResult};
use models::authorization::*;
use super::acl;
use super::acl::BoxedAcl;

/// Stores repository, responsible for handling stores
pub struct StoresRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: BoxedAcl,
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

    /// Checks that slug already exists
    fn slug_exists(&self, slug_arg: String) -> RepoResult<bool>;

    /// Checks name exists
    fn name_exists(&self, name: Vec<Translation>) -> RepoResult<bool>;
}

impl<'a> StoresRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl: BoxedAcl) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<T: Send + 'static, U: LoadQuery<DbConnection, T> + Send + 'static>(&self, query: U) -> Result<T, Error> {
        query.get_result::<T>(self.db_conn).map_err(Error::from)
    }
}

impl<'a> StoresRepo for StoresRepoImpl<'a> {
    /// Find specific store by ID
    fn find(&self, store_id_arg: i32) -> RepoResult<Store> {
        self.execute_query(stores.find(store_id_arg))
            .and_then(|store: Store| {
                acl::check(
                    &*self.acl,
                    &Resource::Stores,
                    &Action::Read,
                    &[&store],
                    Some(self.db_conn),
                ).and_then(|_| Ok(store))
            })
    }

    /// Creates new store
    fn create(&self, payload: NewStore) -> RepoResult<Store> {
        acl::check(
            &*self.acl,
            &Resource::Stores,
            &Action::Create,
            &[&payload],
            Some(self.db_conn),
        ).and_then(|_| {
            let query_store = diesel::insert_into(stores).values(&payload);
            query_store
                .get_result::<Store>(self.db_conn)
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
            .get_results(self.db_conn)
            .map_err(Error::from)
            .and_then(|stores_res: Vec<Store>| {
                let resources = stores_res
                    .iter()
                    .map(|store| (store as &WithScope<Scope>))
                    .collect::<Vec<&WithScope<Scope>>>();
                acl::check(
                    &*self.acl,
                    &Resource::Stores,
                    &Action::Read,
                    &resources,
                    Some(self.db_conn),
                ).and_then(|_| Ok(stores_res.clone()))
            })
    }

    /// Updates specific store
    fn update(&self, store_id_arg: i32, payload: UpdateStore) -> RepoResult<Store> {
        self.execute_query(stores.find(store_id_arg))
            .and_then(|store: Store| {
                acl::check(
                    &*self.acl,
                    &Resource::Stores,
                    &Action::Update,
                    &[&store],
                    Some(self.db_conn),
                )
            })
            .and_then(|_| {
                let filter = stores
                    .filter(id.eq(store_id_arg))
                    .filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<Store>(self.db_conn).map_err(Error::from)
            })
    }

    /// Deactivates specific store
    fn deactivate(&self, store_id_arg: i32) -> RepoResult<Store> {
        self.execute_query(stores.find(store_id_arg))
            .and_then(|store: Store| {
                acl::check(
                    &*self.acl,
                    &Resource::Stores,
                    &Action::Delete,
                    &[&store],
                    Some(self.db_conn),
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

    fn slug_exists(&self, slug_arg: String) -> RepoResult<bool> {
        let query = diesel::select(exists(stores.filter(slug.eq(slug_arg))));

        query
            .get_result(self.db_conn)
            .map_err(Error::from)
            .and_then(|exists| {
                acl::check(
                    &*self.acl,
                    &Resource::Stores,
                    &Action::Read,
                    &[],
                    Some(self.db_conn),
                ).and_then(|_| Ok(exists))
            })
    }

    /// Checks name exists
    fn name_exists(&self, name_arg: Vec<Translation>) -> RepoResult<bool> {
        let res = name_arg
            .into_iter()
            .map(|trans| {
                let query_str = format!(
                    "SELECT EXISTS ( SELECT 1 FROM stores WHERE name @> '[{{\"lang\": \"{}\", \"text\": \"{}\"}}]');",
                    trans.lang, trans.text
                );
                diesel::dsl::sql::<(diesel::sql_types::Bool)>(&query_str)
                    .get_result(self.db_conn)
                    .map_err(Error::from)
            })
            .collect::<RepoResult<Vec<bool>>>();

        res.and_then(|res| Ok(res.into_iter().all(|t| t)))
    }
}
