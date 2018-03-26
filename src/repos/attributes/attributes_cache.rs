//! AttributeCache is a module that caches received from db information about user and his categories
use std::sync::{Arc, Mutex};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use stq_acl::RolesCache;

use models::Attribute;
use models::authorization::*;
use repos::error::RepoError;
use repos::ReposFactory;
use repos::types::RepoResult;

pub trait AttributeCache: Clone + Send + 'static {
    fn get<
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        id: i32,
        db_conn: &C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> RepoResult<Attribute>;
    fn remove(&self, id: i32) -> RepoResult<()>;
}

#[derive(Clone, Default)]
pub struct AttributeCacheImpl<F: ReposFactory> {
    inner: Arc<Mutex<HashMap<i32, Attribute>>>,
    repo_factory: F,
}

impl<F: ReposFactory> AttributeCacheImpl<F> {
    pub fn new(repo_factory: F) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            repo_factory,
        }
    }
}

impl<F: ReposFactory> AttributeCache for AttributeCacheImpl<F> {
    fn get<
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        id: i32,
        db_conn: &C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> RepoResult<Attribute> {
        let mut hash_map = self.inner.lock().unwrap();
        match hash_map.entry(id) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(v) => self.repo_factory
                .create_attributes_repo(db_conn, roles_cache, user_id)
                .find(id)
                .and_then(move |attr| {
                    v.insert(attr.clone());
                    Ok(attr)
                }),
        }
    }

    fn remove(&self, id: i32) -> RepoResult<()> {
        let mut hash_map = self.inner.lock().unwrap();
        hash_map.remove(&id);
        Ok(())
    }
}
