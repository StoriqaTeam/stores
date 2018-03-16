//! AttributeCache is a module that caches received from db information about user and his categories
use std::sync::{Arc, Mutex};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use repos::attributes::{AttributesRepo, AttributesRepoImpl};
use repos::types::{DbConnection, RepoResult};
use repos::acl::BoxedAcl;
use models::Attribute;

#[derive(Clone, Default)]
pub struct AttributeCacheImpl {
    inner: Arc<Mutex<HashMap<i32, Attribute>>>,
}

impl AttributeCacheImpl {
    pub fn get(&self, id: i32, db_conn: &DbConnection, acl: BoxedAcl) -> RepoResult<Attribute> {
        let mut hash_map = self.inner.lock().unwrap();
        match hash_map.entry(id) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(v) => AttributesRepoImpl::new(db_conn, acl)
                .find(id)
                .and_then(move |attr| {
                    v.insert(attr.clone());
                    Ok(attr)
                }),
        }
    }

    pub fn remove(&self, id: i32) -> RepoResult<()> {
        let mut hash_map = self.inner.lock().unwrap();
        hash_map.remove(&id);
        Ok(())
    }
}
