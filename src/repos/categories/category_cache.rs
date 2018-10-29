//! CategoryCache is a module that caches received from db information about user and his categories
use failure::Fail;
use stq_cache::cache::CacheSingle;

use models::Category;

pub struct CategoryCacheImpl<C>
where
    C: CacheSingle<Category>,
{
    cache: C,
}

impl<C> CategoryCacheImpl<C>
where
    C: CacheSingle<Category>,
{
    pub fn new(cache: C) -> Self {
        CategoryCacheImpl { cache }
    }

    pub fn get(&self) -> Option<Category> {
        debug!("Getting category from CategoryCache");

        self.cache.get().unwrap_or_else(|err| {
            error!("{}", err.context("Failed to get category from CategoryCache"));
            None
        })
    }

    pub fn remove(&self) -> bool {
        debug!("Removing category from CategoryCache");

        self.cache.remove().unwrap_or_else(|err| {
            error!("{}", err.context("Failed to remove category from CategoryCache"));
            false
        })
    }

    pub fn set(&self, cat: Category) {
        debug!("Setting category in CategoryCache");

        self.cache.set(cat).unwrap_or_else(|err| {
            error!("{}", err.context("Failed to set category in CategoryCache"));
        })
    }
}
