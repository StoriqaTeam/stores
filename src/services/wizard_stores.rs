//! WizardStores Services, presents CRUD operations with wizard_stores
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use future;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use super::error::ServiceError;
use super::types::ServiceFuture;
use models::*;
use repos::ReposFactory;

pub trait WizardStoresService {
    /// Returns wizard store by user iD
    fn get(&self) -> ServiceFuture<WizardStore>;
    /// Delete specific wizard store
    fn delete(&self) -> ServiceFuture<WizardStore>;
    /// Creates new wizard store
    fn create(&self) -> ServiceFuture<WizardStore>;
    /// Updates specific wizard store
    fn update(&self, payload: UpdateWizardStore) -> ServiceFuture<WizardStore>;
}

/// WizardStores services, responsible for Store-related CRUD operations
pub struct WizardStoresServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub user_id: Option<i32>,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > WizardStoresServiceImpl<T, M, F>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, user_id: Option<i32>, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            user_id,
            repo_factory,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > WizardStoresService for WizardStoresServiceImpl<T, M, F>
{
    /// Returns wizard store by user iD
    fn get(&self) -> ServiceFuture<WizardStore> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        if let Some(user_id) = user_id {
            Box::new(self.cpu_pool.spawn_fn(move || {
                db_pool
                    .get()
                    .map_err(|e| {
                        error!("Could not get connection to db from pool! {}", e.to_string());
                        ServiceError::Connection(e.into())
                    })
                    .and_then(move |conn| {
                        let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                        wizard_stores_repo.find_by_user_id(user_id).map_err(ServiceError::from)
                    })
            }))
        } else {
            Box::new(future::err(ServiceError::Unauthorized(
                "Could not get stores wizard for unauthorized user".to_string(),
            )))
        }
    }

    /// Delete specific wizard store
    fn delete(&self) -> ServiceFuture<WizardStore> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        if let Some(user_id) = user_id {
            Box::new(self.cpu_pool.spawn_fn(move || {
                db_pool
                    .get()
                    .map_err(|e| {
                        error!("Could not get connection to db from pool! {}", e.to_string());
                        ServiceError::Connection(e.into())
                    })
                    .and_then(move |conn| {
                        let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                        wizard_stores_repo.delete(user_id).map_err(ServiceError::from)
                    })
            }))
        } else {
            Box::new(future::err(ServiceError::Unauthorized(
                "Colud not delete stores wizard for unauthorized user".to_string(),
            )))
        }
    }

    /// Creates new wizard store
    fn create(&self) -> ServiceFuture<WizardStore> {
        let cpu_pool = self.cpu_pool.clone();
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();
        if let Some(user_id) = user_id {
            Box::new({
                cpu_pool.spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| {
                            error!("Could not get connection to db from pool! {}", e.to_string());
                            ServiceError::Connection(e.into())
                        })
                        .and_then(move |conn| {
                            let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                            conn.transaction::<WizardStore, ServiceError, _>(move || {
                                wizard_stores_repo
                                    .wizard_exists(user_id)
                                    .map_err(ServiceError::from)
                                    .and_then(|exists| {
                                        if exists {
                                            wizard_stores_repo.find_by_user_id(user_id).map_err(ServiceError::from)
                                        } else {
                                            wizard_stores_repo.create(user_id).map_err(ServiceError::from)
                                        }
                                    })
                            })
                        })
                })
            })
        } else {
            Box::new(future::err(ServiceError::Unauthorized(
                "Colud not create stores wizard for unauthorized user".to_string(),
            )))
        }
    }

    /// Updates specific wizard store
    fn update(&self, payload: UpdateWizardStore) -> ServiceFuture<WizardStore> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        if let Some(user_id) = user_id {
            Box::new(self.cpu_pool.spawn_fn(move || {
                db_pool
                    .get()
                    .map_err(|e| {
                        error!("Could not get connection to db from pool! {}", e.to_string());
                        ServiceError::Connection(e.into())
                    })
                    .and_then(move |conn| {
                        let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                        wizard_stores_repo.update(user_id, payload).map_err(ServiceError::from)
                    })
            }))
        } else {
            Box::new(future::err(ServiceError::Unauthorized(
                "Colud not update stores wizard for unauthorized user".to_string(),
            )))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use futures_cpupool::CpuPool;
    use r2d2;
    use tokio_core::reactor::Core;
    use tokio_core::reactor::Handle;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    fn create_wizard_store_service(
        user_id: Option<i32>,
        _handle: Arc<Handle>,
    ) -> WizardStoresServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        WizardStoresServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            user_id: user_id,
            repo_factory: MOCK_REPO_FACTORY,
        }
    }

    pub fn create_update_store(name: String) -> UpdateWizardStore {
        UpdateWizardStore {
            name: Some(name),
            ..Default::default()
        }
    }

    #[test]
    fn test_get_store() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_wizard_store_service(Some(MOCK_USER_ID), handle);
        let work = service.get();
        let result = core.run(work).unwrap();
        assert_eq!(result.user_id, MOCK_USER_ID);
    }

    #[test]
    fn test_create_store() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_wizard_store_service(Some(MOCK_USER_ID), handle);
        let work = service.create();
        let result = core.run(work).unwrap();
        assert_eq!(result.user_id, MOCK_USER_ID);
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_wizard_store_service(Some(MOCK_USER_ID), handle);
        let update_store = create_update_store(MOCK_STORE_NAME_JSON.to_string());
        let work = service.update(update_store);
        let result = core.run(work).unwrap();
        assert_eq!(result.user_id, MOCK_USER_ID);
        assert_eq!(result.name, Some(MOCK_STORE_NAME_JSON.to_string()));
    }

    #[test]
    fn test_delete() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_wizard_store_service(Some(MOCK_USER_ID), handle);
        let work = service.delete();
        let result = core.run(work).unwrap();
        assert_eq!(result.user_id, MOCK_USER_ID);
    }

}
