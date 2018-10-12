//! AttributeCache is a module that caches received from db information about user and his categories
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use stq_types::AttributeId;

use models::Attribute;

#[derive(Clone, Default)]
pub struct AttributeCacheImpl {
    inner: Arc<Mutex<HashMap<AttributeId, Attribute>>>,
}

impl AttributeCacheImpl {
    pub fn get(&self, _id: AttributeId) -> Option<Attribute> {
        //self.inner.lock().unwrap().get(&id).cloned()
        None
    }

    pub fn contains(&self, _id: AttributeId) -> bool {
        //let hash_map = self.inner.lock().unwrap();
        //hash_map.contains_key(&id)
        false
    }

    pub fn add_attribute(&self, _id: AttributeId, _attribute: Attribute) {
        //let mut hash_map = self.inner.lock().unwrap();
        //hash_map.insert(id, attribute);
    }

    pub fn remove(&self, _id: AttributeId) {
        //let mut hash_map = self.inner.lock().unwrap();
        //hash_map.remove(&id);
    }
}
