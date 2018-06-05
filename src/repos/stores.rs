//! Stores repo, presents CRUD operations with db for users
use diesel;
use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::dsl::exists;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use failure::Fail;
use failure::Error as FailureError;

use stq_acl::*;
use stq_static_resources::Translation;

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::store::stores::dsl::*;
use models::{NewStore, Store, UpdateStore};

/// Stores repository, responsible for handling stores
pub struct StoresRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Store>>,
}

pub trait StoresRepo {
    /// Find specific store by ID
    fn find(&self, store_id: i32) -> RepoResult<Option<Store>>;

    /// Returns list of stores, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> RepoResult<Vec<Store>>;

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
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Store>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(|e| e.into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> StoresRepo for StoresRepoImpl<'a, T> {
    /// Find specific store by ID
    fn find(&self, store_id_arg: i32) -> RepoResult<Option<Store>> {
        debug!("Find in stores with id {}.", store_id_arg);
        let query = stores.find(store_id_arg).filter(is_active.eq(true));
        query
            .get_result(self.db_conn)
            .optional()
            .map_err(|e| e.into())
            .and_then(|store: Option<Store>| {
                if let Some(ref store) = store {
                    acl::check(&*self.acl, &Resource::Stores, &Action::Read, self, Some(store))?;
                };
                Ok(store)
            })
            .map_err(|e: FailureError| e.context(format!("Find store with id: {} error occured", store_id_arg)).into())
   
    }

    /// Creates new store
    fn create(&self, payload: NewStore) -> RepoResult<Store> {
        debug!("Create store {:?}.", payload);
        let query_store = diesel::insert_into(stores).values(&payload);
        query_store
            .get_result::<Store>(self.db_conn)
            .map_err(|e| e.into())
            .and_then(|store| acl::check(&*self.acl, &Resource::Stores, &Action::Create, self, Some(&store)).and_then(|_| Ok(store)))
            .map_err(|e: FailureError| e.context(format!("Create store {:?} error occured.", payload)).into())
    }

    /// Returns list of stores, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> RepoResult<Vec<Store>> {
        debug!("Find in stores from {} count {}.", from, count);
        let query = stores.filter(is_active.eq(true)).filter(id.gt(from)).order(id).limit(count.into());

        query
            .get_results(self.db_conn)
            .map_err(|e| e.into())
            .and_then(|stores_res: Vec<Store>| {
                for store in &stores_res {
                    acl::check(&*self.acl, &Resource::Stores, &Action::Read, self, Some(&store))?;
                }
                Ok(stores_res.clone())
            })
            .map_err(|e: FailureError| e.context(format!("Find in stores from {} count {} error occured.", from, count)).into())
    }

    /// Updates specific store
    fn update(&self, store_id_arg: i32, payload: UpdateStore) -> RepoResult<Store> {
        debug!("Updating store with id {} and payload {:?}.", store_id_arg, payload);
        self.execute_query(stores.find(store_id_arg))
            .and_then(|store: Store| acl::check(&*self.acl, &Resource::Stores, &Action::Update, self, Some(&store)))
            .and_then(|_| {
                let filter = stores.filter(id.eq(store_id_arg)).filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<Store>(self.db_conn)
                .map_err(|e| e.into())
            })
            .map_err(|e: FailureError| e.context(format!("Updating store with id {} and payload {:?} error occured.", store_id_arg, payload)).into())
    }

    /// Deactivates specific store
    fn deactivate(&self, store_id_arg: i32) -> RepoResult<Store> {
        debug!("Deactivate store with id {}.", store_id_arg);
        self.execute_query(stores.find(store_id_arg))
            .and_then(|store: Store| acl::check(&*self.acl, &Resource::Stores, &Action::Delete, self, Some(&store)))
            .and_then(|_| {
                let filter = stores.filter(id.eq(store_id_arg)).filter(is_active.eq(true));
                let query = diesel::update(filter).set(is_active.eq(false));
                self.execute_query(query)
            })
            .map_err(|e: FailureError| e.context(format!("Deactivate store with id {} error occured.", store_id_arg)).into())
    }

    fn slug_exists(&self, slug_arg: String) -> RepoResult<bool> {
        debug!("Check if store slug {} exists.", slug_arg);
        let query = diesel::select(exists(stores.filter(slug.eq(slug_arg))));
        query
            .get_result(self.db_conn)
            .map_err(|e| e.into())
            .and_then(|exists| acl::check(&*self.acl, &Resource::Stores, &Action::Read, self, None).and_then(|_| Ok(exists)))
            .map_err(|e: FailureError| e.context(format!("Store slug exists {} error occured.", slug_arg)).into())
    }

    /// Checks name exists
    fn name_exists(&self, name_arg: Vec<Translation>) -> RepoResult<bool> {
        debug!("Check if store name {:?} exists.", name_arg);
        let res = name_arg
            .into_iter()
            .map(|trans| {
                let query_str = format!(
                    "SELECT EXISTS ( SELECT 1 FROM stores WHERE name @> '[{{\"lang\": \"{}\", \"text\": \"{}\"}}]');",
                    trans.lang, trans.text
                );
                diesel::dsl::sql::<(diesel::sql_types::Bool)>(&query_str)
                    .get_result(self.db_conn)
                    .map_err(|e| e.into())
            })
            .collect::<RepoResult<Vec<bool>>>();

        res.and_then(|res| Ok(res.into_iter().all(|t| t)))
            .and_then(|exists| acl::check(&*self.acl, &Resource::Stores, &Action::Read, self, None).and_then(|_| Ok(exists)))
            .map_err(|e: FailureError| e.context(format!("Store name exists {:?} error occured.", name_arg)).into())
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
