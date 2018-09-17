use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;

use stq_types::UserId;

use models::authorization::*;
use models::{CatAttr, NewCatAttr, OldCatAttr};
use repos::acl;
use repos::categories::CategoryCacheImpl;
use repos::legacy_acl::{Acl, CheckScope};
use repos::types::RepoResult;
use schema::cat_attr_values::dsl::*;

/// CatAttr repository, responsible for handling cat_attr_values
pub struct CategoryAttrsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, CatAttr>>,
    pub cache: CategoryCacheImpl,
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
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, CatAttr>>, cache: CategoryCacheImpl) -> Self {
        Self { db_conn, acl, cache }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CategoryAttrsRepo
    for CategoryAttrsRepoImpl<'a, T>
{
    /// Find specific category_attributes by category ID
    fn find_all_attributes(&self, category_id_arg: i32) -> RepoResult<Vec<CatAttr>> {
        debug!("Find all attributes for category with id {}.", category_id_arg);
        let query = cat_attr_values.filter(cat_id.eq(category_id_arg)).order(id);
        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|cat_attrs_res: Vec<CatAttr>| {
                acl::check(&*self.acl, Resource::CategoryAttrs, Action::Read, self, None).and_then(|_| Ok(cat_attrs_res.clone()))
            }).map_err(|e: FailureError| e.context("List all category attributes error occured").into())
    }

    /// Creates new category attribute
    fn create(&self, payload: NewCatAttr) -> RepoResult<()> {
        debug!("Create new category attribute {:?}.", payload);
        acl::check(&*self.acl, Resource::CategoryAttrs, Action::Create, self, None)?;
        self.cache.clear();
        let query_category_attribute = diesel::insert_into(cat_attr_values).values(&payload);
        query_category_attribute
            .get_result::<CatAttr>(self.db_conn)
            .map(|_| ())
            .map_err(|e| {
                e.context(format!("Creates new category attribute: {:?} error occured", payload))
                    .into()
            })
    }

    /// Delete category attribute
    fn delete(&self, payload: OldCatAttr) -> RepoResult<()> {
        debug!("Delete category attributewith payload {:?}.", payload);
        acl::check(&*self.acl, Resource::CategoryAttrs, Action::Delete, self, None)?;
        self.cache.clear();
        let filtered = cat_attr_values
            .filter(cat_id.eq(payload.cat_id))
            .filter(attr_id.eq(payload.attr_id));
        let query = diesel::delete(filtered);
        query
            .get_result::<CatAttr>(self.db_conn)
            .map(|_| ())
            .map_err(|e| e.context(format!("Delete category attribute: {:?} error occured", payload)).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, CatAttr>
    for CategoryAttrsRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&CatAttr>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
