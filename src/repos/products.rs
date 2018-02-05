use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::select;
use diesel::dsl::exists;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;
use futures::future;
use futures_cpupool::CpuPool;

use models::{UpdateProduct, Product, NewProduct};
use models::product::products::dsl::*;
use super::error::Error;
use super::types::{DbConnection, DbPool, RepoFuture};

/// Products repository, responsible for handling products
#[derive(Clone)]
pub struct ProductsRepoImpl {
    // Todo - no need for Arc, since pool is itself an ARC-like structure
    pub r2d2_pool: DbPool,
    pub cpu_pool: CpuPool
}

pub trait ProductsRepo {
    /// Find specific product by ID
    fn find(&self, product_id: i32) -> RepoFuture<Product>;

    /// Verifies product exist
    fn name_exists(&self, name_arg: String) -> RepoFuture<bool>;

    /// Find specific product by full name
    fn find_by_name(&self, name_arg: String) -> RepoFuture<Product>;

    /// Returns list of products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoFuture<Vec<Product>>;

    /// Creates new product
    fn create(&self, payload: NewProduct) -> RepoFuture<Product>;

    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateProduct) -> RepoFuture<Product>;

    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> RepoFuture<Product>;
}

impl ProductsRepoImpl {
    pub fn new(r2d2_pool: DbPool, cpu_pool: CpuPool) -> Self {
        Self {
            r2d2_pool,
            cpu_pool
        }
    }

    fn get_connection(&self) -> DbConnection {
        match self.r2d2_pool.get() {
            Ok(connection) => connection,
            Err(e) => panic!("Error obtaining connection from pool: {}", e),
        }
    }

    fn execute_query<T: Send + 'static, U: LoadQuery<PgConnection, T> + Send + 'static>(
        &self,
        query: U,
    ) -> RepoFuture<T> {
        let conn = match self.r2d2_pool.get() {
            Ok(connection) => connection,
            Err(_) => {
                return Box::new(future::err(
                    Error::Connection("Cannot connect to products db".to_string()),
                ))
            }
        };

        Box::new(self.cpu_pool.spawn_fn(move || {
            query.get_result::<T>(&*conn).map_err(|e| Error::from(e))
        }))
    }
}

impl ProductsRepo for ProductsRepoImpl {
    /// Find specific product by ID
    fn find(&self, product_id_arg: i32) -> RepoFuture<Product> {
        self.execute_query(products.find(product_id_arg))
    }

    /// Verifies product exist
    fn name_exists(&self, name_arg: String) -> RepoFuture<bool> {
        self.execute_query(select(exists(
            products
                .filter(name.eq(name_arg))
        )))
    }

    /// Find specific product by full name
    fn find_by_name(&self, name_arg: String) -> RepoFuture<Product>{
        let conn = self.get_connection();
        let query = products
            .filter(name.eq(name_arg));

        Box::new(self.cpu_pool.spawn_fn(move || {
            query.first::<Product>(&*conn).map_err(|e| Error::from(e))
        }))
    }


    /// Creates new product
    fn create(&self, payload: NewProduct) -> RepoFuture<Product> {
        let conn = self.get_connection();

        Box::new(self.cpu_pool.spawn_fn(move || {
            let query_product = diesel::insert_into(products).values(&payload);
            query_product
                .get_result::<Product>(&*conn)
                .map_err(Error::from)
        }))
    }

     /// Returns list of products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoFuture<Vec<Product>> {
        let conn = self.get_connection();
        let query = products
            .filter(is_active.eq(true))
            .filter(id.gt(from))
            .order(id)
            .limit(count);

        Box::new(self.cpu_pool.spawn_fn(move || {
            query.get_results(&*conn).map_err(|e| Error::from(e))
        }))
    }

    /// Updates specific product
    fn update(&self, product_id_arg: i32, payload: UpdateProduct) -> RepoFuture<Product> {
        let conn = self.get_connection();
        let filter = products.filter(id.eq(product_id_arg)).filter(is_active.eq(true));

        Box::new(self.cpu_pool.spawn_fn(move || {
            let query = diesel::update(filter).set(&payload);
            query.get_result::<Product>(&*conn).map_err(|e| Error::from(e))
        }))
    }

    /// Deactivates specific product
    fn deactivate(&self, product_id_arg: i32) -> RepoFuture<Product> {
        let filter = products.filter(id.eq(product_id_arg)).filter(is_active.eq(true));
        let query = diesel::update(filter).set(is_active.eq(false));
        self.execute_query(query)
    }
}
