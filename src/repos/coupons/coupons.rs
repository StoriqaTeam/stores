use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::sql_types::Bool;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::{CouponCode, CouponId, StoreId, UserId};

use models::*;
use repos::acl;
use repos::legacy_acl::{Acl, CheckScope};
use repos::types::RepoResult;
use schema::coupons::dsl as Coupons;
use schema::stores::dsl as Stores;

/// Search coupons
#[derive(Clone, Debug)]
pub enum CouponSearch {
    Store(StoreId),
}

/// Coupons repository, responsible for handling coupon
pub struct CouponsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Coupon>>,
}

pub trait CouponsRepo {
    /// Creates new coupon
    fn create(&self, payload: NewCoupon) -> RepoResult<Coupon>;

    /// List all coupons
    fn list(&self) -> RepoResult<Vec<Coupon>>;

    /// Get coupon
    fn get(&self, id_arg: CouponId) -> RepoResult<Option<Coupon>>;

    /// Get coupon by code
    fn get_by_code(&self, code_arg: CouponCode, store_id_arg: StoreId) -> RepoResult<Option<Coupon>>;

    /// Search coupons
    fn find_by(&self, search: CouponSearch) -> RepoResult<Vec<Coupon>>;

    /// Update coupon
    fn update(&self, id_arg: CouponId, payload: UpdateCoupon) -> RepoResult<Coupon>;

    /// Delete coupon
    fn delete(&self, id_arg: CouponId) -> RepoResult<Coupon>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CouponsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Coupon>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CouponsRepo for CouponsRepoImpl<'a, T> {
    /// Creates new coupon
    fn create(&self, payload: NewCoupon) -> RepoResult<Coupon> {
        debug!("Create new coupon {:?}.", payload);
        let mut payload = payload;
        payload.code = payload.code.0.to_uppercase().into();

        let query = diesel::insert_into(Coupons::coupons).values(&payload);
        query
            .get_result::<Coupon>(self.db_conn)
            .map_err(From::from)
            .and_then(|value| {
                acl::check(&*self.acl, Resource::Coupons, Action::Create, self, Some(&value))?;

                Ok(value)
            }).map_err(|e: FailureError| e.context(format!("Creates new coupon: {:?} error occurred", payload)).into())
    }

    /// List all coupons
    fn list(&self) -> RepoResult<Vec<Coupon>> {
        debug!("Find all coupons.");
        let query = Coupons::coupons.order(Coupons::id);

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|values: Vec<Coupon>| {
                for value in &values {
                    acl::check(&*self.acl, Resource::Coupons, Action::Read, self, Some(&value))?;
                }

                Ok(values)
            }).map_err(|e: FailureError| e.context("List all coupons").into())
    }

    /// Get coupon
    fn get(&self, id_arg: CouponId) -> RepoResult<Option<Coupon>> {
        debug!("Find in coupon with id {}.", id_arg);
        let query = Coupons::coupons.filter(Coupons::id.eq(&id_arg));
        query
            .get_result(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|value: Option<Coupon>| {
                if let Some(value) = value.clone() {
                    acl::check(&*self.acl, Resource::Coupons, Action::Read, self, Some(&value))?;
                };

                Ok(value)
            }).map_err(|e: FailureError| e.context(format!("Find coupon by id: {} error occurred", id_arg)).into())
    }

    /// Get coupon by code
    fn get_by_code(&self, code_arg: CouponCode, store_id_arg: StoreId) -> RepoResult<Option<Coupon>> {
        debug!("Find in coupon with by coupon code: {} and store id: {}.", code_arg, store_id_arg);
        let query = Coupons::coupons
            .filter(Coupons::code.eq(&code_arg))
            .filter(Coupons::store_id.eq(store_id_arg));
        query
            .get_result(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|value: Option<Coupon>| {
                if let Some(value) = value.as_ref() {
                    acl::check(&*self.acl, Resource::Coupons, Action::Read, self, Some(value))?;
                };

                Ok(value)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "Find in coupon with by coupon code: {} and store id: {}.",
                    code_arg, store_id_arg
                )).into()
            })
    }

    /// Search coupons
    fn find_by(&self, search: CouponSearch) -> RepoResult<Vec<Coupon>> {
        debug!("Get coupons by search: {:?}.", search);

        let search_exp: Box<BoxableExpression<Coupons::coupons, _, SqlType = Bool>> = match search {
            CouponSearch::Store(value) => Box::new(Coupons::store_id.eq(value)),
        };

        let query = Coupons::coupons.filter(search_exp);

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|values: Vec<Coupon>| {
                for value in &values {
                    acl::check(&*self.acl, Resource::Coupons, Action::Read, self, Some(&value))?;
                }

                Ok(values)
            }).map_err(|e: FailureError| e.context("Search coupons failed.").into())
    }

    /// Update coupon
    fn update(&self, id_arg: CouponId, payload: UpdateCoupon) -> RepoResult<Coupon> {
        debug!("Updating coupon with id {} and payload {:?}.", id_arg, payload);
        let query = Coupons::coupons.find(&id_arg);

        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|value| acl::check(&*self.acl, Resource::Coupons, Action::Update, self, Some(&value)))
            .and_then(|_| {
                let filtered = Coupons::coupons.filter(Coupons::id.eq(&id_arg));
                let query = diesel::update(filtered).set(&payload);

                query.get_result::<Coupon>(self.db_conn).map_err(From::from)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "Updates specific coupon: id: {}, payload: {:?},  error occurred",
                    id_arg, payload
                )).into()
            })
    }

    /// Delete coupon
    fn delete(&self, id_arg: CouponId) -> RepoResult<Coupon> {
        debug!("Delete coupon with id {:?}.", id_arg);
        let filtered = Coupons::coupons.filter(Coupons::id.eq(&id_arg));
        let query = diesel::delete(filtered);

        acl::check(&*self.acl, Resource::Coupons, Action::Delete, self, None)?;

        query
            .get_result::<Coupon>(self.db_conn)
            .map_err(From::from)
            .map_err(|e: FailureError| e.context(format!("Delete coupon: {:?} error occurred", id_arg)).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Coupon>
    for CouponsRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id: UserId, scope: &Scope, obj: Option<&Coupon>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(value) = obj {
                    Coupons::coupons
                        .filter(Coupons::id.eq(&value.id))
                        .inner_join(Stores::stores)
                        .get_result::<(Coupon, Store)>(self.db_conn)
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
