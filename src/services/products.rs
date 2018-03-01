//! Products Services, presents CRUD operations with product

use future;
use futures::future::*;
use futures_cpupool::CpuPool;
use diesel::Connection;

use models::product::{NewProductWithAttributes, Product, UpdateProduct};
use models::{NewProdAttr, ProdAttr, SearchProduct};
use repos::{AttributesRepo, AttributesRepoImpl, ProductAttrsRepo, ProductAttrsRepoImpl, ProductsRepo, ProductsRepoImpl,
            ProductsSearchRepo, ProductsSearchRepoImpl};
use super::types::ServiceFuture;
use super::error::ServiceError as Error;
use repos::types::DbPool;
use repos::acl::{Acl, ApplicationAcl, RolesCache, UnauthorizedACL};
use http::client::ClientHandle;

pub trait ProductsService {
    /// Find product by search pattern limited by `count` and `offset` parameters
    fn search(&self, prod: SearchProduct, count: i64, offset: i64) -> ServiceFuture<Vec<Product>>;
    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<Product>;
    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<Product>;
    /// Creates new product
    fn create(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product>;
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
    fn search(&self, search_product: SearchProduct, count: i64, offset: i64) -> ServiceFuture<Vec<Product>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let products = {
            let products_el = ProductsSearchRepoImpl::new(client_handle, address);
            products_el
                .search(search_product, count, offset)
                .map_err(Error::from)
        };

        Box::new(products.and_then({
            let cpu_pool = self.cpu_pool.clone();
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id.clone();
            let roles_cache = self.roles_cache.clone();
            move |el_products| {
                cpu_pool.spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| Error::Connection(e.into()))
                        .and_then(move |conn| {
                            el_products
                                .into_iter()
                                .map(|el_product| {
                                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                                    });
                                    let products_repo = ProductsRepoImpl::new(&conn, &*acl);
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
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                    });
                    let products_repo = ProductsRepoImpl::new(&conn, &*acl);
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
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                    });
                    let products_repo = ProductsRepoImpl::new(&conn, &*acl);
                    products_repo.deactivate(product_id).map_err(Error::from)
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
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                        (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                    });
                    let products_repo = ProductsRepoImpl::new(&conn, &*acl);
                    products_repo.list(from, count).map_err(Error::from)
                })
        }))
    }

    /// Creates new product
    fn create(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| Error::Connection(e.into()))
                        .and_then(move |conn| {
                            let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                                (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                            });
                            let products_repo = ProductsRepoImpl::new(&conn, &*acl);
                            let attr_repo = AttributesRepoImpl::new(&conn, &*acl);
                            let attr_prod_repo = ProductAttrsRepoImpl::new(&conn, &*acl);
                            let product = payload.product;
                            let attrs = payload.attributes;
                            conn.transaction::<(Product, Vec<ProdAttr>), Error, _>(move || {
                                products_repo
                                    .create(product)
                                    .map_err(Error::from)
                                    .map(move |product| (product, attrs))
                                    .and_then(move |(product, attrs)| {
                                        let product_id = product.id;
                                        let res: Result<Vec<ProdAttr>, Error> = attrs
                                            .into_iter()
                                            .map(|attr_value| {
                                                attr_repo
                                                    .find(attr_value.name.clone())
                                                    .map_err(Error::from)
                                                    .map(|atr| (atr.id, attr_value))
                                                    .and_then(|(atr_id, attr_value)| {
                                                        let new_attr = NewProdAttr {
                                                            prod_id: product_id,
                                                            attr_id: atr_id,
                                                            value: attr_value.value,
                                                            value_type: attr_value.value_type,
                                                        };
                                                        attr_prod_repo.create(new_attr).map_err(Error::from)
                                                    })
                                            })
                                            .collect();
                                        res.and_then(|attrs| Ok((product, attrs)))
                                    })
                            })
                        })
                })
                .and_then({
                    let client_handle = self.client_handle.clone();
                    let address = self.elastic_address.clone();
                    move |(product, attrs)| {
                        let products_el = ProductsSearchRepoImpl::new(client_handle, address);
                        products_el
                            .create_product(product.clone().into())
                            .map_err(Error::from)
                            .and_then(move |_| {
                                let res: Vec<_> = attrs
                                    .into_iter()
                                    .map(|attr_value| {
                                        Box::new(
                                            products_el
                                                .create_attribute_product_value(attr_value)
                                                .map_err(Error::from),
                                        )
                                    })
                                    .collect();
                                join_all(res)
                            })
                            .and_then(|_| future::ok(product))
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
                        .map_err(|e| Error::Connection(e.into()))
                        .and_then(move |conn| {
                            let acl = user_id.map_or((Box::new(UnauthorizedACL::new()) as Box<Acl>), |id| {
                                (Box::new(ApplicationAcl::new(roles_cache.clone(), id)) as Box<Acl>)
                            });
                            let products_repo = ProductsRepoImpl::new(&conn, &*acl);
                            products_repo
                                .find(product_id.clone())
                                .and_then(move |_user| products_repo.update(product_id, payload))
                                .map_err(Error::from)
                        })
                })
                .and_then({
                    let client_handle = self.client_handle.clone();
                    let address = self.elastic_address.clone();
                    move |product| {
                        let products_el = ProductsSearchRepoImpl::new(client_handle, address);
                        products_el
                            .update(product.clone().into())
                            .map_err(Error::from)
                            .and_then(|_| future::ok(product))
                    }
                }),
        )
    }
}
