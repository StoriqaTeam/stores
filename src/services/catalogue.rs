
use futures_cpupool::CpuPool;

use repos::catalogue::{CatalogueRepo, CatalogueRepoImpl};
use super::types::ServiceFuture;
use super::error::Error;
use repos::types::DbPool;


/// Catalogue service, responsible for common static data
pub trait CatalogueService {
    /// Languages endpoint
    fn languages(&self) -> ServiceFuture<Vec<String>>;
    /// Currencies endpoint
    fn currencies(&self) -> ServiceFuture<Vec<String>>;
}

pub struct CatalogueServiceImpl {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
}

impl CatalogueServiceImpl {
    pub fn new(db_pool: DbPool, cpu_pool: CpuPool) -> Self {
        Self { db_pool, cpu_pool }
    }
}


impl CatalogueService for CatalogueServiceImpl {
    /// Healthcheck endpoint, always returns OK status
    fn languages(&self) -> ServiceFuture<Vec<String>> {
        let db_pool = self.db_pool.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let catalogue_repo = CatalogueRepoImpl::new(&conn);
                    catalogue_repo.languages().map_err(|e| Error::from(e))
                })
        }))
    }

    fn currencies(&self) -> ServiceFuture<Vec<String>>{
        let db_pool = self.db_pool.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let catalogue_repo = CatalogueRepoImpl::new(&conn);
                    catalogue_repo.currencies().map_err(|e| Error::from(e))
                })
        }))
    }
    
}



