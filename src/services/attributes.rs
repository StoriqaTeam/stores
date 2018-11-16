//! Attributes Services, presents CRUD operations with attributes
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use r2d2::ManageConnection;
use stq_static_resources::language::{Language, Translation};
use stq_types::newtypes::AttributeValueCode;

use models::{Attribute, CreateAttributePayload, NewAttribute, NewAttributeValue, UpdateAttribute};
use repos::{AttributeValuesRepo, ReposFactory};
use services::types::ServiceFuture;
use services::Service;
use stq_types::AttributeId;

pub trait AttributesService {
    /// Returns attribute by ID
    fn get_attribute(&self, attribute_id: AttributeId) -> ServiceFuture<Option<Attribute>>;
    /// Returns all attributes
    fn list_attributes(&self) -> ServiceFuture<Vec<Attribute>>;
    /// Creates new attribute
    fn create_attribute(&self, payload: CreateAttributePayload) -> ServiceFuture<Attribute>;
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
                .map_err(|e| e.context("Service Attributes, get endpoint error occurred.").into())
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
                .map_err(|e| e.context("Service Attributes, list endpoint error occurred.").into())
        })
    }

    /// Creates new attribute
    fn create_attribute(&self, create_attribute_payload: CreateAttributePayload) -> ServiceFuture<Attribute> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let attributes_repo = repo_factory.create_attributes_repo(&*conn, user_id);
            let attribute_values_repo = repo_factory.create_attribute_values_repo(&*conn, user_id);
            conn.transaction::<(Attribute), FailureError, _>(move || {
                let meta_field = if let Some(meta_field) = &create_attribute_payload.meta_field {
                    Some(serde_json::to_value(&meta_field)?)
                } else {
                    None
                };
                let new_attribute = NewAttribute {
                    name: create_attribute_payload.name.clone(),
                    value_type: create_attribute_payload.value_type.clone(),
                    meta_field,
                };
                let created_attribute = attributes_repo.create(new_attribute)?;
                create_attribute_values(&*attribute_values_repo, created_attribute.id, create_attribute_payload)?;
                Ok(created_attribute)
            }).map_err(|e| e.context("Service Attributes, create endpoint error occurred.").into())
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
                .map_err(|e| e.context("Service Attributes, update endpoint error occurred.").into())
        })
    }
}

fn create_attribute_values(
    attribute_values_repo: &AttributeValuesRepo,
    attribute_id: AttributeId,
    create_attribute_payload: CreateAttributePayload,
) -> Result<(), FailureError> {
    let meta = create_attribute_payload
        .meta_field
        .ok_or(format_err!("Can not create attribute values without meta_field"))?;
    match (meta.values, meta.translated_values) {
        (Some(codes), None) => create_attribute_values_from_codes(attribute_values_repo, attribute_id, codes),
        (None, Some(translated_values)) => {
            create_attribute_values_from_translations(attribute_values_repo, attribute_id, translated_values)
        }
        _ => Err(format_err!("Either values or translated_values should be in meta field")),
    }
}

fn create_attribute_values_from_codes(
    attribute_values_repo: &AttributeValuesRepo,
    attr_id: AttributeId,
    codes: Vec<String>,
) -> Result<(), FailureError> {
    for code in codes {
        let new_attribute_value = NewAttributeValue {
            attr_id,
            code: AttributeValueCode(code),
            translations: None,
        };
        let _ = attribute_values_repo.create(new_attribute_value)?;
    }
    Ok(())
}

fn create_attribute_values_from_translations(
    attribute_values_repo: &AttributeValuesRepo,
    attr_id: AttributeId,
    translations: Vec<Vec<Translation>>,
) -> Result<(), FailureError> {
    for translation in translations {
        let (code, value_translations) = extract_code_and_serialize_translations(translation)?;
        let new_attribute_value = NewAttributeValue {
            attr_id,
            code,
            translations: Some(value_translations),
        };
        let _ = attribute_values_repo.create(new_attribute_value)?;
    }
    Ok(())
}

fn extract_code_and_serialize_translations(
    translations: Vec<Translation>,
) -> Result<(AttributeValueCode, serde_json::Value), FailureError> {
    let en_translation = translations
        .iter()
        .find(|t| t.lang == Language::En)
        .ok_or(format_err!("Default {} language is missing in translations", Language::En))?;
    let serialized_translations = serde_json::to_value(&translations)?;
    Ok((AttributeValueCode(en_translation.text.clone()), serialized_translations))
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
