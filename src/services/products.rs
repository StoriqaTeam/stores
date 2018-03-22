//! Products Services, presents CRUD operations with product

use futures_cpupool::CpuPool;
use diesel::Connection;

use models::*;
use repos::{AttributesRepo, AttributesRepoImpl, ProductAttrsRepo, ProductAttrsRepoImpl, ProductsRepo, ProductsRepoImpl};
use super::types::ServiceFuture;
use super::error::ServiceError;
use repos::types::DbPool;
use repos::acl::{ApplicationAcl, BoxedAcl, RolesCacheImpl, UnauthorizedAcl};

use stq_http::client::ClientHandle;

pub trait ProductsService {
    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<Product>;
    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<Product>;
    /// Creates base product
    fn create(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product>;
    /// Lists product variants limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Product>>;
    /// Updates  product
    fn update(&self, product_id: i32, payload: UpdateProductWithAttributes) -> ServiceFuture<Product>;
}

/// Products services, responsible for Product-related CRUD operations
pub struct ProductsServiceImpl {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub roles_cache: RolesCacheImpl,
    pub user_id: Option<i32>,
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

impl ProductsServiceImpl {
    pub fn new(
        db_pool: DbPool,
        cpu_pool: CpuPool,
        roles_cache: RolesCacheImpl,
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

fn acl_for_id(roles_cache: RolesCacheImpl, user_id: Option<i32>) -> BoxedAcl {
    user_id.map_or(Box::new(UnauthorizedAcl::default()) as BoxedAcl, |id| {
        (Box::new(ApplicationAcl::new(roles_cache, id)) as BoxedAcl)
    })
}

impl ProductsService for ProductsServiceImpl {
    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo.find(product_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);

                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo
                        .deactivate(product_id)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Product>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo.list(from, count).map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new product
    fn create(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let cpu_pool = self.cpu_pool.clone();

        Box::new(cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), user_id);
                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    let acl = acl_for_id(roles_cache.clone(), user_id);
                    let prod_attr_repo = ProductAttrsRepoImpl::new(&conn, acl);
                    let acl = acl_for_id(roles_cache.clone(), user_id);
                    let attr_repo = AttributesRepoImpl::new(&conn, acl);
                    let product = payload.product;
                    let attributes = payload.attributes;

                    conn.transaction::<(Product), ServiceError, _>(move || {
                        products_repo
                            .create(product)
                            .map_err(ServiceError::from)
                            .map(move |product| (product, attributes))
                            .and_then(move |(product, attributes)| {
                                let product_id = product.id;
                                let base_product_id = product.base_product_id;
                                let res: Result<Vec<ProdAttr>, ServiceError> = attributes
                                    .into_iter()
                                    .map(|attr_value| {
                                        attr_repo
                                            .find(attr_value.attr_id)
                                            .and_then(|attr| {
                                                let new_prod_attr = NewProdAttr::new(
                                                    product_id,
                                                    base_product_id,
                                                    attr_value.attr_id,
                                                    attr_value.value,
                                                    attr.value_type,
                                                    attr_value.meta_field,
                                                );
                                                prod_attr_repo.create(new_prod_attr)
                                            })
                                            .map_err(ServiceError::from)
                                    })
                                    .collect();
                                res.and_then(|_| Ok(product))
                            })
                    })
                })
        }))
    }

    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateProductWithAttributes) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let cpu_pool = self.cpu_pool.clone();

        Box::new(cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), user_id);
                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    let acl = acl_for_id(roles_cache.clone(), user_id);
                    let prod_attr_repo = ProductAttrsRepoImpl::new(&conn, acl);
                    let product = payload.product;
                    let attributes = payload.attributes;

                    conn.transaction::<(Product), ServiceError, _>(move || {
                        products_repo
                            .update(product_id, product)
                            .map_err(ServiceError::from)
                            .map(move |product| (product, attributes))
                            .and_then(move |(product, attributes)| {
                                let product_id = product.id;
                                let base_product_id = product.base_product_id;
                                let res: Result<Vec<ProdAttr>, ServiceError> = attributes
                                    .into_iter()
                                    .map(|attr_value| {
                                        let update_attr = UpdateProdAttr::new(
                                            product_id,
                                            base_product_id,
                                            attr_value.attr_id,
                                            attr_value.value,
                                            attr_value.meta_field,
                                        );
                                        prod_attr_repo
                                            .update(update_attr)
                                            .map_err(ServiceError::from)
                                    })
                                    .collect();
                                res.and_then(|_| Ok(product))
                            })
                    })
                })
        }))
    }
}
