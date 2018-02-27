use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;

use models::{Attribute, NewAttribute, UpdateAttribute};
use models::attribute::attributes::dsl::*;
use super::error::Error;
use super::types::{DbConnection, RepoResult};
use repos::acl::Acl;
use models::authorization::*;

/// Attributes repository, responsible for handling attribute_values
pub struct AttributesRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: &'a Acl,
}

pub trait AttributesRepo {
    /// Find specific attribute by name
    fn find(&mut self, name: String) -> RepoResult<Attribute>;

    /// Creates new attribute
    fn create(&mut self, payload: NewAttribute) -> RepoResult<Attribute>;

    /// Updates specific attribute
    fn update(&mut self, attribute_id_arg: i32, payload: UpdateAttribute) -> RepoResult<Attribute>;
}

impl<'a> AttributesRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl: &'a Acl) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a> AttributesRepo for AttributesRepoImpl<'a> {
    /// Find specific attribute by name
    fn find(&mut self, name_arg: String) -> RepoResult<Attribute> {
        let query = attributes.filter(name.eq(name_arg));

        query
            .first::<Attribute>(&**self.db_conn)
            .map_err(|e| Error::from(e))
            .and_then(|attribute: Attribute| {
                acl!(
                    [],
                    self.acl,
                    Resource::Products,
                    Action::Read,
                    Some(self.db_conn)
                ).and_then(|_| Ok(attribute))
            })
    }

    /// Creates new attribute
    fn create(&mut self, payload: NewAttribute) -> RepoResult<Attribute> {
        acl!(
            [],
            self.acl,
            Resource::Attributes,
            Action::Create,
            Some(self.db_conn)
        ).and_then(|_| {
            let query_attribute = diesel::insert_into(attributes).values(&payload);
            query_attribute
                .get_result::<Attribute>(&**self.db_conn)
                .map_err(Error::from)
        })
    }

    /// Updates specific attribute
    fn update(&mut self, attribute_id_arg: i32, payload: UpdateAttribute) -> RepoResult<Attribute> {
        let query = attributes.find(attribute_id_arg);

        query
            .first::<Attribute>(&**self.db_conn)
            .map_err(|e| Error::from(e))
            .and_then(|_| {
                acl!(
                    [],
                    self.acl,
                    Resource::Attributes,
                    Action::Update,
                    Some(self.db_conn)
                )
            })
            .and_then(|_| {
                let filter = attributes.filter(id.eq(attribute_id_arg));

                let query = diesel::update(filter).set(&payload);
                query
                    .get_result::<Attribute>(&**self.db_conn)
                    .map_err(|e| Error::from(e))
            })
    }
}
