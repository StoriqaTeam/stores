//! WizardStores Services, presents CRUD operations with wizard_stores
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use future;
use r2d2::ManageConnection;

use super::types::ServiceFuture;
use errors::Error;
use models::*;
use repos::ReposFactory;
use services::Service;

pub trait WizardStoresService {
    /// Returns wizard store by user iD
    fn get_wizard_store(&self) -> ServiceFuture<Option<WizardStore>>;
    /// Delete specific wizard store
    fn delete_wizard_store(&self) -> ServiceFuture<WizardStore>;
    /// Creates new wizard store
    fn create_wizard_store(&self) -> ServiceFuture<WizardStore>;
    /// Updates specific wizard store
    fn update_wizard_store(&self, payload: UpdateWizardStore) -> ServiceFuture<WizardStore>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > WizardStoresService for Service<T, M, F>
{
    /// Returns wizard store by user iD
    fn get_wizard_store(&self) -> ServiceFuture<Option<WizardStore>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        if let Some(user_id) = user_id {
            self.spawn_on_pool(move |conn| {
                let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                wizard_stores_repo
                    .find_by_user_id(user_id)
                    .map_err(|e| e.context("Service wizard store, get_wizard_store endpoint error occured.").into())
            })
        } else {
            Box::new(future::err(
                format_err!("Denied request to wizard for unauthorized user")
                    .context(Error::Forbidden)
                    .into(),
            ))
        }
    }

    /// Delete specific wizard store
    fn delete_wizard_store(&self) -> ServiceFuture<WizardStore> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        if let Some(user_id) = user_id {
            self.spawn_on_pool(move |conn| {
                let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                wizard_stores_repo.delete(user_id).map_err(|e| {
                    e.context("Service wizard store, delete_wizard_store endpoint error occured.")
                        .into()
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

    /// Creates new wizard store
    fn create_wizard_store(&self) -> ServiceFuture<WizardStore> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        if let Some(user_id) = user_id {
            self.spawn_on_pool(move |conn| {
                let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                conn.transaction::<WizardStore, FailureError, _>(move || {
                    wizard_stores_repo.find_by_user_id(user_id).and_then(|wizard| {
                        if let Some(wizard) = wizard {
                            Ok(wizard)
                        } else {
                            wizard_stores_repo.create(user_id)
                        }
                    })
                }).map_err(|e| {
                    e.context("Service wizard store, create_wizard_store endpoint error occured.")
                        .into()
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
    fn update_wizard_store(&self, payload: UpdateWizardStore) -> ServiceFuture<WizardStore> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        if let Some(user_id) = user_id {
            self.spawn_on_pool(move |conn| {
                if let Some(slug) = payload.slug.clone() {
                    let stores_repo = repo_factory.create_stores_repo(&*conn, Some(user_id));
                    let slug_exist = if let Some(store_id) = payload.store_id {
                        let store = stores_repo.find(store_id)?;
                        let store = store.ok_or(format_err!("Not found such store id : {}", store_id).context(Error::NotFound))?;
                        if store.slug == slug {
                            // if updated slug equal wizard stores store slug
                            Ok(false)
                        } else {
                            // if updated slug equal other stores slug
                            stores_repo.slug_exists(slug.clone())
                        }
                    } else {
                        stores_repo.slug_exists(slug.clone())
                    }?;
                    if slug_exist {
                        return Err(format_err!("Store with slug '{}' already exists.", slug)
                            .context(Error::Validate(
                                validation_errors!({"slug": ["slug" => "Store with this slug already exists"]}),
                            )).into());
                    }
                }
                let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, Some(user_id));
                wizard_stores_repo.update(user_id, payload).map_err(|e| {
                    e.context("Service wizard store, update_wizard_store endpoint error occured.")
                        .into()
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
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use tokio_core::reactor::Core;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

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
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_wizard_store();
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().user_id, MOCK_USER_ID);
    }

    #[test]
    fn test_create_store() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.create_wizard_store();
        let result = core.run(work).unwrap();
        assert_eq!(result.user_id, MOCK_USER_ID);
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let update_store = create_update_store(MOCK_STORE_NAME_JSON.to_string());
        let work = service.update_wizard_store(update_store);
        let result = core.run(work).unwrap();
        assert_eq!(result.user_id, MOCK_USER_ID);
        assert_eq!(result.name, Some(MOCK_STORE_NAME_JSON.to_string()));
    }

    #[test]
    fn test_delete() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.delete_wizard_store();
        let result = core.run(work).unwrap();
        assert_eq!(result.user_id, MOCK_USER_ID);
    }

}
