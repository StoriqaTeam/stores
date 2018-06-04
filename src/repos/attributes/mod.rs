//! Attributes module responsible for CRUD operations
use std::convert::From;

use diesel;
use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;

use stq_acl::{Acl, CheckScope};

use models::attribute::attributes::dsl::*;
use models::authorization::*;
use models::{Attribute, NewAttribute, UpdateAttribute};
use repos::acl;
use repos::error::RepoError as Error;
use repos::types::RepoResult;

pub mod attributes_cache;

pub use self::attributes_cache::*;

/// Attributes repository, responsible for handling attribute_values
pub struct AttributesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, Error, Attribute>>,
    pub cache: AttributeCacheImpl,
}

pub trait AttributesRepo {
    /// Find specific attribute by id
    fn find(&self, id_arg: i32) -> RepoResult<Attribute>;

    /// List all attributes
    fn list(&self) -> RepoResult<Vec<Attribute>>;

    /// Creates new attribute
    fn create(&self, payload: NewAttribute) -> RepoResult<Attribute>;

    /// Updates specific attribute
    fn update(&self, attribute_id_arg: i32, payload: UpdateAttribute) -> RepoResult<Attribute>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> AttributesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, Error, Attribute>>, cache: AttributeCacheImpl) -> Self {
        Self { db_conn, acl, cache }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> AttributesRepo for AttributesRepoImpl<'a, T> {
    /// Find specific attribute by id
    fn find(&self, id_arg: i32) -> RepoResult<Attribute> {
        debug!("Find in attributes with id {}.", id_arg);
        if self.cache.contains(id_arg) {
            self.cache.get(id_arg)
        } else {
            let query = attributes.filter(id.eq(id_arg));
            query
                .first::<Attribute>(self.db_conn)
                .map_err(Error::from)
                .and_then(|attribute: Attribute| {
                    acl::check(&*self.acl, &Resource::Attributes, &Action::Read, self, Some(&attribute)).and_then(|_| {
                        self.cache.add_attribute(id_arg, attribute.clone());
                        Ok(attribute)
                    })
                })
        }
    }

    /// List all attributes
    fn list(&self) -> RepoResult<Vec<Attribute>> {
        debug!("Find all attributes.");
        let query = attributes.order(id);

        query
            .get_results(self.db_conn)
            .map_err(Error::from)
            .and_then(|attributes_vec: Vec<Attribute>| {
                for attribute in &attributes_vec {
                    acl::check(&*self.acl, &Resource::Attributes, &Action::Read, self, Some(&attribute))?;
                }
                Ok(attributes_vec)
            })
    }

    /// Creates new attribute
    fn create(&self, payload: NewAttribute) -> RepoResult<Attribute> {
        debug!("Create attribute {:?}.", payload);
        let query_attribute = diesel::insert_into(attributes).values(&payload);
        query_attribute
            .get_result::<Attribute>(self.db_conn)
            .map_err(Error::from)
            .and_then(|attribute| {
                acl::check(&*self.acl, &Resource::Attributes, &Action::Create, self, Some(&attribute)).and_then(|_| {
                    self.cache.add_attribute(attribute.id, attribute.clone());
                    Ok(attribute)
                })
            })
    }

    /// Updates specific attribute
    fn update(&self, attribute_id_arg: i32, payload: UpdateAttribute) -> RepoResult<Attribute> {
        debug!("Updating attribute with id {} and payload {:?}.", attribute_id_arg, payload);
        let query = attributes.find(attribute_id_arg);

        query
            .first::<Attribute>(self.db_conn)
            .map_err(Error::from)
            .and_then(|attribute| acl::check(&*self.acl, &Resource::Attributes, &Action::Update, self, Some(&attribute)))
            .and_then(|_| {
                self.cache.remove(attribute_id_arg);
                let filter = attributes.filter(id.eq(attribute_id_arg));
                let query = diesel::update(filter).set(&payload);
                query.get_result::<Attribute>(self.db_conn).map_err(Error::from)
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Attribute>
    for AttributesRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: i32, scope: &Scope, _obj: Option<&Attribute>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
