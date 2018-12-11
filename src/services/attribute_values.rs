//! AttributeValue Services, presents CRUD operations with attribute_values
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use r2d2::ManageConnection;

use errors::Error;
use repos::ReposFactory;
use services::types::ServiceFuture;
use services::Service;
use stq_types::{AttributeId, AttributeValueCode, AttributeValueId};

use models::attributes::attribute_values::AttributeValue;
use models::attributes::attribute_values::NewAttributeValue;
use models::attributes::attribute_values::UpdateAttributeValue;
use repos::{AttributeValuesSearchTerms, ProductAttrsRepo, ProductAttrsSearchTerms};

pub trait AttributeValuesService {
    fn create_attribute_value(&self, new_attribute_value: NewAttributeValue) -> ServiceFuture<AttributeValue>;
    fn get_attribute_value(&self, attr_value_id: AttributeValueId) -> ServiceFuture<Option<AttributeValue>>;
    fn delete_attribute_value(&self, attr_value_id: AttributeValueId) -> ServiceFuture<AttributeValue>;
    fn get_attribute_values(&self, attr_id: AttributeId) -> ServiceFuture<Vec<AttributeValue>>;
    fn update_attribute_value(&self, attr_value_id: AttributeValueId, update: UpdateAttributeValue) -> ServiceFuture<AttributeValue>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewAttributeValuePayload {
    pub code: AttributeValueCode,
    pub translations: Option<serde_json::Value>,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > AttributeValuesService for Service<T, M, F>
{
    fn create_attribute_value(&self, new_attribute_value: NewAttributeValue) -> ServiceFuture<AttributeValue> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let attribute_values_repo = repo_factory.create_attribute_values_repo(&*conn, user_id);
            conn.transaction::<(AttributeValue), FailureError, _>(move || attribute_values_repo.create(new_attribute_value))
                .map_err(|e| e.context("AttributeValuesService, create_attribute_value error occurred.").into())
        })
    }

    fn get_attribute_value(&self, attr_value_id: AttributeValueId) -> ServiceFuture<Option<AttributeValue>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        self.spawn_on_pool(move |conn| {
            let attribute_values_repo = repo_factory.create_attribute_values_repo(&*conn, user_id);
            attribute_values_repo
                .get(attr_value_id)
                .map_err(|e| e.context("AttributeValuesService, get_attribute_value error occurred.").into())
        })
    }

    fn delete_attribute_value(&self, attr_value_id: AttributeValueId) -> ServiceFuture<AttributeValue> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let attribute_values_repo = repo_factory.create_attribute_values_repo(&*conn, user_id);
            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
            conn.transaction::<(AttributeValue), FailureError, _>(move || {
                let attribute_value = attribute_values_repo
                    .get(attr_value_id)?
                    .ok_or(format_err!("Attribute value {} not found", attr_value_id,))?;
                validate_delete_attribute_value(&attribute_value, &*prod_attr_repo)?;

                attribute_values_repo.delete(attribute_value.id)
            })
            .map_err(|e| e.context("AttributeValuesService, delete_attribute_value error occurred.").into())
        })
    }

    fn get_attribute_values(&self, attr_id: AttributeId) -> ServiceFuture<Vec<AttributeValue>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        self.spawn_on_pool(move |conn| {
            let attribute_values_repo = repo_factory.create_attribute_values_repo(&*conn, user_id);
            attribute_values_repo
                .find_many(AttributeValuesSearchTerms {
                    attr_id: Some(attr_id),
                    ..Default::default()
                })
                .map_err(|e| e.context("AttributeValuesService, get_attribute_values error occurred.").into())
        })
    }

    fn update_attribute_value(&self, attr_value_id: AttributeValueId, update: UpdateAttributeValue) -> ServiceFuture<AttributeValue> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let attribute_values_repo = repo_factory.create_attribute_values_repo(&*conn, user_id);
            conn.transaction::<(AttributeValue), FailureError, _>(move || attribute_values_repo.update(attr_value_id, update))
                .map_err(|e| e.context("AttributeValuesService, update_attribute_value error occurred.").into())
        })
    }
}

fn validate_delete_attribute_value(value: &AttributeValue, prod_attr_repo: &ProductAttrsRepo) -> Result<(), FailureError> {
    let prod_attrs = prod_attr_repo.find_many(ProductAttrsSearchTerms {
        attr_value_id: Some(value.id),
        ..Default::default()
    })?;
    if !prod_attrs.is_empty() {
        return Err(
            format_err!("Attribute value {} is used in {} products.", value.code, prod_attrs.len())
                .context(Error::Validate(
                    validation_errors!({"attr_id": ["attr_id" => "Attribute value is used in products."]}),
                ))
                .into(),
        );
    }
    Ok(())
}
