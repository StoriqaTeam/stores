//! CustomAttributes Services, presents CRUD operations with custom_attributes

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use r2d2::ManageConnection;

use stq_types::{BaseProductId, CustomAttributeId};

use super::types::ServiceFuture;
use models::{CustomAttribute, NewCustomAttribute};
use repos::ReposFactory;
use services::Service;

pub trait CustomAttributesService {
    /// Returns custom_attribute by base product id
    fn get_custom_attributes_by_base_product(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<Vec<CustomAttribute>>;
    /// Creates new custom_attribute
    fn create_custom_attribute(&self, payload: NewCustomAttribute) -> ServiceFuture<CustomAttribute>;
    /// Returns all custom attributes
    fn list_custom_attributes(&self) -> ServiceFuture<Vec<CustomAttribute>>;
    /// Returns custom_attribute by ID
    fn get_custom_attribute(&self, id_arg: CustomAttributeId) -> ServiceFuture<Option<CustomAttribute>>;
    /// Deletes custom_attribute
    fn delete_custom_attribute(&self, id_arg: CustomAttributeId) -> ServiceFuture<CustomAttribute>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CustomAttributesService for Service<T, M, F>
{
    /// Returns custom_attribute by base product id
    fn get_custom_attributes_by_base_product(&self, base_product_id_arg: BaseProductId) -> ServiceFuture<Vec<CustomAttribute>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let custom_attributes_repo = repo_factory.create_custom_attributes_repo(&*conn, user_id);
            custom_attributes_repo.find_all_attributes(base_product_id_arg).map_err(|e| {
                e.context("Service CustomAttributes, get_custom_attributes_by_base_product endpoint error occured.")
                    .into()
            })
        })
    }

    /// Returns custom_attribute by ID
    fn get_custom_attribute(&self, id_arg: CustomAttributeId) -> ServiceFuture<Option<CustomAttribute>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let custom_attributes_repo = repo_factory.create_custom_attributes_repo(&*conn, user_id);
            custom_attributes_repo.get_custom_attribute(id_arg).map_err(|e| {
                e.context("Service CustomAttributes, get_custom_attribute endpoint error occured.")
                    .into()
            })
        })
    }

    /// Creates new custom_attribute
    fn create_custom_attribute(&self, new_custom_attribute: NewCustomAttribute) -> ServiceFuture<CustomAttribute> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let custom_attributes_repo = repo_factory.create_custom_attributes_repo(&*conn, user_id);
            conn.transaction::<(CustomAttribute), FailureError, _>(move || custom_attributes_repo.create(new_custom_attribute))
                .map_err(|e| e.context("Service CustomAttributes, create endpoint error occured.").into())
        })
    }

    /// Returns all custom attributes
    fn list_custom_attributes(&self) -> ServiceFuture<Vec<CustomAttribute>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let attributes_repo = repo_factory.create_custom_attributes_repo(&*conn, user_id);
            attributes_repo
                .list()
                .map_err(|e| e.context("Service CustomAttributes, list endpoint error occured.").into())
        })
    }

    /// Deletes custom_attribute
    fn delete_custom_attribute(&self, id_arg: CustomAttributeId) -> ServiceFuture<CustomAttribute> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let custom_attributes_repo = repo_factory.create_custom_attributes_repo(&*conn, user_id);

            custom_attributes_repo
                .delete(id_arg)
                .map_err(|e| e.context("Service CustomAttributes, delete endpoint error occured.").into())
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;
    use tokio_core::reactor::Core;

    use stq_types::{AttributeId, BaseProductId, CustomAttributeId};

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    pub fn create_new_custom_attributes(base_product_id: BaseProductId) -> NewCustomAttribute {
        NewCustomAttribute {
            base_product_id: base_product_id,
            attribute_id: AttributeId(1),
        }
    }

    #[test]
    fn test_get_custom_attributes_by_base_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_custom_attributes_by_base_product(BaseProductId(1));
        let result = core.run(work);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_custom_attribute() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_custom_attribute(CustomAttributeId(1));
        let result = core.run(work);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_custom_attributes() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.list_custom_attributes();
        let result = core.run(work);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_custom_attributes() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_custom_attributes = create_new_custom_attributes(MOCK_BASE_PRODUCT_ID);
        let work = service.create_custom_attribute(new_custom_attributes);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, CustomAttributeId(1));
    }

}
