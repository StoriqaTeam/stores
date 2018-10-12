//! CategoryCache is a module that caches received from db information about user and his categories
use std::sync::{Arc, Mutex};

use models::Category;

#[derive(Clone, Default)]
pub struct CategoryCacheImpl {
    inner: Arc<Mutex<Option<Category>>>,
}

impl CategoryCacheImpl {
    pub fn get(&self) -> Option<Category> {
        //let category = self.inner.lock().unwrap();
        //category.clone()
        None
    }

    pub fn clear(&self) {
        //let mut category = self.inner.lock().unwrap();
        //*category = None;
    }

    pub fn is_some(&self) -> bool {
        //let category = self.inner.lock().unwrap();
        //category.is_some()
        false
    }

    pub fn set(&self, _cat: Category) {
        //let mut category = self.inner.lock().unwrap();
        //*category = Some(cat);
    }
}
