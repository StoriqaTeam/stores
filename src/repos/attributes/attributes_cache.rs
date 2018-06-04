//! AttributeCache is a module that caches received from db information about user and his categories
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex};

use models::Attribute;
use repos::error::RepoError;
use repos::types::RepoResult;

#[derive(Clone, Default)]
pub struct AttributeCacheImpl {
    inner: Arc<Mutex<HashMap<i32, Attribute>>>,
}

impl AttributeCacheImpl {
    pub fn get(&self, id: i32) -> RepoResult<Attribute> {
        let mut hash_map = self.inner.lock().unwrap();
        match hash_map.entry(id) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(_) => Err(RepoError::NotFound),
        }
    }

    pub fn contains(&self, id: i32) -> bool {
        let hash_map = self.inner.lock().unwrap();
        hash_map.contains_key(&id)
    }

    pub fn add_attribute(&self, id: i32, attribute: Attribute) {
        let mut hash_map = self.inner.lock().unwrap();
        hash_map.insert(id, attribute);
    }

    pub fn remove(&self, id: i32) {
        let mut hash_map = self.inner.lock().unwrap();
        hash_map.remove(&id);
    }
}
