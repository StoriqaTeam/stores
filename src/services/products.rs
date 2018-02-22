//! Products Services, presents CRUD operations with product

use future;
use futures_cpupool::CpuPool;
use diesel::Connection;
use futures::prelude::*;

use models::product::{NewProduct, Product, UpdateProduct};
use models::SearchProduct;
use repos::{ProductsRepo, ProductsRepoImpl, ProductsSearchRepo, ProductsSearchRepoImpl};
use super::types::ServiceFuture;
use super::error::Error;
use repos::types::DbPool;
use repos::acl::{Acl, ApplicationAcl, RolesCache, UnauthorizedACL};
use http::client::ClientHandle;

pub trait ProductsService {
    /// Find stores by name limited by `count` parameters
    fn search(&self, prod: SearchProduct, count: i64, offset: i64) -> ServiceFuture<Vec<Product>>;
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
pub struct ProductsServiceImpl<R: RolesCache + Clone + Send + 'static> {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub roles_cache: R,
    pub user_id: Option<i32>,
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

impl<R: RolesCache + Clone + Send + 'static> ProductsServiceImpl<R> {
    pub fn new(
        db_pool: DbPool,
        cpu_pool: CpuPool,
        roles_cache: R,
        user_id: Option<i32>,
        client_handle: ClientHandle,
        elastic_address: String,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            roles_cache,
            user_id,
            client_handle,
            elastic_address,
        }
    }
}

impl<R: RolesCache + Clone + Send + 'static> ProductsService for ProductsServiceImpl<R> {
    fn search(&self, prod: SearchProduct, count: i64, offset: i64) -> ServiceFuture<Vec<Product>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let fut = {
            let mut products_el = ProductsSearchRepoImpl::new(client_handle, address);
            products_el.search(prod, count, offset).map_err(Error::from)
        };

        let cpu_pool = self.cpu_pool.clone();
        Box::new(cpu_pool.spawn(fut).and_then({
            let cpu_pool = self.cpu_pool.clone();
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id.clone();
            let roles_cache = self.roles_cache.clone();
            move |el_products| {
                cpu_pool.spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| Error::Database(format!("Connection error {}", e)))
                        .and_then(move |conn| {
                            el_products
                                .into_iter()
                                .map(|el_product| {
                                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                                    });
                                    let mut products_repo = ProductsRepoImpl::new(&conn, acl);
                                    products_repo.find(el_product.id).map_err(Error::from)
                                })
                                .collect()
                        })
                })
            }
        }))
    }

    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                    });
                    let mut products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo.find(product_id).map_err(Error::from)
                })
        }))
    }

    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                    });
                    let mut products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo
                        .deactivate(product_id)
                        .map_err(|e| Error::from(e))
                })
        }))
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Product>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                    });
                    let mut products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo.list(from, count).map_err(|e| Error::from(e))
                })
        }))
    }

    /// Creates new product
    fn create(&self, payload: NewProduct) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| Error::Database(format!("Connection error {}", e)))
                        .and_then(move |conn| {
                            let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                                (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                            });
                            let mut products_repo = ProductsRepoImpl::new(&conn, acl);
                            conn.transaction::<Product, Error, _>(move || {
                                products_repo
                                    .name_exists(payload.name.to_string())
                                    .map(move |exists| (payload, exists))
                                    .map_err(Error::from)
                                    .and_then(|(payload, exists)| match exists {
                                        false => Ok(payload),
                                        true => Err(Error::Database("Product already exists".into())),
                                    })
                                    .and_then(move |new_product| {
                                        products_repo
                                            .create(new_product)
                                            .map_err(|e| Error::from(e))
                                    })
                                //rollback if error
                            })
                        })
                })
                .and_then({
                    let cpu_pool = self.cpu_pool.clone();
                    let client_handle = self.client_handle.clone();
                    let address = self.elastic_address.clone();
                    move |product| {
                        let fut = {
                            let mut products_el = ProductsSearchRepoImpl::new(client_handle, address);
                            products_el
                                .create(product.clone().into())
                                .map_err(Error::from)
                                .and_then(|_| future::ok(product))
                        };
                        cpu_pool.spawn(fut)
                    }
                }),
        )
    }

    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateProduct) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| Error::Database(format!("Connection error {}", e)))
                        .and_then(move |conn| {
                            let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                                (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                            });
                            let mut products_repo = ProductsRepoImpl::new(&conn, acl);
                            products_repo
                                .find(product_id.clone())
                                .and_then(move |_user| products_repo.update(product_id, payload))
                                .map_err(|e| Error::from(e))
                        })
                })
                .and_then({
                    let cpu_pool = self.cpu_pool.clone();
                    let client_handle = self.client_handle.clone();
                    let address = self.elastic_address.clone();
                    move |product| {
                        let fut = {
                            let mut products_el = ProductsSearchRepoImpl::new(client_handle, address);
                            products_el
                                .update(product.clone().into())
                                .map_err(Error::from)
                                .and_then(|_| future::ok(product))
                        };
                        cpu_pool.spawn(fut)
                    }
                }),
        )
    }
}
