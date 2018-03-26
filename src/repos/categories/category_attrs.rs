use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;

use models::{CatAttr, NewCatAttr, OldCatAttr};
use models::category_attribute::cat_attr_values::dsl::*;
use models::authorization::*;
use repos::error::RepoError as Error;
use repos::types::{DbConnection, RepoResult};
use repos::acl::{self, BoxedAcl};

/// CatAttr repository, responsible for handling cat_attr_values
pub struct CategoryAttrsRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: BoxedAcl,
}

pub trait CategoryAttrsRepo {
    /// Find category attributes by category ID
    fn find_all_attributes(&self, category_id_arg: i32) -> RepoResult<Vec<CatAttr>>;

    /// Creates new category_attribute
    fn create(&self, payload: NewCatAttr) -> RepoResult<()>;

    /// Delete attr from category
    fn delete(&self, payload: OldCatAttr) -> RepoResult<()>;
}

impl<'a> CategoryAttrsRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl: BoxedAcl) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a> CategoryAttrsRepo for CategoryAttrsRepoImpl<'a> {
    /// Find specific category_attributes by category ID
    fn find_all_attributes(&self, category_id_arg: i32) -> RepoResult<Vec<CatAttr>> {
        debug!(
            "Find all attributes for category with id {}.",
            category_id_arg
        );
        let query = cat_attr_values.filter(cat_id.eq(category_id_arg)).order(id);
        query
            .get_results(&**self.db_conn)
            .map_err(Error::from)
            .and_then(|cat_attrs_res: Vec<CatAttr>| {
                acl::check(
                    &*self.acl,
                    &Resource::CategoryAttrs,
                    &Action::Read,
                    &[],
                    Some(self.db_conn),
                ).and_then(|_| Ok(cat_attrs_res.clone()))
            })
    }

    /// Creates new category attribute
    fn create(&self, payload: NewCatAttr) -> RepoResult<()> {
        debug!("Create new category attribute {:?}.", payload);
        acl::check(
            &*self.acl,
            &Resource::CategoryAttrs,
            &Action::Create,
            &[],
            Some(self.db_conn),
        ).and_then(|_| {
            let query_category_attribute = diesel::insert_into(cat_attr_values).values(&payload);
            query_category_attribute
                .get_result::<CatAttr>(&**self.db_conn)
                .map_err(Error::from)
                .map(|_| ())
        })
    }

    /// Delete category attribute
    fn delete(&self, payload: OldCatAttr) -> RepoResult<()> {
        debug!("Delete category attributewith payload {:?}.", payload);
        let filtered = cat_attr_values
            .filter(cat_id.eq(payload.cat_id))
            .filter(attr_id.eq(payload.attr_id));
        let query = diesel::delete(filtered);
        query
            .get_result::<CatAttr>(&**self.db_conn)
            .map_err(Error::from)
            .map(|_| ())
    }
}
