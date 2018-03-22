use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;
use stq_acl::*;

use models::{NewProduct, Product, UpdateProduct};
use models::product::products::dsl::*;
use repos::error::RepoError as Error;
use super::types::{DbConnection, RepoResult};
use models::authorization::*;
use super::acl;
use super::acl::BoxedAcl;

/// Products repository, responsible for handling products
pub struct ProductsRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: BoxedAcl,
}

pub trait ProductsRepo {
    /// Find specific product by ID
    fn find(&self, product_id: i32) -> RepoResult<Product>;

    /// Returns list of products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<Product>>;

    /// Returns list of products with base id
    fn find_with_base_id(&self, base_id: i32) -> RepoResult<Vec<Product>>;

    /// Creates new product
    fn create(&self, payload: NewProduct) -> RepoResult<Product>;

    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateProduct) -> RepoResult<Product>;

    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> RepoResult<Product>;
}

impl<'a> ProductsRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl: BoxedAcl) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<T: Send + 'static, U: LoadQuery<PgConnection, T> + Send + 'static>(&self, query: U) -> RepoResult<T> {
        query.get_result::<T>(&**self.db_conn).map_err(Error::from)
    }
}

impl<'a> ProductsRepo for ProductsRepoImpl<'a> {
    /// Find specific product by ID
    fn find(&self, product_id_arg: i32) -> RepoResult<Product> {
        debug!("Find in products with id {}.", product_id_arg);
        self.execute_query(products.find(product_id_arg))
            .and_then(|product: Product| {
                acl::check(
                    &*self.acl,
                    &Resource::Products,
                    &Action::Read,
                    &[&product],
                    Some(self.db_conn),
                ).and_then(|_| Ok(product))
            })
    }

    /// Creates new product
    fn create(&self, payload: NewProduct) -> RepoResult<Product> {
        debug!("Create products {:?}.", payload);
        acl::check(
            &*self.acl,
            &Resource::Products,
            &Action::Create,
            &[&payload],
            Some(self.db_conn),
        ).and_then(|_| {
            let query_product = diesel::insert_into(products).values(&payload);
            query_product
                .get_result::<Product>(&**self.db_conn)
                .map_err(Error::from)
        })
    }

    /// Returns list of products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<Product>> {
        debug!("Find in products with ids from {} count {}.", from, count);
        let query = products
            .filter(is_active.eq(true))
            .filter(id.ge(from))
            .order(id)
            .limit(count);

        query
            .get_results(&**self.db_conn)
            .map_err(Error::from)
            .and_then(|products_res: Vec<Product>| {
                let resources = products_res
                    .iter()
                    .map(|product| (product as &WithScope<Scope>))
                    .collect::<Vec<&WithScope<Scope>>>();
                acl::check(
                    &*self.acl,
                    &Resource::Products,
                    &Action::Read,
                    &resources,
                    Some(self.db_conn),
                ).and_then(|_| Ok(products_res.clone()))
            })
    }

    /// Returns list of products with base id
    fn find_with_base_id(&self, base_id_arg: i32) -> RepoResult<Vec<Product>> {
        debug!("Find in products with id {}.", base_id_arg);
        let query = products
            .filter(is_active.eq(true))
            .filter(base_product_id.ge(base_id_arg));

        query
            .get_results(&**self.db_conn)
            .map_err(Error::from)
            .and_then(|products_res: Vec<Product>| {
                let resources = products_res
                    .iter()
                    .map(|product| (product as &WithScope<Scope>))
                    .collect::<Vec<&WithScope<Scope>>>();
                acl::check(
                    &*self.acl,
                    &Resource::Products,
                    &Action::Read,
                    &resources,
                    Some(self.db_conn),
                ).and_then(|_| Ok(products_res.clone()))
            })
    }

    /// Updates specific product
    fn update(&self, product_id_arg: i32, payload: UpdateProduct) -> RepoResult<Product> {
        debug!(
            "Updating base product with id {} and payload {:?}.",
            product_id_arg, payload
        );
        self.execute_query(products.find(product_id_arg))
            .and_then(|product: Product| {
                acl::check(
                    &*self.acl,
                    &Resource::Products,
                    &Action::Update,
                    &[&product],
                    Some(self.db_conn),
                )
            })
            .and_then(|_| {
                let filter = products
                    .filter(id.eq(product_id_arg))
                    .filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query
                    .get_result::<Product>(&**self.db_conn)
                    .map_err(Error::from)
            })
    }

    /// Deactivates specific product
    fn deactivate(&self, product_id_arg: i32) -> RepoResult<Product> {
        debug!("Deactivate base product with id {}.", product_id_arg);
        self.execute_query(products.find(product_id_arg))
            .and_then(|product: Product| {
                acl::check(
                    &*self.acl,
                    &Resource::Products,
                    &Action::Delete,
                    &[&product],
                    Some(self.db_conn),
                )
            })
            .and_then(|_| {
                let filter = products
                    .filter(id.eq(product_id_arg))
                    .filter(is_active.eq(true));
                let query = diesel::update(filter).set(is_active.eq(false));
                self.execute_query(query)
            })
    }
}
