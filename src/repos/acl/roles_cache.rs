//! RolesCache is a module that caches received from db information about user and his roles
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use repos::types::DbConnection;
use models::authorization::*;
use stq_acl::RolesCache;
use repos::error::RepoError as Error;
use repos::ReposFactory;

#[derive(Clone)]
pub struct RolesCacheImpl<F: ReposFactory> {
    roles_cache: Arc<Mutex<HashMap<i32, Vec<Role>>>>,
    repo_factory: F,
}

impl<F: ReposFactory> RolesCacheImpl<F> {
    pub fn new (repo_factory: F) -> Self {
        Self {
            roles_cache: Arc::new(Mutex::new(HashMap::new())),
            repo_factory
        }
    }
}

impl<F: ReposFactory> RolesCache for RolesCacheImpl<F> {
    type Role = Role;
    type Error = Error;

    fn get(&self, user_id: i32, db_conn: Option<&DbConnection>) -> Result<Vec<Self::Role>, Self::Error> {
        let mut hash_map = self.roles_cache.lock().unwrap();
        let repo_factory = self.repo_factory;
        match hash_map.entry(user_id) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(v) => db_conn
                .ok_or(Error::Connection(format_err!("No connection to db")))
                .and_then(|con| {
                    repo_factory.create_user_roles_repo(con).list_for_user(user_id)
                })
                .and_then(move |vec: Vec<Role>| {
                    v.insert(vec.clone());
                    Ok(vec)
                }),
        }
    }

    fn clear(&self) -> Result<(), Self::Error> {
        let mut hash_map = self.roles_cache.lock().unwrap();
        hash_map.clear();
        Ok(())
    }

    fn remove(&self, id: i32) -> Result<(), Self::Error> {
        let mut hash_map = self.roles_cache.lock().unwrap();
        hash_map.remove(&id);
        Ok(())
    }
}
