use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;
use stq_acl::*;

use models::{BaseProduct, NewBaseProduct, UpdateBaseProduct};
use models::base_product::base_products::dsl::*;
use repos::error::RepoError as Error;
use super::types::{DbConnection, RepoResult};
use models::authorization::*;
use super::acl;
use super::acl::BoxedAcl;

/// BaseProducts repository, responsible for handling base_products
pub struct BaseProductsRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: BoxedAcl,
}

pub trait BaseProductsRepo {
    /// Find specific base_product by ID
    fn find(&self, base_product_id: i32) -> RepoResult<BaseProduct>;

    /// Returns list of base_products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<BaseProduct>>;

    /// Creates new base_product
    fn create(&self, payload: NewBaseProduct) -> RepoResult<BaseProduct>;

    /// Updates specific base_product
    fn update(&self, base_product_id: i32, payload: UpdateBaseProduct) -> RepoResult<BaseProduct>;

    /// Deactivates specific base_product
    fn deactivate(&self, base_product_id: i32) -> RepoResult<BaseProduct>;
}

impl<'a> BaseProductsRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl: BoxedAcl) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<T: Send + 'static, U: LoadQuery<PgConnection, T> + Send + 'static>(&self, query: U) -> RepoResult<T> {
        query.get_result::<T>(&**self.db_conn).map_err(Error::from)
    }
}

impl<'a> BaseProductsRepo for BaseProductsRepoImpl<'a> {
    /// Find specific base_product by ID
    fn find(&self, base_product_id_arg: i32) -> RepoResult<BaseProduct> {
        self.execute_query(base_products.find(base_product_id_arg))
            .and_then(|base_product: BaseProduct| {
                acl::check(
                    &*self.acl,
                    &Resource::BaseProducts,
                    &Action::Read,
                    &[&base_product],
                    Some(self.db_conn),
                ).and_then(|_| Ok(base_product))
            })
    }

    /// Creates new base_product
    fn create(&self, payload: NewBaseProduct) -> RepoResult<BaseProduct> {
        acl::check(
            &*self.acl,
            &Resource::BaseProducts,
            &Action::Create,
            &[&payload],
            Some(self.db_conn),
        ).and_then(|_| {
            let query_base_product = diesel::insert_into(base_products).values(&payload);
            query_base_product
                .get_result::<BaseProduct>(&**self.db_conn)
                .map_err(Error::from)
        })
    }

    /// Returns list of base_products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<BaseProduct>> {
        let query = base_products
            .filter(is_active.eq(true))
            .filter(id.ge(from))
            .order(id)
            .limit(count);

        query
            .get_results(&**self.db_conn)
            .map_err(Error::from)
            .and_then(|base_products_res: Vec<BaseProduct>| {
                let resources = base_products_res
                    .iter()
                    .map(|base_product| (base_product as &WithScope<Scope>))
                    .collect::<Vec<&WithScope<Scope>>>();
                acl::check(
                    &*self.acl,
                    &Resource::BaseProducts,
                    &Action::Read,
                    &resources,
                    Some(self.db_conn),
                ).and_then(|_| Ok(base_products_res.clone()))
            })
    }

    /// Updates specific base_product
    fn update(&self, base_product_id_arg: i32, payload: UpdateBaseProduct) -> RepoResult<BaseProduct> {
        self.execute_query(base_products.find(base_product_id_arg))
            .and_then(|base_product: BaseProduct| {
                acl::check(
                    &*self.acl,
                    &Resource::BaseProducts,
                    &Action::Update,
                    &[&base_product],
                    Some(self.db_conn),
                )
            })
            .and_then(|_| {
                let filter = base_products
                    .filter(id.eq(base_product_id_arg))
                    .filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query
                    .get_result::<BaseProduct>(&**self.db_conn)
                    .map_err(Error::from)
            })
    }

    /// Deactivates specific base_product
    fn deactivate(&self, base_product_id_arg: i32) -> RepoResult<BaseProduct> {
        self.execute_query(base_products.find(base_product_id_arg))
            .and_then(|base_product: BaseProduct| {
                acl::check(
                    &*self.acl,
                    &Resource::BaseProducts,
                    &Action::Delete,
                    &[&base_product],
                    Some(self.db_conn),
                )
            })
            .and_then(|_| {
                let filter = base_products
                    .filter(id.eq(base_product_id_arg))
                    .filter(is_active.eq(true));
                let query = diesel::update(filter).set(is_active.eq(false));
                self.execute_query(query)
            })
    }
}
