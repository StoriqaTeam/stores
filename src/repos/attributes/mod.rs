//! Attributes module responsible for CRUD operations
use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;
use std::sync::Arc;
use stq_cache::cache::Cache;
use stq_types::{AttributeId, UserId};

use models::authorization::*;
use models::{Attribute, NewAttribute, UpdateAttribute};
use repos::acl;
use repos::legacy_acl::CheckScope;
use repos::types::{RepoAcl, RepoResult};
use schema::attributes::dsl::*;

pub mod attributes_cache;

pub use self::attributes_cache::*;

/// Attributes repository, responsible for handling attribute_values
pub struct AttributesRepoImpl<'a, C, T>
where
    C: Cache<Attribute>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    pub db_conn: &'a T,
    pub acl: Box<RepoAcl<Attribute>>,
    pub cache: Arc<AttributeCacheImpl<C>>,
}

pub trait AttributesRepo {
    /// Find specific attribute by id
    fn find(&self, id_arg: AttributeId) -> RepoResult<Option<Attribute>>;

    /// List all attributes
    fn list(&self) -> RepoResult<Vec<Attribute>>;

    /// Creates new attribute
    fn create(&self, payload: NewAttribute) -> RepoResult<Attribute>;

    /// Updates specific attribute
    fn update(&self, attribute_id_arg: AttributeId, payload: UpdateAttribute) -> RepoResult<Attribute>;

    /// Deletes specific attribute
    fn delete(&self, attribute_id_arg: AttributeId) -> RepoResult<()>;
}

impl<'a, C, T> AttributesRepoImpl<'a, C, T>
where
    C: Cache<Attribute>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    pub fn new(db_conn: &'a T, acl: Box<RepoAcl<Attribute>>, cache: Arc<AttributeCacheImpl<C>>) -> Self {
        Self { db_conn, acl, cache }
    }
}

impl<'a, C, T> AttributesRepo for AttributesRepoImpl<'a, C, T>
where
    C: Cache<Attribute>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    /// Find specific attribute by id
    fn find(&self, id_arg: AttributeId) -> RepoResult<Option<Attribute>> {
        debug!("Find in attributes with id {}.", id_arg);
        if let Some(attr) = self.cache.get(id_arg) {
            Ok(Some(attr))
        } else {
            let query = attributes.find(id_arg);
            query
                .get_result(self.db_conn)
                .optional()
                .map_err(From::from)
                .and_then(|attribute: Option<Attribute>| {
                    if let Some(attribute) = attribute.clone() {
                        acl::check(&*self.acl, Resource::Attributes, Action::Read, self, Some(&attribute))?;
                        self.cache.set(id_arg, attribute.clone());
                    };
                    Ok(attribute)
                }).map_err(|e: FailureError| e.context(format!("Find attribute by id: {} error occurred", id_arg)).into())
        }
    }

    /// List all attributes
    fn list(&self) -> RepoResult<Vec<Attribute>> {
        debug!("Find all attributes.");
        let query = attributes.order(id);

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|attributes_vec: Vec<Attribute>| {
                for attribute in &attributes_vec {
                    acl::check(&*self.acl, Resource::Attributes, Action::Read, self, Some(&attribute))?;
                }
                Ok(attributes_vec)
            }).map_err(|e: FailureError| e.context("List all attributes").into())
    }

    /// Creates new attribute
    fn create(&self, payload: NewAttribute) -> RepoResult<Attribute> {
        debug!("Create attribute {:?}.", payload);
        let query_attribute = diesel::insert_into(attributes).values(&payload);
        query_attribute
            .get_result::<Attribute>(self.db_conn)
            .map_err(From::from)
            .and_then(|attribute| {
                acl::check(&*self.acl, Resource::Attributes, Action::Create, self, Some(&attribute)).and_then(|_| {
                    self.cache.set(attribute.id, attribute.clone());
                    Ok(attribute)
                })
            }).map_err(|e: FailureError| e.context(format!("Creates new attribute: {:?} error occurred", payload)).into())
    }

    /// Updates specific attribute
    fn update(&self, attribute_id_arg: AttributeId, payload: UpdateAttribute) -> RepoResult<Attribute> {
        debug!("Updating attribute with id {} and payload {:?}.", attribute_id_arg, payload);
        let query = attributes.find(attribute_id_arg);

        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|attribute| acl::check(&*self.acl, Resource::Attributes, Action::Update, self, Some(&attribute)))
            .and_then(|_| {
                self.cache.remove(attribute_id_arg);
                let filter = attributes.filter(id.eq(attribute_id_arg));
                let query = diesel::update(filter).set(&payload);
                query.get_result::<Attribute>(self.db_conn).map_err(From::from)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "Updates specific attribute: id: {}, payload: {:?},  error occurred",
                    attribute_id_arg, payload
                )).into()
            })
    }

    /// Deletes specific attribute
    fn delete(&self, attribute_id_arg: AttributeId) -> RepoResult<()> {
        debug!("Deleting attribute with id {}", attribute_id_arg);
        let attribute: Option<Attribute> = attributes.find(attribute_id_arg).get_result(self.db_conn).optional()?;
        let attribute = attribute.ok_or(format_err!("Attribute {} not found", attribute_id_arg))?;

        acl::check(&*self.acl, Resource::Attributes, Action::Delete, self, Some(&attribute))?;

        diesel::delete(attributes.filter(id.eq(attribute_id_arg))).get_result::<Attribute>(self.db_conn)?;

        Ok(())
    }
}

impl<'a, C, T> CheckScope<Scope, Attribute> for AttributesRepoImpl<'a, C, T>
where
    C: Cache<Attribute>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&Attribute>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
