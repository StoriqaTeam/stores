use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use stq_acl::*;

use models::{BaseProduct, NewProduct, Product, Store, UpdateProduct};
use models::product::products::dsl::*;
use models::store::stores::dsl as Stores;
use models::base_product::base_products::dsl as BaseProducts;

use repos::error::RepoError as Error;
use super::types::RepoResult;
use models::authorization::*;
use super::acl;

/// Products repository, responsible for handling products
pub struct ProductsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, Error, Product>>,
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
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, Error, Product>>) -> Self {
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
                    self,
                    Some(&product),
                ).and_then(|_| Ok(product))
            })
    }

    /// Creates new product
    fn create(&self, payload: NewProduct) -> RepoResult<Product> {
        let query_product = diesel::insert_into(products).values(&payload);
        query_product
            .get_result::<Product>(self.db_conn)
            .map_err(Error::from)
            .and_then(|prod| {
                acl::check(
                    &*self.acl,
                    &Resource::Products,
                    &Action::Create,
                    self,
                    Some(&prod),
                ).and_then(|_| Ok(prod))
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
                for product in products_res.iter() {
                    acl::check(
                        &*self.acl,
                        &Resource::Products,
                        &Action::Read,
                        self,
                        Some(&product),
                    )?;
                }
                Ok(products_res.clone())
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
                for product in products_res.iter() {
                    acl::check(
                        &*self.acl,
                        &Resource::Products,
                        &Action::Read,
                        self,
                        Some(&product),
                    )?;
                }
                Ok(products_res.clone())
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
                    self,
                    Some(&product),
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
                    self,
                    Some(&product),
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

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Product>
    for ProductsRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id: i32, scope: &Scope, obj: Option<&Product>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(product) = obj {
                    BaseProducts::base_products
                        .find(product.base_product_id)
                        .get_result::<BaseProduct>(self.db_conn)
                        .and_then(|base_prod: BaseProduct| {
                            Stores::stores
                                .find(base_prod.store_id)
                                .get_result::<Store>(self.db_conn)
                                .and_then(|store: Store| Ok(store.user_id == user_id))
                        })
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}
