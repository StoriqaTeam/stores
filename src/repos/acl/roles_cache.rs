//! RolesCache is a module that caches received from db information about user and his roles
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures::Future;
use futures_cpupool::CpuPool;

use repos::user_roles::{UserRolesRepo, UserRolesRepoImpl};
use repos::types::DbPool;
use repos::error::Error;
use models::authorization::*;
use repos::acl::SystemACL;


#[derive(Clone)]
pub struct RolesCacheImpl {
    roles_cache: Arc<Mutex<HashMap<i32, Vec<Role>>>>,
    db_pool: DbPool,
    cpu_pool: CpuPool,
}

impl RolesCacheImpl {
    pub fn new(db_pool: DbPool, cpu_pool: CpuPool) -> Self {
        RolesCacheImpl {
            roles_cache: Arc::new(Mutex::new(HashMap::new())),
            db_pool: db_pool,
            cpu_pool: cpu_pool,
        }
    }
}

pub trait RolesCache {
    fn get(&mut self, id: i32) -> Vec<Role>;
}

impl RolesCache for RolesCacheImpl {
    fn get(&mut self, id: i32) -> Vec<Role> {
        let mut mutex = self.roles_cache.lock().unwrap();
        let vec = mutex.entry(id).or_insert_with(|| {
            let db_pool = self.db_pool.clone();
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| Error::Connection(format!("Connection error {}", e)))
                        .and_then(move |conn| {
                            let repo = UserRolesRepoImpl::new(&conn, Box::new(SystemACL::new()));
                            repo.list_for_user(id)
                                .map(|users| users.into_iter().map(|u| u.role).collect())
                        })
                })
                .wait()
                .unwrap_or_default()
        });
        vec.clone()
    }
}
