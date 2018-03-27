//! Stores repo, presents CRUD operations with db for users
use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::dsl::exists;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use stq_acl::*;
use stq_static_resources::Translation;

use models::{NewStore, Store, UpdateStore};
use models::store::stores::dsl::*;
use super::error::RepoError as Error;
use super::types::RepoResult;
use models::authorization::*;
use super::acl;

/// Stores repository, responsible for handling stores
pub struct StoresRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, Error, Store>>,
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

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> StoresRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, Error, Store>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(Error::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> StoresRepo for StoresRepoImpl<'a, T> {
    /// Find specific store by ID
    fn find(&self, store_id_arg: i32) -> RepoResult<Store> {
        self.execute_query(stores.find(store_id_arg))
            .and_then(|store: Store| {
                acl::check(
                    &*self.acl,
                    &Resource::Stores,
                    &Action::Read,
                    self,
                    Some(&store),
                ).and_then(|_| Ok(store))
            })
    }

    /// Creates new store
    fn create(&self, payload: NewStore) -> RepoResult<Store> {
        let query_store = diesel::insert_into(stores).values(&payload);
        query_store
            .get_result::<Store>(self.db_conn)
            .map_err(Error::from)
            .and_then(|store| {
                acl::check(
                    &*self.acl,
                    &Resource::Stores,
                    &Action::Create,
                    self,
                    Some(&store),
                ).and_then(|_| Ok(store))
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
                for store in stores_res.iter() {
                    acl::check(
                        &*self.acl,
                        &Resource::Stores,
                        &Action::Read,
                        self,
                        Some(&store),
                    )?;
                }
                Ok(stores_res.clone())
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
                    self,
                    Some(&store),
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
                    self,
                    Some(&store),
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
            .and_then(|exists| acl::check(&*self.acl, &Resource::Stores, &Action::Read, self, None).and_then(|_| Ok(exists)))
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
            .and_then(|exists| acl::check(&*self.acl, &Resource::Stores, &Action::Read, self, None).and_then(|_| Ok(exists)))
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Store>
    for StoresRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: i32, scope: &Scope, obj: Option<&Store>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(store) = obj {
                    store.user_id == user_id_arg
                } else {
                    false
                }
            }
        }
    }
}
