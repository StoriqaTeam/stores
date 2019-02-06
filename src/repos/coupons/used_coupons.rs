use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::sql_types::Bool;
use diesel::Connection;
use errors::Error;
use failure::Error as FailureError;

use stq_types::{CouponId, UserId};

use models::*;
use repos::acl;
use repos::legacy_acl::CheckScope;
use repos::types::{RepoAcl, RepoResult};
use schema::used_coupons::dsl as DslUsedCoupons;

/// Search coupons
#[derive(Clone, Debug)]
pub enum UsedCouponSearch {
    Coupon(CouponId),
    User(UserId),
}

/// UsedCoupons repository, responsible for handling used_coupons table
pub struct UsedCouponsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<RepoAcl<UsedCoupon>>,
}

pub trait UsedCouponsRepo {
    /// Creates new used coupon
    fn create(&self, payload: NewUsedCoupon) -> RepoResult<UsedCoupon>;

    /// List all used coupons
    fn list(&self) -> RepoResult<Vec<UsedCoupon>>;

    /// Search used coupons
    fn find_by(&self, search: UsedCouponSearch) -> RepoResult<Vec<UsedCoupon>>;

    /// Check user used coupon
    fn user_used_coupon(&self, id_arg: CouponId, user_id: UserId) -> RepoResult<bool>;

    /// Delete used coupon
    fn delete(&self, id_arg: CouponId, user_id_arg: UserId) -> RepoResult<UsedCoupon>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UsedCouponsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<RepoAcl<UsedCoupon>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UsedCouponsRepo
    for UsedCouponsRepoImpl<'a, T>
{
    /// Creates new used coupon
    fn create(&self, payload: NewUsedCoupon) -> RepoResult<UsedCoupon> {
        debug!("Create new used coupon record {:?}.", payload);

        let query = diesel::insert_into(DslUsedCoupons::used_coupons).values(&payload);
        query
            .get_result::<UsedCoupon>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|value| {
                acl::check(&*self.acl, Resource::UsedCoupons, Action::Create, self, Some(&value))?;

                Ok(value)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Creates new used coupon record: {:?} error occurred", payload))
                    .into()
            })
    }

    /// List all used coupons
    fn list(&self) -> RepoResult<Vec<UsedCoupon>> {
        debug!("Find all used coupons.");
        let query = DslUsedCoupons::used_coupons.order(DslUsedCoupons::coupon_id);

        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|values: Vec<UsedCoupon>| {
                for value in &values {
                    acl::check(&*self.acl, Resource::UsedCoupons, Action::Read, self, Some(&value))?;
                }

                Ok(values)
            })
            .map_err(|e: FailureError| e.context("List all used coupons").into())
    }

    /// Search used coupons
    fn find_by(&self, search: UsedCouponSearch) -> RepoResult<Vec<UsedCoupon>> {
        debug!("Get used coupons by search: {:?}.", search);

        let search_exp: Box<BoxableExpression<DslUsedCoupons::used_coupons, _, SqlType = Bool>> = match search {
            UsedCouponSearch::Coupon(value) => Box::new(DslUsedCoupons::coupon_id.eq(value)),
            UsedCouponSearch::User(value) => Box::new(DslUsedCoupons::user_id.eq(value)),
        };

        let query = DslUsedCoupons::used_coupons.filter(search_exp);

        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|values: Vec<UsedCoupon>| {
                for value in &values {
                    acl::check(&*self.acl, Resource::UsedCoupons, Action::Read, self, Some(&value))?;
                }

                Ok(values)
            })
            .map_err(|e: FailureError| e.context("Search used coupons failed.").into())
    }

    /// Check user used coupon
    fn user_used_coupon(&self, id_arg: CouponId, user_id_arg: UserId) -> RepoResult<bool> {
        debug!("Check coupon_id: {} for user_id: {}.", id_arg, user_id_arg);

        acl::check(&*self.acl, Resource::UsedCoupons, Action::Read, self, None)?;

        let query = DslUsedCoupons::used_coupons
            .filter(DslUsedCoupons::coupon_id.eq(&id_arg))
            .filter(DslUsedCoupons::user_id.eq(&user_id_arg));

        query
            .get_result(self.db_conn)
            .optional()
            .map_err(|e| Error::from(e).into())
            .and_then(|value: Option<UsedCoupon>| match value {
                Some(_) => Ok(true),
                None => Ok(false),
            })
            .map_err(|e: FailureError| {
                e.context(format!("Check coupon_id: {} for user_id: {}.", id_arg, user_id_arg))
                    .into()
            })
    }

    /// Delete used coupon
    fn delete(&self, id_arg: CouponId, user_id_arg: UserId) -> RepoResult<UsedCoupon> {
        debug!("Delete used coupon with coupon_id {} and user_id: {}.", id_arg, user_id_arg);

        acl::check(&*self.acl, Resource::UsedCoupons, Action::Delete, self, None)?;

        let filtered = DslUsedCoupons::used_coupons
            .filter(DslUsedCoupons::coupon_id.eq(&id_arg))
            .filter(DslUsedCoupons::user_id.eq(&user_id_arg));

        let query = diesel::delete(filtered);

        query
            .get_result::<UsedCoupon>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Delete used coupon: by coupon_id: {} and user_id: {} error occurred",
                    id_arg, user_id_arg
                ))
                .into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, UsedCoupon>
    for UsedCouponsRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&UsedCoupon>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
