//! AttributeCache is a module that caches received from db information about user and his categories
use failure::Fail;
use stq_cache::cache::Cache;
use stq_types::AttributeId;

use models::Attribute;

pub struct AttributeCacheImpl<C>
where
    C: Cache<Attribute>,
{
    cache: C,
}

impl<C> AttributeCacheImpl<C>
where
    C: Cache<Attribute>,
{
    pub fn new(cache: C) -> Self {
        AttributeCacheImpl { cache }
    }

    pub fn get(&self, id: AttributeId) -> Option<Attribute> {
        debug!("Getting an attribute from AttributeCache at key '{}'", id);

        self.cache.get(id.to_string().as_str()).unwrap_or_else(|err| {
            let err = err.context(format!("Failed to get an attribute from AttributeCache at key '{}'", id));
            error!("{}", err);
            None
        })
    }

    pub fn remove(&self, id: AttributeId) -> bool {
        debug!("Removing an attribute from AttributeCache at key '{}'", id);

        self.cache.remove(id.to_string().as_str()).unwrap_or_else(|err| {
            let err = err.context(format!("Failed to remove an attribute from AttributeCache at key '{}'", id));
            error!("{}", err);
            false
        })
    }

    pub fn set(&self, id: AttributeId, attribute: Attribute) {
        debug!("Setting an attribute in AttributeCache at key '{}'", id);

        self.cache.set(id.to_string().as_str(), attribute).unwrap_or_else(|err| {
            let err = err.context(format!("Failed to set an attribute in AttributeCache at key '{}'", id));
            error!("{}", err);
        })
    }
}
