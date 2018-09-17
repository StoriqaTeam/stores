//! WizardStores Services, presents CRUD operations with wizard_stores
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;
use future;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_types::UserId;

use super::types::ServiceFuture;
use errors::Error;
use models::*;
use repos::ReposFactory;

pub trait WizardStoresService {
    /// Returns wizard store by user iD
    fn get(&self) -> ServiceFuture<Option<WizardStore>>;
    /// Delete specific wizard store
    fn delete(&self) -> ServiceFuture<WizardStore>;
    /// Creates new wizard store
    fn create(&self) -> ServiceFuture<WizardStore>;
    /// Updates specific wizard store
    fn update(&self, payload: UpdateWizardStore) -> ServiceFuture<WizardStore>;
}

/// WizardStores services
pub struct WizardStoresServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub user_id: Option<UserId>,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > WizardStoresServiceImpl<T, M, F>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, user_id: Option<UserId>, repo_factory: F) -> Self {
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
    fn get(&self) -> ServiceFuture<Option<WizardStore>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        if let Some(user_id) = user_id {
            Box::new(self.cpu_pool.spawn_fn(move || {
                db_pool
                    .get()
                    .map_err(|e| e.context(Error::Connection).into())
                    .and_then(move |conn| {
                        let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                        wizard_stores_repo.find_by_user_id(user_id)
                    })
            }))
        } else {
            Box::new(future::err(
                format_err!("Denied request to wizard for unauthorized user")
                    .context(Error::Forbidden)
                    .into(),
            ))
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
                    .map_err(|e| e.context(Error::Connection).into())
                    .and_then(move |conn| {
                        let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                        wizard_stores_repo.delete(user_id)
                    })
            }))
        } else {
            Box::new(future::err(
                format_err!("Denied request to wizard for unauthorized user")
                    .context(Error::Forbidden)
                    .into(),
            ))
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
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                            conn.transaction::<WizardStore, FailureError, _>(move || {
                                wizard_stores_repo.find_by_user_id(user_id).and_then(|wizard| {
                                    if let Some(wizard) = wizard {
                                        Ok(wizard)
                                    } else {
                                        wizard_stores_repo.create(user_id)
                                    }
                                })
                            })
                        })
                })
            })
        } else {
            Box::new(future::err(
                format_err!("Denied request to wizard for unauthorized user")
                    .context(Error::Forbidden)
                    .into(),
            ))
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
                    .map_err(|e| e.context(Error::Connection).into())
                    .and_then(move |conn| {
                        if let Some(slug) = payload.slug.clone() {
                            let stores_repo = repo_factory.create_stores_repo(&*conn, Some(user_id));
                            let slug_exist = if let Some(store_id) = payload.store_id {
                                stores_repo
                                    .find(store_id)
                                    .and_then(|store| {
                                        if let Some(store) = store {
                                            Ok(store)
                                        } else {
                                            Err(format_err!("Not found such store id : {}", store_id)
                                                .context(Error::NotFound)
                                                .into())
                                        }
                                    }).and_then(|s| {
                                        if s.slug == slug {
                                            // if updated slug equal wizard stores store slug
                                            Ok(false)
                                        } else {
                                            // if updated slug equal other stores slug
                                            stores_repo.slug_exists(slug.clone())
                                        }
                                    })
                            } else {
                                stores_repo.slug_exists(slug.clone())
                            };
                            slug_exist.and_then(|exists| {
                                if exists {
                                    Err(format_err!("Store with slug '{}' already exists.", slug)
                                        .context(Error::Validate(
                                            validation_errors!({"slug": ["slug" => "Store with this slug already exists"]}),
                                        )).into())
                                } else {
                                    let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                                    wizard_stores_repo.update(user_id, payload)
                                }
                            })
                        } else {
                            let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                            wizard_stores_repo.update(user_id, payload)
                        }
                    })
            }))
        } else {
            Box::new(future::err(
                format_err!("Denied request to wizard for unauthorized user")
                    .context(Error::Forbidden)
                    .into(),
            ))
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

    use stq_types::*;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    fn create_wizard_store_service(
        user_id: Option<UserId>,
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
        assert_eq!(result.unwrap().user_id, MOCK_USER_ID);
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
