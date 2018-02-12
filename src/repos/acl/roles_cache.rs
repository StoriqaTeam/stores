//! RolesCache is a module that caches received from db information about user and his roles
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use repos::user_roles::{UserRolesRepo, UserRolesRepoImpl};
use repos::types::{DbConnection, RepoResult};
use models::authorization::*;
use repos::acl::SystemACL;


#[derive(Clone)]
pub struct RolesCacheImpl {
    roles_cache: Arc<Mutex<HashMap<i32, Vec<Role>>>>,
}

impl RolesCacheImpl {
    pub fn new() -> Self {
        RolesCacheImpl {
            roles_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}


pub trait RolesCache {
    fn get(&mut self, id: i32, db_conn: &DbConnection) -> RepoResult<Vec<Role>>;
}

impl RolesCache for RolesCacheImpl {
    fn get(&mut self, id: i32, db_conn: &DbConnection) -> RepoResult<Vec<Role>> {
        let hash_map = self.roles_cache.lock().unwrap();
        if let Some(vec) = hash_map.get(&id) {
            Ok(vec.clone())
        } else {
            let roles = self.roles_cache.clone();
            let repo =
                UserRolesRepoImpl::new(db_conn, Box::new(SystemACL::new()));
            repo.list_for_user(id)
                .map(|users| users.into_iter().map(|u| u.role).collect())
                .and_then(|vec: Vec<Role>| {
                    let mut hash_map = roles.lock().unwrap();
                    hash_map.insert(id, vec.clone());
                    Ok(vec)
                })
        }
    }
}
