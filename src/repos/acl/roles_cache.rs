//! RolesCache is a module that caches received from db information about user and his roles
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use models::authorization::*;
use stq_acl::RolesCache;

#[derive(Default, Clone)]
pub struct RolesCacheImpl {
    roles_cache: Arc<Mutex<HashMap<i32, Vec<Role>>>>,
}

impl RolesCache for RolesCacheImpl {
    type Role = Role;

    fn get(&self, user_id: i32) -> Vec<Self::Role> {
        let mut hash_map = self.roles_cache.lock().unwrap();
        match hash_map.entry(user_id) {
            Entry::Occupied(o) => o.get().clone(),
            Entry::Vacant(_) => vec![],
        }
    }

    fn clear(&self) {
        let mut hash_map = self.roles_cache.lock().unwrap();
        hash_map.clear();
    }

    fn remove(&self, id: i32) {
        let mut hash_map = self.roles_cache.lock().unwrap();
        hash_map.remove(&id);
    }

    fn contains(&self, id: i32) -> bool {
        let hash_map = self.roles_cache.lock().unwrap();
        hash_map.contains_key(&id)
    }

    fn add_roles(&self, id: i32, roles: &Vec<Self::Role>) {
        let mut hash_map = self.roles_cache.lock().unwrap();
        hash_map.insert(id, roles.clone());
    }
}
