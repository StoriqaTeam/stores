
use futures::future::*;
use futures_cpupool::CpuPool;
use diesel::Connection;

use models::*;
use repos::{BaseProductsRepo, BaseProductsRepoImpl, ProductAttrsRepo, ProductAttrsRepoImpl, ProductsRepo, ProductsRepoImpl};
use elastic::{ProductsElastic, ProductsElasticImpl};
use super::types::ServiceFuture;
use super::error::ServiceError as Error;
use repos::types::{DbPool, RepoResult};
use repos::acl::{ApplicationAcl, BoxedAcl, RolesCacheImpl, UnauthorizedAcl};

use stq_http::client::ClientHandle;

pub trait BaseProductsService {
    /// Find product by search pattern limited by `count` and `offset` parameters
    fn search(&self, prod: SearchProduct, count: i64, offset: i64) -> ServiceFuture<Vec<BaseProduct>>;
    /// auto complete limited by `count` and `offset` parameters
    fn auto_complete(&self, name: String, count: i64, offset: i64) -> ServiceFuture<Vec<String>>;
    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<BaseProduct>;
    /// Returns product by ID
    fn get_with_variants(&self, product_id: i32) -> ServiceFuture<BaseProductWithVariants>;
    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<BaseProduct>;
    /// Creates base product
    fn create(&self, payload: NewBaseProduct) -> ServiceFuture<BaseProduct>;
    /// Lists base products limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<BaseProduct>>;
    /// Updates base product
    fn update(&self, product_id: i32, payload: UpdateBaseProduct) -> ServiceFuture<BaseProduct>;
}

/// Products services, responsible for Product-related CRUD operations
pub struct BaseProductsServiceImpl {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub roles_cache: RolesCacheImpl,
    pub user_id: Option<i32>,
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

impl BaseProductsServiceImpl {
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

impl BaseProductsService for BaseProductsServiceImpl {
    fn search(&self, search_product: SearchProduct, count: i64, offset: i64) -> ServiceFuture<Vec<BaseProduct>> {
        let products = {
                let client_handle = self.client_handle.clone();
                let address = self.elastic_address.clone();
                let products_el = ProductsElasticImpl::new(client_handle, address);
                products_el
                    .search(search_product, count, offset)
                    .map_err(Error::from)
        };

        Box::new(products.and_then({
            let cpu_pool = self.cpu_pool.clone();
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id;
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
                                    let acl = acl_for_id(roles_cache.clone(), user_id);
                                    let products_repo = BaseProductsRepoImpl::new(&conn, acl);
                                    products_repo.find(el_product.id).map_err(Error::from)
                                })
                                .collect()
                        })
                })
            }
        }))
    }

    fn auto_complete(&self, name: String, count: i64, offset: i64) -> ServiceFuture<Vec<String>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let products_names = {
            let products_el = ProductsElasticImpl::new(client_handle, address);
            products_el
                .auto_complete(name, count, offset)
                .map_err(Error::from)
        };

        Box::new(products_names)
    }

    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<BaseProduct> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let products_repo = BaseProductsRepoImpl::new(&conn, acl);
                    products_repo.find(product_id).map_err(Error::from)
                })
        }))
    }

    /// Returns product by ID
    fn get_with_variants(&self, base_product_id: i32) -> ServiceFuture<BaseProductWithVariants> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), user_id);
                    let base_products_repo = BaseProductsRepoImpl::new(&conn, acl);
                    let acl = acl_for_id(roles_cache.clone(), user_id);
                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    let acl = acl_for_id(roles_cache.clone(), user_id);
                    let attr_prod_repo = ProductAttrsRepoImpl::new(&conn, acl);
                    base_products_repo
                        .find(base_product_id)
                        .map(|base_product| base_product)
                        .and_then(move |base_product| {
                            products_repo
                                .find_with_base_id(base_product.id)
                                .map(|products| (base_product, products))
                                .and_then(move |(base_product, products)| {
                                    products
                                        .into_iter()
                                        .map(|product| {
                                            attr_prod_repo
                                                .find_all_attributes(product.id)
                                                .map(|attrs| {
                                                    attrs
                                                        .into_iter()
                                                        .map(|attr| attr.into())
                                                        .collect::<Vec<AttrValue>>()
                                                })
                                                .map(|attrs| VariantsWithAttributes::new(product, attrs))
                                        })
                                        .collect::<RepoResult<Vec<VariantsWithAttributes>>>()
                                        .and_then(|var| Ok(BaseProductWithVariants::new(base_product, var)))
                                })
                        })
                        .map_err(Error::from)
                })
        }))
    }

    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<BaseProduct> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);

                    let products_repo = BaseProductsRepoImpl::new(&conn, acl);
                    products_repo.deactivate(product_id).map_err(Error::from)
                })
        }))
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<BaseProduct>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let products_repo = BaseProductsRepoImpl::new(&conn, acl);
                    products_repo.list(from, count).map_err(Error::from)
                })
        }))
    }

    /// Creates new product
    fn create(&self, payload: NewBaseProduct) -> ServiceFuture<BaseProduct> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let cpu_pool = self.cpu_pool.clone();
        Box::new(cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), user_id);
                    let products_repo = BaseProductsRepoImpl::new(&conn, acl);
                    conn.transaction::<(BaseProduct), Error, _>(move || products_repo.create(payload).map_err(Error::from))
                })
        }))
    }

    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateBaseProduct) -> ServiceFuture<BaseProduct> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();
        let cpu_pool = self.cpu_pool.clone();

        Box::new(cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), user_id);
                    let products_repo = BaseProductsRepoImpl::new(&conn, acl);
                    conn.transaction::<(BaseProduct), Error, _>(move || {
                        products_repo
                            .update(product_id, payload)
                            .map_err(Error::from)
                    })
                })
        }))
    }
}
