//! Attributes Services, presents CRUD operations with attributes
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use r2d2::ManageConnection;

use models::{Attribute, NewAttribute, UpdateAttribute};
use repos::ReposFactory;
use services::types::ServiceFuture;
use services::Service;
use stq_types::AttributeId;

pub trait AttributesService {
    /// Returns attribute by ID
    fn get_attribute(&self, attribute_id: AttributeId) -> ServiceFuture<Option<Attribute>>;
    /// Returns all attributes
    fn list_attributes(&self) -> ServiceFuture<Vec<Attribute>>;
    /// Creates new attribute
    fn create_attribute(&self, payload: NewAttribute) -> ServiceFuture<Attribute>;
    /// Updates specific attribute
    fn update_attribute(&self, attribute_id: AttributeId, payload: UpdateAttribute) -> ServiceFuture<Attribute>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > AttributesService for Service<T, M, F>
{
    /// Returns attribute by ID
    fn get_attribute(&self, attribute_id: AttributeId) -> ServiceFuture<Option<Attribute>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let attributes_repo = repo_factory.create_attributes_repo(&*conn, user_id);
            attributes_repo
                .find(attribute_id)
                .map_err(|e| e.context("Service Attributes, get endpoint error occured.").into())
        })
    }

    /// Returns all attributes
    fn list_attributes(&self) -> ServiceFuture<Vec<Attribute>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let attributes_repo = repo_factory.create_attributes_repo(&*conn, user_id);
            attributes_repo
                .list()
                .map_err(|e| e.context("Service Attributes, list endpoint error occured.").into())
        })
    }

    /// Creates new attribute
    fn create_attribute(&self, new_attribute: NewAttribute) -> ServiceFuture<Attribute> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let attributes_repo = repo_factory.create_attributes_repo(&*conn, user_id);
            conn.transaction::<(Attribute), FailureError, _>(move || attributes_repo.create(new_attribute))
                .map_err(|e| e.context("Service Attributes, create endpoint error occured.").into())
        })
    }

    /// Updates specific attribute
    fn update_attribute(&self, attribute_id: AttributeId, payload: UpdateAttribute) -> ServiceFuture<Attribute> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let attributes_repo = repo_factory.create_attributes_repo(&*conn, user_id);
            attributes_repo
                .update(attribute_id, payload)
                .map_err(|e| e.context("Service Attributes, update endpoint error occured.").into())
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use serde_json;
    use tokio_core::reactor::Core;

    use stq_static_resources::*;
    use stq_types::*;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

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
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_attribute(AttributeId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, AttributeId(1));
    }

    #[test]
    fn test_create_attribute() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_attribute = create_new_attribute(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.create_attribute(new_attribute);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, AttributeId(1));
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_attribute = create_update_attribute(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.update_attribute(AttributeId(1), new_attribute);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, AttributeId(1));
    }

}
