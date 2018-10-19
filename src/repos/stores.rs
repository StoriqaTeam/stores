//! Stores repo, presents CRUD operations with db for users
use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::dsl::exists;
use diesel::dsl::sql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_static_resources::{ModerationStatus, Translation};
use stq_types::{StoreId, UserId};

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::{ModeratorStoreSearchTerms, NewStore, Store, UpdateStore};
use repos::legacy_acl::*;
use schema::base_products::dsl as BaseProducts;
use schema::products::dsl as Products;
use schema::stores::dsl::*;

/// Stores repository, responsible for handling stores
pub struct StoresRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Store>>,
}

pub trait StoresRepo {
    /// Get store count
    fn count(&self, only_active: bool) -> RepoResult<i64>;

    /// Find specific store by ID
    fn find(&self, store_id: StoreId) -> RepoResult<Option<Store>>;

    /// Returns list of stores, limited by `from` and `count` parameters
    fn list(&self, from: StoreId, count: i32) -> RepoResult<Vec<Store>>;

    /// Creates new store
    fn create(&self, payload: NewStore) -> RepoResult<Store>;

    /// Updates specific store
    fn update(&self, store_id: StoreId, payload: UpdateStore) -> RepoResult<Store>;

    /// Deactivates specific store
    fn deactivate(&self, store_id: StoreId) -> RepoResult<Store>;

    /// Delete store by user id
    fn delete_by_user(&self, user_id_arg: UserId) -> RepoResult<Option<Store>>;

    /// Get store by user id
    fn get_by_user(&self, user_id_arg: UserId) -> RepoResult<Option<Store>>;

    /// Checks that slug already exists
    fn slug_exists(&self, slug_arg: String) -> RepoResult<bool>;

    /// Checks name exists
    fn name_exists(&self, name: Vec<Translation>) -> RepoResult<bool>;

    /// Checks if vendor code exists across the store
    fn vendor_code_exists(&self, store_id: StoreId, vendor_code: &str) -> RepoResult<Option<bool>>;

    /// Search stores limited by `from`, `skip` and `count` parameters
    fn moderator_search(&self, from: Option<StoreId>, skip: i64, count: i64, term: ModeratorStoreSearchTerms) -> RepoResult<Vec<Store>>;

    /// Set moderation status for specific store
    fn set_moderation_status(&self, store_id: StoreId, status: ModerationStatus) -> RepoResult<Store>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> StoresRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Store>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> StoresRepo for StoresRepoImpl<'a, T> {
    /// Get store count
    fn count(&self, only_active: bool) -> RepoResult<i64> {
        let query = if only_active {
            stores.filter(is_active.eq(true)).into_boxed()
        } else {
            stores.into_boxed()
        };

        acl::check(&*self.acl, Resource::Stores, Action::Read, self, None)
            .and_then(|_| query.count().get_result(self.db_conn).map_err(From::from))
            .map_err(|e| FailureError::from(e).context("Count stores error occurred").into())
    }

    /// Find specific store by ID
    fn find(&self, store_id_arg: StoreId) -> RepoResult<Option<Store>> {
        debug!("Find in stores with id {}.", store_id_arg);
        let query = stores.find(store_id_arg).filter(is_active.eq(true));
        query
            .get_result(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|store: Option<Store>| {
                if let Some(ref store) = store {
                    acl::check(&*self.acl, Resource::Stores, Action::Read, self, Some(store))?;
                };
                Ok(store)
            }).map_err(|e: FailureError| e.context(format!("Find store with id: {} error occurred", store_id_arg)).into())
    }

    /// Creates new store
    fn create(&self, payload: NewStore) -> RepoResult<Store> {
        debug!("Create store {:?}.", payload);
        let query_store = diesel::insert_into(stores).values(&payload);
        query_store
            .get_result::<Store>(self.db_conn)
            .map_err(From::from)
            .and_then(|store| acl::check(&*self.acl, Resource::Stores, Action::Create, self, Some(&store)).and_then(|_| Ok(store)))
            .map_err(|e: FailureError| e.context(format!("Create store {:?} error occurred.", payload)).into())
    }

    /// Returns list of stores, limited by `from` and `count` parameters
    fn list(&self, from: StoreId, count: i32) -> RepoResult<Vec<Store>> {
        debug!("Find in stores from {} count {}.", from, count);
        let query = stores.filter(is_active.eq(true)).filter(id.gt(from)).order(id).limit(count.into());

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|stores_res: Vec<Store>| {
                for store in &stores_res {
                    acl::check(&*self.acl, Resource::Stores, Action::Read, self, Some(&store))?;
                }
                Ok(stores_res.clone())
            }).map_err(|e: FailureError| {
                e.context(format!("Find in stores from {} count {} error occurred.", from, count))
                    .into()
            })
    }

    /// Updates specific store
    fn update(&self, store_id_arg: StoreId, payload: UpdateStore) -> RepoResult<Store> {
        debug!("Updating store with id {} and payload {:?}.", store_id_arg, payload);
        self.execute_query(stores.find(store_id_arg))
            .and_then(|store: Store| acl::check(&*self.acl, Resource::Stores, Action::Update, self, Some(&store)))
            .and_then(|_| {
                let filter = stores.filter(id.eq(store_id_arg)).filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<Store>(self.db_conn).map_err(From::from)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "Updating store with id {} and payload {:?} error occurred.",
                    store_id_arg, payload
                )).into()
            })
    }

    /// Deactivates specific store
    fn deactivate(&self, store_id_arg: StoreId) -> RepoResult<Store> {
        debug!("Deactivate store with id {}.", store_id_arg);
        self.execute_query(stores.find(store_id_arg))
            .and_then(|store: Store| acl::check(&*self.acl, Resource::Stores, Action::Delete, self, Some(&store)))
            .and_then(|_| {
                let filter = stores.filter(id.eq(store_id_arg)).filter(is_active.eq(true));
                let query = diesel::update(filter).set(is_active.eq(false));
                self.execute_query(query)
            }).map_err(|e: FailureError| {
                e.context(format!("Deactivate store with id {} error occurred.", store_id_arg))
                    .into()
            })
    }

    /// Delete store by user id
    fn delete_by_user(&self, user_id_arg: UserId) -> RepoResult<Option<Store>> {
        debug!("Delete store by user id {}.", user_id_arg);
        let query = stores.filter(user_id.eq(user_id_arg));

        query
            .get_result(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|store_res: Option<Store>| {
                if let Some(store_res) = store_res {
                    acl::check(&*self.acl, Resource::Stores, Action::Delete, self, Some(&store_res))?;
                    let filter = stores.filter(user_id.eq(user_id_arg));
                    let query = diesel::update(filter).set(is_active.eq(false));
                    self.execute_query(query).map(Some).map_err(From::from)
                } else {
                    Ok(None)
                }
            }).map_err(|e: FailureError| e.context(format!("Delete store by user id {}.", user_id_arg)).into())
    }

    /// Get store by user id
    fn get_by_user(&self, user_id_arg: UserId) -> RepoResult<Option<Store>> {
        debug!("get store by user id {}.", user_id_arg);
        let query = stores.filter(user_id.eq(user_id_arg)).filter(is_active.eq(true));

        query
            .get_result(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|store_res: Option<Store>| {
                if let Some(ref store_res) = store_res {
                    acl::check(&*self.acl, Resource::Stores, Action::Read, self, Some(store_res))?;
                };
                Ok(store_res)
            }).map_err(|e: FailureError| e.context(format!("Get store by user id {}.", user_id_arg)).into())
    }

    /// Checks slug exists
    fn slug_exists(&self, slug_arg: String) -> RepoResult<bool> {
        debug!("Check if store slug {} exists.", slug_arg);
        let query = diesel::select(exists(stores.filter(slug.eq(slug_arg.clone())).filter(is_active.eq(true))));
        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|exists| acl::check(&*self.acl, Resource::Stores, Action::Read, self, None).and_then(|_| Ok(exists)))
            .map_err(move |e: FailureError| e.context(format!("Store slug exists {} error occurred.", slug_arg)).into())
    }

    /// Checks name exists
    fn name_exists(&self, name_arg: Vec<Translation>) -> RepoResult<bool> {
        debug!("Check if store name {:?} exists.", name_arg);
        let res = name_arg
            .clone()
            .into_iter()
            .map(|trans| {
                let query_str = format!(
                    "SELECT EXISTS ( SELECT 1 FROM stores WHERE name @> '[{{\"lang\": \"{}\", \"text\": \"{}\"}}]');",
                    trans.lang, trans.text
                );
                diesel::dsl::sql::<(diesel::sql_types::Bool)>(&query_str)
                    .get_result(self.db_conn)
                    .map_err(From::from)
            }).collect::<RepoResult<Vec<bool>>>();

        res.and_then(|res| Ok(res.into_iter().all(|t| t)))
            .and_then(|exists| acl::check(&*self.acl, Resource::Stores, Action::Read, self, None).and_then(|_| Ok(exists)))
            .map_err(move |e: FailureError| e.context(format!("Store name exists {:?} error occurred.", name_arg)).into())
    }

    /// Checks if vendor code exists across the store
    fn vendor_code_exists(&self, store_id: StoreId, vendor_code: &str) -> RepoResult<Option<bool>> {
        debug!("Check if vendor code '{}' exists for store '{}'", vendor_code, store_id);

        {
            if self.find(store_id)?.is_none() {
                return Ok(None);
            }

            let vendor_code_exists_query = diesel::select(exists(
                BaseProducts::base_products.inner_join(Products::products).filter(
                    BaseProducts::is_active
                        .eq(true)
                        .and(BaseProducts::store_id.eq(store_id))
                        .and(Products::is_active.eq(true))
                        .and(Products::vendor_code.eq(vendor_code)),
                ),
            ));

            vendor_code_exists_query
                .get_result::<bool>(self.db_conn)
                .map(Some)
                .map_err(From::from)
        }.map_err(move |e: FailureError| {
            let msg = format!("Vendor code '{}' exists in store '{}' error occurred.", vendor_code, store_id);
            e.context(msg).into()
        })
    }

    /// Search stores limited by `from`, `skip` and `count` parameters
    fn moderator_search(&self, from: Option<StoreId>, skip: i64, count: i64, term: ModeratorStoreSearchTerms) -> RepoResult<Vec<Store>> {
        let mut query = stores.into_boxed();

        if let Some(from_id) = from {
            query = query.filter(id.ge(from_id));
        }
        if skip > 0 {
            query = query.offset(skip);
        }
        if count > 0 {
            query = query.limit(count);
        }

        if let Some(term_name) = term.name {
            query = query.filter(sql(format!("name::text like '%{}%'", term_name).as_ref()));
        }

        if let Some(ref store_manager_ids) = term.store_manager_ids {
            query = query.filter(user_id.eq_any(store_manager_ids));
        }

        if let Some(term_state) = term.state {
            query = query.filter(status.eq(term_state));
        }

        query
            .order(id)
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|stores_res: Vec<Store>| {
                for store in &stores_res {
                    acl::check(&*self.acl, Resource::Stores, Action::Read, self, Some(&store))?;
                }

                Ok(stores_res)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "moderator search for stores error occurred (from id: {:?}, skip: {}, count: {})",
                    from, skip, count
                )).into()
            })
    }

    /// Set moderation status for specific store
    fn set_moderation_status(&self, store_id_arg: StoreId, status_arg: ModerationStatus) -> RepoResult<Store> {
        let query = stores.find(store_id_arg);

        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|s: Store| acl::check(&*self.acl, Resource::Stores, Action::Moderate, self, Some(&s)))
            .and_then(|_| {
                let filter = stores.filter(id.eq(store_id_arg));
                let query = diesel::update(filter).set(status.eq(status_arg));

                query.get_result(self.db_conn).map_err(From::from)
            }).map_err(|e: FailureError| {
                e.context(format!("Set moderation status for store {:?} error occurred", store_id_arg))
                    .into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Store>
    for StoresRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&Store>) -> bool {
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
