//! Wizard stores repo, presents CRUD operations with db for users
use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::dsl::exists;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::UserId;

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::wizard_store::wizard_stores::dsl::*;
use models::{NewWizardStore, UpdateWizardStore, WizardStore};
use repos::legacy_acl::*;

/// Wizard stores repository, responsible for handling wizard stores
pub struct WizardStoresRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, WizardStore>>,
}

pub trait WizardStoresRepo {
    /// Find specific store by user ID
    fn find_by_user_id(&self, user_id: UserId) -> RepoResult<Option<WizardStore>>;

    /// Creates new wizard store
    fn create(&self, user_id: UserId) -> RepoResult<WizardStore>;

    /// Updates specific wizard store
    fn update(&self, user_id: UserId, payload: UpdateWizardStore) -> RepoResult<WizardStore>;

    /// Delete specific wizard store
    fn delete(&self, user_id: UserId) -> RepoResult<WizardStore>;

    /// Check if the wizard already exists
    fn wizard_exists(&self, user_id: UserId) -> RepoResult<bool>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> WizardStoresRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, WizardStore>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> WizardStoresRepo
    for WizardStoresRepoImpl<'a, T>
{
    /// Find specific store by user ID
    fn find_by_user_id(&self, user_id_arg: UserId) -> RepoResult<Option<WizardStore>> {
        debug!("Find in wizard stores with user id {}.", user_id_arg);
        let query = wizard_stores.filter(user_id.eq(user_id_arg));
        query
            .get_result(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|wizard_store: Option<WizardStore>| {
                if let Some(ref wizard_store) = wizard_store {
                    acl::check(&*self.acl, Resource::WizardStores, Action::Read, self, Some(wizard_store))?;
                };
                Ok(wizard_store)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Find in wizard stores with user id {} error occured.", user_id_arg))
                    .into()
            })
    }

    /// Creates new wizard store
    fn create(&self, user_id_arg: UserId) -> RepoResult<WizardStore> {
        debug!("Create wizard store for user {:?}.", user_id_arg);
        let payload = NewWizardStore { user_id: user_id_arg };
        let query_store = diesel::insert_into(wizard_stores).values(&payload);
        query_store
            .get_result::<WizardStore>(self.db_conn)
            .map_err(From::from)
            .and_then(|wizard_store| {
                acl::check(&*self.acl, Resource::WizardStores, Action::Create, self, Some(&wizard_store)).and_then(|_| Ok(wizard_store))
            })
            .map_err(|e: FailureError| {
                e.context(format!("Create wizard store for user id {:?} error occured.", user_id_arg))
                    .into()
            })
    }

    /// Updates specific wizard store
    fn update(&self, user_id_arg: UserId, payload: UpdateWizardStore) -> RepoResult<WizardStore> {
        debug!("Updating wizard store with user_id {} and payload {:?}.", user_id_arg, payload);
        self.execute_query(wizard_stores.filter(user_id.eq(user_id_arg)))
            .and_then(|wizard_store: WizardStore| acl::check(&*self.acl, Resource::WizardStores, Action::Update, self, Some(&wizard_store)))
            .and_then(|_| {
                let filter = wizard_stores.filter(user_id.eq(user_id_arg));
                let query = diesel::update(filter).set(&payload);
                query.get_result::<WizardStore>(self.db_conn).map_err(From::from)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Updating wizard store with user_id {} and payload {:?} error occured.",
                    user_id_arg, payload
                )).into()
            })
    }

    /// Set specific wizard store completed
    fn delete(&self, user_id_arg: UserId) -> RepoResult<WizardStore> {
        debug!("Delete wizard store with user_id {}.", user_id_arg);
        self.execute_query(wizard_stores.filter(user_id.eq(user_id_arg)))
            .and_then(|wizard_store: WizardStore| acl::check(&*self.acl, Resource::WizardStores, Action::Delete, self, Some(&wizard_store)))
            .and_then(|_| {
                let filter = wizard_stores.filter(user_id.eq(user_id_arg));
                let query = diesel::update(filter).set(completed.eq(true));
                query.get_result::<WizardStore>(self.db_conn).map_err(From::from)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Delete wizard store with user_id {} error occured.", user_id_arg))
                    .into()
            })
    }

    /// Check if the wizard already exists
    fn wizard_exists(&self, user_id_arg: UserId) -> RepoResult<bool> {
        debug!("Check if wizard already exists for user {}.", user_id_arg);
        let query = diesel::select(exists(wizard_stores.filter(user_id.eq(user_id_arg))));
        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|exists| acl::check(&*self.acl, Resource::WizardStores, Action::Read, self, None).and_then(|_| Ok(exists)))
            .map_err(|e: FailureError| {
                e.context(format!("Check if wizard already exists for user {} error occured.", user_id_arg))
                    .into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, WizardStore>
    for WizardStoresRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&WizardStore>) -> bool {
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
