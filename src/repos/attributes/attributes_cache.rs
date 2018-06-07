//! AttributeCache is a module that caches received from db information about user and his categories
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use models::Attribute;

#[derive(Clone, Default)]
pub struct AttributeCacheImpl {
    inner: Arc<Mutex<HashMap<i32, Attribute>>>,
}

impl AttributeCacheImpl {
    pub fn get(&self, id: i32) -> Option<Attribute> {
        self.inner.lock().unwrap().get(&id).cloned()
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
