//! CategoryCache is a module that caches received from db information about user and his categories
use std::sync::{Arc, Mutex};

use repos::types::RepoResult;
use models::Category;
use repos::error::RepoError;

pub trait CategoryCache: Clone + Send + 'static {
    fn get(&self) -> RepoResult<Category>;
    fn clear(&self);
    fn is_some(&self) -> bool;
    fn set(&self, cat: Category);
}

#[derive(Clone, Default)]
pub struct CategoryCacheImpl {
    inner: Arc<Mutex<Option<Category>>>,
}

impl CategoryCacheImpl {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }
}

impl CategoryCache for CategoryCacheImpl {
    fn get(&self) -> RepoResult<Category> {
        let hash_map = self.inner.lock().unwrap();
        if let Some(c) = hash_map.clone() {
            Ok(c)
        } else {
            Err(RepoError::NotFound)
        }
    }

    fn clear(&self) {
        let mut hash_map = self.inner.lock().unwrap();
        *hash_map = None;
    }

    fn is_some(&self) -> bool {
        let hash_map = self.inner.lock().unwrap();
        hash_map.is_some()
    }

    fn set(&self, cat: Category) {
        let mut hash_map = self.inner.lock().unwrap();
        *hash_map = Some(cat);
    }
}
