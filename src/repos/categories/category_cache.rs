//! CategoryCache is a module that caches received from db information about user and his categories
use std::sync::{Arc, Mutex};

use stq_acl::RolesCache;

use repos::types::{DbConnection, RepoResult};
use repos::ReposFactory;
use models::Category;
use models::authorization::*;
use repos::error::RepoError;

pub trait CategoryCache: Clone + Send + 'static {
    fn get<T: RolesCache<Role = Role, Error = RepoError> + 'static>(
        &self,
        db_conn: &DbConnection,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> RepoResult<Category>;
    fn clear(&self) -> RepoResult<()>;
}

#[derive(Clone)]
pub struct CategoryCacheImpl<F: ReposFactory> {
    categories_cache: Arc<Mutex<Option<Category>>>,
    repo_factory: F,
}

impl<F: ReposFactory> CategoryCacheImpl<F> {
    pub fn new(repo_factory: F) -> Self {
        Self {
            categories_cache: Arc::new(Mutex::new(None)),
            repo_factory,
        }
    }
}

impl<F: ReposFactory> CategoryCache for CategoryCacheImpl<F> {
    fn get<T: RolesCache<Role = Role, Error = RepoError> + 'static>(
        &self,
        db_conn: &DbConnection,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> RepoResult<Category> {
        let mut category = self.categories_cache.lock().unwrap();
        if let Some(c) = category.clone() {
            Ok(c)
        } else {
            self.repo_factory
                .create_categories_repo(db_conn, roles_cache, user_id)
                .get_all()
                .and_then(|cat| {
                    *category = Some(cat.clone());
                    Ok(cat)
                })
        }
    }

    fn clear(&self) -> RepoResult<()> {
        let mut category = self.categories_cache.lock().unwrap();
        *category = None;
        Ok(())
    }
}
