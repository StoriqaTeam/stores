use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use stq_acl::*;

use models::{NewProduct, Product, UpdateProduct};
use models::product::products::dsl::*;
use repos::error::RepoError as Error;
use super::types::RepoResult;
use models::authorization::*;
use super::acl;

/// Products repository, responsible for handling products
pub struct ProductsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, Error, T>>,
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

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ProductsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, Error, T>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(Error::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ProductsRepo for ProductsRepoImpl<'a, T> {
    /// Find specific product by ID
    fn find(&self, product_id_arg: i32) -> RepoResult<Product> {
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
        acl::check(
            &*self.acl,
            &Resource::Products,
            &Action::Create,
            &[&payload],
            Some(self.db_conn),
        ).and_then(|_| {
            let query_product = diesel::insert_into(products).values(&payload);
            query_product
                .get_result::<Product>(self.db_conn)
                .map_err(Error::from)
        })
    }

    /// Returns list of products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<Product>> {
        let query = products
            .filter(is_active.eq(true))
            .filter(id.ge(from))
            .order(id)
            .limit(count);

        query
            .get_results(self.db_conn)
            .map_err(Error::from)
            .and_then(|products_res: Vec<Product>| {
                let resources = products_res
                    .iter()
                    .map(|product| (product as &WithScope<Scope, T>))
                    .collect::<Vec<&WithScope<Scope, T>>>();
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
        let query = products
            .filter(is_active.eq(true))
            .filter(base_product_id.ge(base_id_arg));

        query
            .get_results(self.db_conn)
            .map_err(Error::from)
            .and_then(|products_res: Vec<Product>| {
                let resources = products_res
                    .iter()
                    .map(|product| (product as &WithScope<Scope, T>))
                    .collect::<Vec<&WithScope<Scope, T>>>();
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
                    .get_result::<Product>(self.db_conn)
                    .map_err(Error::from)
            })
    }

    /// Deactivates specific product
    fn deactivate(&self, product_id_arg: i32) -> RepoResult<Product> {
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
