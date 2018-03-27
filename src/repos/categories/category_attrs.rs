use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use stq_acl::{Acl, CheckScope};

use models::{CatAttr, NewCatAttr, OldCatAttr};
use models::category_attribute::cat_attr_values::dsl::*;
use models::authorization::*;
use repos::error::RepoError as Error;
use repos::types::RepoResult;
use repos::acl;

/// CatAttr repository, responsible for handling cat_attr_values
pub struct CategoryAttrsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, Error, CatAttr>>,
}

pub trait CategoryAttrsRepo {
    /// Find category attributes by category ID
    fn find_all_attributes(&self, category_id_arg: i32) -> RepoResult<Vec<CatAttr>>;

    /// Creates new category_attribute
    fn create(&self, payload: NewCatAttr) -> RepoResult<()>;

    /// Delete attr from category
    fn delete(&self, payload: OldCatAttr) -> RepoResult<()>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CategoryAttrsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, Error, CatAttr>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CategoryAttrsRepo
    for CategoryAttrsRepoImpl<'a, T>
{
    /// Find specific category_attributes by category ID
    fn find_all_attributes(&self, category_id_arg: i32) -> RepoResult<Vec<CatAttr>> {
        let query = cat_attr_values.filter(cat_id.eq(category_id_arg)).order(id);

        query
            .get_results(self.db_conn)
            .map_err(Error::from)
            .and_then(|cat_attrs_res: Vec<CatAttr>| {
                acl::check(
                    &*self.acl,
                    &Resource::CategoryAttrs,
                    &Action::Read,
                    self,
                    None,
                ).and_then(|_| Ok(cat_attrs_res.clone()))
            })
    }

    /// Creates new category attribute
    fn create(&self, payload: NewCatAttr) -> RepoResult<()> {
        acl::check(
            &*self.acl,
            &Resource::CategoryAttrs,
            &Action::Create,
            self,
            None,
        ).and_then(|_| {
            let query_category_attribute = diesel::insert_into(cat_attr_values).values(&payload);
            query_category_attribute
                .get_result::<CatAttr>(self.db_conn)
                .map_err(Error::from)
                .map(|_| ())
        })
    }

    /// Delete category attribute
    fn delete(&self, payload: OldCatAttr) -> RepoResult<()> {
        acl::check(
            &*self.acl,
            &Resource::CategoryAttrs,
            &Action::Delete,
            self,
            None,
        ).and_then(|_| {
            let filtered = cat_attr_values
                .filter(cat_id.eq(payload.cat_id))
                .filter(attr_id.eq(payload.attr_id));
            let query = diesel::delete(filtered);
            query
                .get_result::<CatAttr>(self.db_conn)
                .map_err(Error::from)
                .map(|_| ())
        })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, CatAttr>
    for CategoryAttrsRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: i32, scope: &Scope, _obj: Option<&CatAttr>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
