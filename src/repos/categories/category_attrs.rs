use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use errors::Error;
use failure::Error as FailureError;

use failure::Fail;
use std::sync::Arc;
use stq_cache::cache::CacheSingle;
use stq_types::{AttributeId, CategoryId, UserId};

use models::authorization::*;
use models::{CatAttr, Category, NewCatAttr, OldCatAttr};
use repos::acl;
use repos::categories::CategoryCacheImpl;
use repos::legacy_acl::CheckScope;
use repos::types::{RepoAcl, RepoResult};
use schema::cat_attr_values::dsl::*;

/// CatAttr repository, responsible for handling cat_attr_values
pub struct CategoryAttrsRepoImpl<'a, C, T>
where
    C: CacheSingle<Category>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    pub db_conn: &'a T,
    pub acl: Box<RepoAcl<CatAttr>>,
    pub cache: Arc<CategoryCacheImpl<C>>,
}

pub trait CategoryAttrsRepo {
    /// Find category attributes by category ID
    fn find_all_attributes(&self, category_id_arg: CategoryId) -> RepoResult<Vec<CatAttr>>;

    /// Find category attributes by attribute ID
    fn find_all_attributes_by_attribute_id(&self, attribute_id_arg: AttributeId) -> RepoResult<Vec<CatAttr>>;

    /// Creates new category_attribute
    fn create(&self, payload: NewCatAttr) -> RepoResult<()>;

    /// Delete attr from category
    fn delete(&self, payload: OldCatAttr) -> RepoResult<()>;

    /// Deletes specific categories
    fn delete_all_by_category_ids(&self, category_ids_arg: &[CategoryId]) -> RepoResult<()>;
}

impl<'a, C, T> CategoryAttrsRepoImpl<'a, C, T>
where
    C: CacheSingle<Category>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    pub fn new(db_conn: &'a T, acl: Box<RepoAcl<CatAttr>>, cache: Arc<CategoryCacheImpl<C>>) -> Self {
        Self { db_conn, acl, cache }
    }
}

impl<'a, C, T> CategoryAttrsRepo for CategoryAttrsRepoImpl<'a, C, T>
where
    C: CacheSingle<Category>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    /// Find specific category_attributes by category ID
    fn find_all_attributes(&self, category_id_arg: CategoryId) -> RepoResult<Vec<CatAttr>> {
        debug!("Find all attributes for category with id {}.", category_id_arg);
        let query = cat_attr_values.filter(cat_id.eq(category_id_arg)).order(id);
        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|cat_attrs_res: Vec<CatAttr>| {
                acl::check(&*self.acl, Resource::CategoryAttrs, Action::Read, self, None).and_then(|_| Ok(cat_attrs_res.clone()))
            })
            .map_err(|e: FailureError| e.context("List all category attributes error occurred").into())
    }

    /// Find category attributes by attribute ID
    fn find_all_attributes_by_attribute_id(&self, attribute_id_arg: AttributeId) -> RepoResult<Vec<CatAttr>> {
        let query = cat_attr_values.filter(attr_id.eq(attribute_id_arg)).order(id);
        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|cat_attrs_res: Vec<CatAttr>| {
                acl::check(&*self.acl, Resource::CategoryAttrs, Action::Read, self, None).and_then(|_| Ok(cat_attrs_res.clone()))
            })
            .map_err(|e: FailureError| e.context("Find category attributes by attribute ID error occurred").into())
    }

    /// Creates new category attribute
    fn create(&self, payload: NewCatAttr) -> RepoResult<()> {
        debug!("Create new category attribute {:?}.", payload);
        acl::check(&*self.acl, Resource::CategoryAttrs, Action::Create, self, None)?;
        self.cache.remove();
        let query_category_attribute = diesel::insert_into(cat_attr_values).values(&payload);
        query_category_attribute
            .get_result::<CatAttr>(self.db_conn)
            .map(|_| ())
            .map_err(|e| {
                e.context(format!("Creates new category attribute: {:?} error occurred", payload))
                    .into()
            })
    }

    /// Delete category attribute
    fn delete(&self, payload: OldCatAttr) -> RepoResult<()> {
        debug!("Delete category attribute with payload {:?}.", payload);
        acl::check(&*self.acl, Resource::CategoryAttrs, Action::Delete, self, None)?;
        self.cache.remove();
        let filtered = cat_attr_values
            .filter(cat_id.eq(payload.cat_id))
            .filter(attr_id.eq(payload.attr_id));
        let query = diesel::delete(filtered);
        query
            .get_result::<CatAttr>(self.db_conn)
            .map(|_| ())
            .map_err(|e| e.context(format!("Delete category attribute: {:?} error occurred", payload)).into())
    }

    /// Deletes specific categories
    fn delete_all_by_category_ids(&self, category_ids_arg: &[CategoryId]) -> RepoResult<()> {
        debug!("Delete categories attribute({}).", category_ids_arg.len());
        self.cache.remove();

        cat_attr_values
            .filter(cat_id.eq_any(category_ids_arg))
            .load::<CatAttr>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|cat_attrs| {
                cat_attrs
                    .into_iter()
                    .try_for_each(|cat_attr| acl::check(&*self.acl, Resource::CategoryAttrs, Action::Delete, self, Some(&cat_attr)))
            })?;

        diesel::delete(cat_attr_values)
            .filter(cat_id.eq_any(category_ids_arg))
            .execute(self.db_conn)?;

        Ok(())
    }
}

impl<'a, C, T> CheckScope<Scope, CatAttr> for CategoryAttrsRepoImpl<'a, C, T>
where
    C: CacheSingle<Category>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&CatAttr>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
