//! AttributeCache is a module that caches received from db information about user and his categories
use std::sync::{Arc, Mutex};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use models::Attribute;
use repos::error::RepoError;
use repos::types::RepoResult;

pub trait AttributeCache: Clone + Send + 'static {
    fn get(&self, id: i32) -> RepoResult<Attribute>;
    fn remove(&self, id: i32);
    fn contains(&self, id: i32) -> bool;
    fn add_attribute(&self, id: i32, attribute: Attribute);
}

#[derive(Clone, Default)]
pub struct AttributeCacheImpl {
    inner: Arc<Mutex<HashMap<i32, Attribute>>>,
}

impl AttributeCache for AttributeCacheImpl {
    fn get(&self, id: i32) -> RepoResult<Attribute> {
        let mut hash_map = self.inner.lock().unwrap();
        match hash_map.entry(id) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(_) => return Err(RepoError::NotFound),
        }
    }

    fn contains(&self, id: i32) -> bool {
        let hash_map = self.inner.lock().unwrap();
        hash_map.contains_key(&id)
    }

    fn add_attribute(&self, id: i32, attribute: Attribute) {
        let mut hash_map = self.inner.lock().unwrap();
        hash_map.insert(id, attribute);
    }

    fn remove(&self, id: i32) {
        let mut hash_map = self.inner.lock().unwrap();
        hash_map.remove(&id);
    }
}
