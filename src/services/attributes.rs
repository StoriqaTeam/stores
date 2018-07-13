//! Attributes Services, presents CRUD operations with attributes
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;
use futures::Future;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_types::UserId;

use errors::Error;
use models::{Attribute, NewAttribute, UpdateAttribute};
use repos::ReposFactory;
use services::types::ServiceFuture;

pub trait AttributesService {
    /// Returns attribute by ID
    fn get(&self, attribute_id: i32) -> ServiceFuture<Option<Attribute>>;
    /// Returns all attributes
    fn list(&self) -> ServiceFuture<Vec<Attribute>>;
    /// Creates new attribute
    fn create(&self, payload: NewAttribute) -> ServiceFuture<Attribute>;
    /// Updates specific attribute
    fn update(&self, attribute_id: i32, payload: UpdateAttribute) -> ServiceFuture<Attribute>;
}

/// Attributes services, responsible for Attribute-related CRUD operations
pub struct AttributesServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    F: ReposFactory<T>,
    M: ManageConnection<Connection = T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub user_id: Option<UserId>,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        F: ReposFactory<T>,
        M: ManageConnection<Connection = T>,
    > AttributesServiceImpl<T, F, M>
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
        F: ReposFactory<T>,
        M: ManageConnection<Connection = T>,
    > AttributesService for AttributesServiceImpl<T, F, M>
{
    /// Returns attribute by ID
    fn get(&self, attribute_id: i32) -> ServiceFuture<Option<Attribute>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let attributes_repo = repo_factory.create_attributes_repo(&*conn, user_id);
                            attributes_repo.find(attribute_id)
                        })
                })
                .map_err(|e| e.context("Service Attributes, get endpoint error occured.").into()),
        )
    }

    /// Returns all attributes
    fn list(&self) -> ServiceFuture<Vec<Attribute>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let attributes_repo = repo_factory.create_attributes_repo(&*conn, user_id);
                            attributes_repo.list()
                        })
                })
                .map_err(|e| e.context("Service Attributes, list endpoint error occured.").into()),
        )
    }

    /// Creates new attribute
    fn create(&self, new_attribute: NewAttribute) -> ServiceFuture<Attribute> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let attributes_repo = repo_factory.create_attributes_repo(&*conn, user_id);
                            conn.transaction::<(Attribute), FailureError, _>(move || attributes_repo.create(new_attribute))
                        })
                })
                .map_err(|e| e.context("Service Attributes, create endpoint error occured.").into()),
        )
    }

    /// Updates specific attribute
    fn update(&self, attribute_id: i32, payload: UpdateAttribute) -> ServiceFuture<Attribute> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let attributes_repo = repo_factory.create_attributes_repo(&*conn, user_id);
                            attributes_repo.update(attribute_id, payload)
                        })
                })
                .map_err(|e| e.context("Service Attributes, update endpoint error occured.").into()),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use futures_cpupool::CpuPool;
    use r2d2;
    use serde_json;
    use tokio_core::reactor::Core;
    use tokio_core::reactor::Handle;

    use stq_static_resources::*;
    use stq_types::*;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    #[allow(unused)]
    fn create_attribute_service(
        user_id: Option<UserId>,
        handle: Arc<Handle>,
    ) -> AttributesServiceImpl<MockConnection, ReposFactoryMock, MockConnectionManager> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        AttributesServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            user_id: user_id,
            repo_factory: MOCK_REPO_FACTORY,
        }
    }

    pub fn create_new_attribute(name: &str) -> NewAttribute {
        NewAttribute {
            name: serde_json::from_str(name).unwrap(),
            value_type: AttributeType::Str,
            meta_field: None,
        }
    }

    pub fn create_update_attribute(name: &str) -> UpdateAttribute {
        UpdateAttribute {
            name: Some(serde_json::from_str(name).unwrap()),
            meta_field: None,
        }
    }

    #[test]
    fn test_get_attribute() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_attribute_service(Some(MOCK_USER_ID), handle);
        let work = service.get(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, 1);
    }

    #[test]
    fn test_create_attribute() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_attribute_service(Some(MOCK_USER_ID), handle);
        let new_attribute = create_new_attribute(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.create(new_attribute);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_attribute_service(Some(MOCK_USER_ID), handle);
        let new_attribute = create_update_attribute(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.update(1, new_attribute);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
    }

}
