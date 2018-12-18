use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::sql_types::Bool;
use diesel::Connection;
use failure::Error as FailureError;
use repos::types::RepoAcl;

use stq_types::{AttributeId, AttributeValueCode, AttributeValueId, UserId};

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::{AttributeValue, NewAttributeValue, UpdateAttributeValue};
use repos::legacy_acl::*;
use schema::attribute_values::dsl::*;

pub trait AttributeValuesRepo {
    fn create(&self, new_attribute: NewAttributeValue) -> RepoResult<AttributeValue>;
    fn get(&self, attribute_value_id: AttributeValueId) -> RepoResult<Option<AttributeValue>>;
    fn find(&self, attr_id: AttributeId, code: AttributeValueCode) -> RepoResult<Option<AttributeValue>>;
    fn find_many(&self, search_terms: AttributeValuesSearchTerms) -> RepoResult<Vec<AttributeValue>>;
    fn update(&self, id: AttributeValueId, update: UpdateAttributeValue) -> RepoResult<AttributeValue>;
    fn delete(&self, id: AttributeValueId) -> RepoResult<AttributeValue>;
}

/// AttributeValues repository, responsible for handling attribute_values
pub struct AttributeValuesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<RepoAcl<AttributeValue>>,
}

#[derive(Debug, Clone, Default)]
pub struct AttributeValuesSearchTerms {
    pub attr_id: Option<AttributeId>,
    pub ids: Option<Vec<AttributeValueId>>,
    pub code: Option<AttributeValueCode>,
}

impl<'a, T> AttributeValuesRepoImpl<'a, T>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    pub fn new(db_conn: &'a T, acl: Box<RepoAcl<AttributeValue>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T> AttributeValuesRepo for AttributeValuesRepoImpl<'a, T>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    fn create(&self, new_attribute_value: NewAttributeValue) -> RepoResult<AttributeValue> {
        debug!("Create attribute value {:?}.", new_attribute_value);
        diesel::insert_into(attribute_values)
            .values(&new_attribute_value)
            .get_result::<AttributeValue>(self.db_conn)
            .map_err(From::from)
            .and_then(|attr_value| {
                acl::check(&*self.acl, Resource::AttributeValues, Action::Create, self, Some(&attr_value)).and_then(|_| Ok(attr_value))
            }).map_err(|e: FailureError| {
                e.context(format!("Create new attribute_value {:?} error occurred", new_attribute_value))
                    .into()
            })
    }

    fn get(&self, attribute_value_id: AttributeValueId) -> RepoResult<Option<AttributeValue>> {
        let res = attribute_values.find(attribute_value_id).get_result(self.db_conn).optional()?;
        acl::check(&*self.acl, Resource::AttributeValues, Action::Read, self, res.as_ref())?;
        Ok(res)
    }

    fn find(&self, attr_id_arg: AttributeId, code_arg: AttributeValueCode) -> RepoResult<Option<AttributeValue>> {
        let res = attribute_values
            .filter(attr_id.eq(attr_id_arg).and(code.eq(code_arg)))
            .get_result(self.db_conn)
            .optional()?;
        acl::check(&*self.acl, Resource::AttributeValues, Action::Read, self, res.as_ref())?;
        Ok(res)
    }

    fn find_many(&self, search_terms: AttributeValuesSearchTerms) -> RepoResult<Vec<AttributeValue>> {
        type BoxedExpr = Box<BoxableExpression<attribute_values, Pg, SqlType = Bool>>;

        let mut query: BoxedExpr = Box::new(id.eq(id));

        if let Some(attr_id_filter) = search_terms.attr_id {
            query = Box::new(query.and(attr_id.eq(attr_id_filter)));
        }

        if let Some(code_filter) = search_terms.code {
            query = Box::new(query.and(code.eq(code_filter)));
        }

        if let Some(ids_filter) = search_terms.ids {
            query = Box::new(query.and(id.eq_any(ids_filter)));
        }

        attribute_values
            .filter(query)
            .order_by(id)
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|results: Vec<AttributeValue>| {
                for result in results.iter() {
                    acl::check(&*self.acl, Resource::AttributeValues, Action::Read, self, Some(result))?;
                }
                Ok(results)
            }).map_err(|e: FailureError| {
                e.context(format!("Find many attribute values by search terms error occurred"))
                    .into()
            })
    }

    fn update(&self, id_arg: AttributeValueId, update: UpdateAttributeValue) -> RepoResult<AttributeValue> {
        debug!("Changing attribute value {}  - {:?}.", id_arg, update);
        let res = attribute_values.find(id_arg).get_result(self.db_conn)?;
        acl::check(&*self.acl, Resource::AttributeValues, Action::Update, self, Some(&res))?;

        diesel::update(attribute_values.filter(id.eq(id_arg)))
            .set(&update)
            .get_result::<AttributeValue>(self.db_conn)
            .map_err(From::from)
    }

    fn delete(&self, id_arg: AttributeValueId) -> RepoResult<AttributeValue> {
        let res: AttributeValue = attribute_values.find(id_arg).get_result(self.db_conn)?;
        acl::check(&*self.acl, Resource::AttributeValues, Action::Delete, self, Some(&res))?;

        diesel::delete(attribute_values.filter(id.eq(id_arg)))
            .get_result::<AttributeValue>(self.db_conn)
            .map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, AttributeValue>
    for AttributeValuesRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id_arg: UserId, scope: &Scope, _obj: Option<&AttributeValue>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
