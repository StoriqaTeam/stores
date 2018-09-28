use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::{ProductId, UserId};

use models::authorization::*;
use models::{BaseProduct, CustomAttributeValue, NewCustomAttributeValue, Product, Store};
use repos::acl;
use repos::legacy_acl::{Acl, CheckScope};
use repos::types::RepoResult;
use schema::base_products::dsl as BaseProducts;
use schema::custom_attributes_values::dsl::*;
use schema::products::dsl as Products;
use schema::stores::dsl as Stores;

/// CustomAttribute repository, responsible for handling custom_attributes_values
pub struct CustomAttributesValuesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, CustomAttributeValue>>,
}

pub trait CustomAttributesValuesRepo {
    /// Find custom attributes values by base_product_id
    fn find_all_attributes(&self, product_id_arg: ProductId) -> RepoResult<Vec<CustomAttributeValue>>;

    /// Creates new custom_attribute values
    fn create(&self, payload: Vec<NewCustomAttributeValue>) -> RepoResult<Vec<CustomAttributeValue>>;

    /// Delete custom attribute values
    fn delete(&self, product_id_arg: ProductId) -> RepoResult<Vec<CustomAttributeValue>>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CustomAttributesValuesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, CustomAttributeValue>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CustomAttributesValuesRepo
    for CustomAttributesValuesRepoImpl<'a, T>
{
    /// Find custom attributes by base_product_id
    fn find_all_attributes(&self, product_id_arg: ProductId) -> RepoResult<Vec<CustomAttributeValue>> {
        debug!("Find all custom attributes values for product with id {}.", product_id_arg);
        let query = custom_attributes_values.filter(product_id.eq(product_id_arg)).order(id);
        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|custom_attributes_values_res: Vec<CustomAttributeValue>| {
                for custom_attribute in &custom_attributes_values_res {
                    acl::check(
                        &*self.acl,
                        Resource::CustomAttributesValues,
                        Action::Read,
                        self,
                        Some(&custom_attribute),
                    )?;
                }
                Ok(custom_attributes_values_res)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "List all custom attributes values error occured for product {}",
                    product_id_arg
                )).into()
            })
    }

    /// Creates new custom attribute values
    fn create(&self, payload: Vec<NewCustomAttributeValue>) -> RepoResult<Vec<CustomAttributeValue>> {
        debug!("Create new custom attribute values {:?}.", payload);
        let query_custom_attribute = diesel::insert_into(custom_attributes_values).values(&payload);
        query_custom_attribute
            .get_results::<CustomAttributeValue>(self.db_conn)
            .map_err(From::from)
            .and_then(|custom_attribute_values: Vec<CustomAttributeValue>| {
                for custom_attribute_value in &custom_attribute_values {
                    acl::check(
                        &*self.acl,
                        Resource::CustomAttributesValues,
                        Action::Create,
                        self,
                        Some(&custom_attribute_value),
                    )?;
                }
                Ok(custom_attribute_values)
            }).map_err(|e: FailureError| {
                e.context(format!("Creates new custom attribute values: {:?} error occured", payload))
                    .into()
            })
    }

    /// Delete all custom attributes values for product
    fn delete(&self, product_id_arg: ProductId) -> RepoResult<Vec<CustomAttributeValue>> {
        debug!("Delete custom attribute values with for product id {:?}.", product_id_arg);
        let filtered = custom_attributes_values.filter(product_id.eq(product_id_arg));
        let query = diesel::delete(filtered);
        query
            .get_results::<CustomAttributeValue>(self.db_conn)
            .map_err(From::from)
            .and_then(|custom_attribute_values: Vec<CustomAttributeValue>| {
                for custom_attribute_value in &custom_attribute_values {
                    acl::check(
                        &*self.acl,
                        Resource::CustomAttributesValues,
                        Action::Delete,
                        self,
                        Some(&custom_attribute_value),
                    )?;
                }
                Ok(custom_attribute_values)
            }).map_err(|e: FailureError| {
                e.context(format!("Delete custom attribute values: {:?} error occured", product_id_arg))
                    .into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, CustomAttributeValue>
    for CustomAttributesValuesRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id: UserId, scope: &Scope, obj: Option<&CustomAttributeValue>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(custom_attribute_value) = obj {
                    Products::products
                        .filter(Products::id.eq(custom_attribute_value.product_id))
                        .inner_join(BaseProducts::base_products.inner_join(Stores::stores))
                        .get_result::<(Product, (BaseProduct, Store))>(self.db_conn)
                        .map(|(_, (_, s))| s.user_id == user_id)
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}
