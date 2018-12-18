use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::{BaseProductId, CustomAttributeId, UserId};

use models::authorization::*;
use models::{BaseProductRaw, CustomAttribute, NewCustomAttribute, Store};
use repos::acl;
use repos::legacy_acl::CheckScope;
use repos::types::{RepoAcl, RepoResult};
use schema::base_products::dsl as BaseProducts;
use schema::custom_attributes::dsl::*;
use schema::stores::dsl as Stores;

/// CustomAttribute repository, responsible for handling custom_attributes
pub struct CustomAttributesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<RepoAcl<CustomAttribute>>,
}

pub trait CustomAttributesRepo {
    /// Find custom attributes by base_product_id
    fn find_all_attributes(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<CustomAttribute>>;

    /// Creates new custom_attribute
    fn create(&self, payload: NewCustomAttribute) -> RepoResult<CustomAttribute>;

    /// List all custom attributes
    fn list(&self) -> RepoResult<Vec<CustomAttribute>>;

    /// get custom attribute
    fn get_custom_attribute(&self, id_arg: CustomAttributeId) -> RepoResult<Option<CustomAttribute>>;

    /// Delete custom attribute
    fn delete(&self, id_arg: CustomAttributeId) -> RepoResult<CustomAttribute>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CustomAttributesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<RepoAcl<CustomAttribute>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CustomAttributesRepo
    for CustomAttributesRepoImpl<'a, T>
{
    /// Find custom attributes by base_product_id
    fn find_all_attributes(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<CustomAttribute>> {
        debug!("Find all attributes for base product with id {}.", base_product_id_arg);
        let query = custom_attributes.filter(base_product_id.eq(base_product_id_arg)).order(id);
        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|custom_attributes_res: Vec<CustomAttribute>| {
                for custom_attribute in &custom_attributes_res {
                    acl::check(&*self.acl, Resource::CustomAttributes, Action::Read, self, Some(&custom_attribute))?;
                }
                Ok(custom_attributes_res)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "List all custom attributes error occurred for base product {}",
                    base_product_id_arg
                )).into()
            })
    }

    /// Creates new custom attribute
    fn create(&self, payload: NewCustomAttribute) -> RepoResult<CustomAttribute> {
        debug!("Create new custom attribute {:?}.", payload);
        let query_custom_attribute = diesel::insert_into(custom_attributes).values(&payload);
        query_custom_attribute
            .get_result::<CustomAttribute>(self.db_conn)
            .map_err(From::from)
            .and_then(|custom_attribute| {
                acl::check(
                    &*self.acl,
                    Resource::CustomAttributes,
                    Action::Create,
                    self,
                    Some(&custom_attribute),
                )?;
                Ok(custom_attribute)
            }).map_err(|e: FailureError| {
                e.context(format!("Creates new custom attribute: {:?} error occurred", payload))
                    .into()
            })
    }

    // List all custom attributes
    fn list(&self) -> RepoResult<Vec<CustomAttribute>> {
        debug!("Find all attributes.");
        let query = custom_attributes.order(id);

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|attributes_vec: Vec<CustomAttribute>| {
                for attribute in &attributes_vec {
                    acl::check(&*self.acl, Resource::CustomAttributes, Action::Read, self, Some(&attribute))?;
                }
                Ok(attributes_vec)
            }).map_err(|e: FailureError| e.context("List all custom attributes").into())
    }

    /// get custom attribute
    fn get_custom_attribute(&self, id_arg: CustomAttributeId) -> RepoResult<Option<CustomAttribute>> {
        debug!("Find in custom attribute with id {}.", id_arg);
        let query = custom_attributes.filter(id.eq(id_arg));
        query
            .get_result(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|attribute: Option<CustomAttribute>| {
                if let Some(attribute) = attribute.clone() {
                    acl::check(&*self.acl, Resource::CustomAttributes, Action::Read, self, Some(&attribute))?;
                };
                Ok(attribute)
            }).map_err(|e: FailureError| e.context(format!("Find custom attribute by id: {} error occurred", id_arg)).into())
    }

    /// Delete custom attribute
    fn delete(&self, id_arg: CustomAttributeId) -> RepoResult<CustomAttribute> {
        debug!("Delete custom attribute with id {:?}.", id_arg);
        let filtered = custom_attributes.filter(id.eq(id_arg));
        let query = diesel::delete(filtered);
        query
            .get_result::<CustomAttribute>(self.db_conn)
            .map_err(From::from)
            .and_then(|custom_attribute| {
                acl::check(
                    &*self.acl,
                    Resource::CustomAttributes,
                    Action::Delete,
                    self,
                    Some(&custom_attribute),
                )?;
                Ok(custom_attribute)
            }).map_err(|e: FailureError| e.context(format!("Delete custom attribute: {:?} error occurred", id_arg)).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, CustomAttribute>
    for CustomAttributesRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id: UserId, scope: &Scope, obj: Option<&CustomAttribute>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(custom_attribute) = obj {
                    BaseProducts::base_products
                        .filter(BaseProducts::id.eq(custom_attribute.base_product_id))
                        .inner_join(Stores::stores)
                        .get_result::<(BaseProductRaw, Store)>(self.db_conn)
                        .map(|(_, s)| s.user_id == user_id)
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}
