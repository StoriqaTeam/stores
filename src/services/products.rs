
use futures::future;
use futures::Future;
use futures_cpupool::CpuPool;


use models::product::{NewProduct, UpdateProduct, Product};
use repos::products::{ProductsRepo, ProductsRepoImpl};
use super::types::ServiceFuture;
use super::error::Error;
use repos::types::DbPool;


pub trait ProductsService {
    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<Product>;
    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<Product>;
    /// Creates new product
    fn create(&self, payload: NewProduct) -> ServiceFuture<Product>;
    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Product>>;
    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateProduct) -> ServiceFuture<Product>;
}

/// Products services, responsible for Product-related CRUD operations
pub struct ProductsServiceImpl<U: 'static + ProductsRepo + Clone> {
    pub products_repo: U,
    pub user_email: Option<String>
}

impl ProductsServiceImpl<ProductsRepoImpl> {
    pub fn new(r2d2_pool: DbPool, cpu_pool:CpuPool, user_email: Option<String>) -> Self {
        let products_repo = ProductsRepoImpl::new(r2d2_pool.clone(), cpu_pool.clone());
        Self {
            products_repo: products_repo,
            user_email: user_email
        }
    }
}

impl<U: ProductsRepo + Clone> ProductsService for ProductsServiceImpl<U> {
    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<Product> {
        Box::new(self.products_repo.find(product_id).map_err(Error::from))
    }
    
    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<Product> {
        Box::new(
            self.products_repo
                .deactivate(product_id)
                .map_err(|e| Error::from(e)),
        )
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Product>> {
        Box::new(
            self.products_repo
                .list(from, count)
                .map_err(|e| Error::from(e)),
        )
    }

    /// Creates new product
    fn create(&self, payload: NewProduct) -> ServiceFuture<Product> {
        let products_repo = self.products_repo.clone();
        Box::new(
            products_repo
                .name_exists(payload.name.to_string())
                .map(move |exists| (payload, exists))
                .map_err(Error::from)
                .and_then(|(payload, exists)| match exists {
                    false => future::ok(payload),
                    true => future::err(Error::Validate(
                        validation_errors!({"name": ["name" => "Name already exists"]}),
                    )),
                })
                .and_then(move |new_product| {
                    products_repo
                        .create(new_product)
                        .map_err(|e| Error::from(e))
                })
        )
    }

    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateProduct) -> ServiceFuture<Product> {
        let products_repo = self.products_repo.clone();

        Box::new(
            products_repo
                .find(product_id)
                .and_then(move |_product| products_repo.update(product_id, payload))
                .map_err(|e| Error::from(e)),
        )
    }
}