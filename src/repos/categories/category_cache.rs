//! CategoryCache is a module that caches received from db information about user and his categories
use std::sync::{Arc, Mutex};

use repos::categories::{CategoriesRepo, CategoriesRepoImpl};
use repos::types::{DbConnection, RepoResult};
use repos::acl::BoxedAcl;
use models::Category;

#[derive(Clone, Default)]
pub struct CategoryCacheImpl {
    categories_cache: Arc<Mutex<Option<Category>>>,
}

impl CategoryCacheImpl {
    pub fn get(&self, db_conn: &DbConnection, acl: BoxedAcl) -> RepoResult<Category> {
        let mut category = self.categories_cache.lock().unwrap();
        if let Some(c) = category.clone() {
            Ok(c)
        } else {
            CategoriesRepoImpl::new(db_conn, acl)
                .get_all()
                .and_then(|cat| {
                    *category = Some(cat.clone());
                    Ok(cat)
                })
        }
    }

    pub fn clear(&self) -> RepoResult<()> {
        let mut category = self.categories_cache.lock().unwrap();
        *category = None;
        Ok(())
    }
}
