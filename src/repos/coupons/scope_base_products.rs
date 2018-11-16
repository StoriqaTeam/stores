use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::{BaseProductId, CouponId, UserId};

use models::*;
use repos::acl;
use repos::legacy_acl::CheckScope;
use repos::types::{RepoAcl, RepoResult};
use schema::base_products::dsl as DslBaseProducts;
use schema::coupon_scope_base_products::dsl as DslCouponScope;
use schema::stores::dsl as DslStores;

/// CouponScopeBaseProducts repository, responsible for handling coupon_scope_base_products table
pub struct CouponScopeBaseProductsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<RepoAcl<CouponScopeBaseProducts>>,
}

pub trait CouponScopeBaseProductsRepo {
    /// Add base product in to coupon
    fn create(&self, payload: NewCouponScopeBaseProducts) -> RepoResult<CouponScopeBaseProducts>;

    /// Search base_products by coupon id
    fn find_base_products(&self, id_arg: CouponId) -> RepoResult<Vec<BaseProductId>>;

    /// Delete coupon for scope base products
    fn delete(&self, id_arg: CouponId, base_product_arg: BaseProductId) -> RepoResult<CouponScopeBaseProducts>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CouponScopeBaseProductsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<RepoAcl<CouponScopeBaseProducts>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CouponScopeBaseProductsRepo
    for CouponScopeBaseProductsRepoImpl<'a, T>
{
    /// Add base product in to coupon
    fn create(&self, payload: NewCouponScopeBaseProducts) -> RepoResult<CouponScopeBaseProducts> {
        debug!("Add coupon scope for base product {:?}.", payload);

        let query = diesel::insert_into(DslCouponScope::coupon_scope_base_products).values(&payload);
        query
            .get_result::<CouponScopeBaseProducts>(self.db_conn)
            .map_err(From::from)
            .and_then(|value| {
                acl::check(&*self.acl, Resource::CouponScopeBaseProducts, Action::Create, self, Some(&value))?;

                Ok(value)
            }).map_err(|e: FailureError| {
                e.context(format!("Add coupon scope for base product: {:?} error occurred", payload))
                    .into()
            })
    }

    /// Search base_products by coupon id
    fn find_base_products(&self, id_arg: CouponId) -> RepoResult<Vec<BaseProductId>> {
        debug!("Get base product ids by coupon_id: {}.", id_arg);

        let query = DslCouponScope::coupon_scope_base_products.filter(DslCouponScope::coupon_id.eq(&id_arg));

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|values: Vec<CouponScopeBaseProducts>| {
                let mut results = vec![];

                for value in &values {
                    acl::check(&*self.acl, Resource::CouponScopeBaseProducts, Action::Read, self, Some(&value))?;
                    results.push(value.base_product_id);
                }

                Ok(results)
            }).map_err(|e: FailureError| e.context("Search records coupon scope for base products failed.").into())
    }

    /// Delete coupon for scope base products
    fn delete(&self, id_arg: CouponId, base_product_arg: BaseProductId) -> RepoResult<CouponScopeBaseProducts> {
        debug!("Delete record for coupon_id: {} and base_product_id: {}.", id_arg, base_product_arg);
        let filtered = DslCouponScope::coupon_scope_base_products
            .filter(DslCouponScope::coupon_id.eq(&id_arg))
            .filter(DslCouponScope::base_product_id.eq(&base_product_arg));

        acl::check(&*self.acl, Resource::CouponScopeBaseProducts, Action::Delete, self, None)?;

        let query = diesel::delete(filtered);

        query
            .get_result::<CouponScopeBaseProducts>(self.db_conn)
            .map_err(From::from)
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Delete record coupon scope for base product, coupon_id: {} and base_product_id: {} error occurred",
                    id_arg, base_product_arg
                )).into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, CouponScopeBaseProducts>
    for CouponScopeBaseProductsRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id: UserId, scope: &Scope, obj: Option<&CouponScopeBaseProducts>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(value) = obj {
                    DslBaseProducts::base_products
                        .filter(DslBaseProducts::id.eq(value.base_product_id))
                        .inner_join(DslStores::stores)
                        .get_result::<(BaseProduct, Store)>(self.db_conn)
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
