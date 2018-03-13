//! Products Services, presents CRUD operations with product

use future;
use futures::future::*;
use futures_cpupool::CpuPool;
use diesel::Connection;
use stq_acl::UnauthorizedACL;

use models::*;
use repos::{ProductAttrsRepo, ProductAttrsRepoImpl, ProductsRepo, ProductsRepoImpl};
use elastic::{AttributesSearchRepo, AttributesSearchRepoImpl, ProductsElastic, ProductsElasticImpl};
use super::types::ServiceFuture;
use super::error::ServiceError as Error;
use repos::types::DbPool;
use repos::acl::{ApplicationAcl, BoxedAcl, RolesCacheImpl};

use stq_http::client::ClientHandle;

pub trait ProductsService {
    /// Find product by search pattern limited by `count` and `offset` parameters
    fn search(&self, prod: SearchProduct, count: i64, offset: i64) -> ServiceFuture<Vec<Product>>;
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
    user_id.map_or(Box::new(UnauthorizedACL::default()) as BoxedAcl, |id| {
        (Box::new(ApplicationAcl::new(roles_cache, id)) as BoxedAcl)
    })
}

impl ProductsService for ProductsServiceImpl {
    fn search(&self, search_product: SearchProduct, count: i64, offset: i64) -> ServiceFuture<Vec<Product>> {
        let products = {
            let client_handle = self.client_handle.clone();
            let address = self.elastic_address.clone();
            let attrs = search_product.attr_filters.clone();
            join_all(attrs.into_iter().map(move |attr| {
                let attribute_el = AttributesSearchRepoImpl::new(client_handle.clone(), address.clone());
                let name = attr.name.clone();
                Box::new(
                    attribute_el
                        .find_by_name(SearchAttribute { name: name })
                        .map_err(Error::from)
                        .and_then(|el_attribute| future::ok((el_attribute.id, attr))),
                )
            })).and_then({
                let client_handle = self.client_handle.clone();
                let address = self.elastic_address.clone();
                move |attributes_with_values| {
                    let products_el = ProductsElasticImpl::new(client_handle, address);
                    let search_product_elastic = SearchProductElastic::new(
                        search_product.name,
                        attributes_with_values,
                        search_product.categories_ids,
                    );
                    products_el
                        .search(search_product_elastic, count, offset)
                        .map_err(Error::from)
                }
            })
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
                                    let products_repo = ProductsRepoImpl::new(&conn, acl);
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
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo.find(product_id).map_err(Error::from)
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
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);

                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo.deactivate(product_id).map_err(Error::from)
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
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo.list(from, count).map_err(Error::from)
                })
        }))
    }

    /// Creates new product
    fn create(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let attributes = {
            let attrs = payload.attributes.clone();
            join_all(attrs.into_iter().map(move |attr| {
                let attribute_el = AttributesSearchRepoImpl::new(client_handle.clone(), address.clone());
                let name = attr.name.clone();
                Box::new(
                    attribute_el
                        .find_by_name(SearchAttribute { name: name })
                        .map_err(Error::from)
                        .and_then(|el_attribute| future::ok((el_attribute.id, attr))),
                )
            }))
        };

        Box::new(attributes.and_then({
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id;
            let roles_cache = self.roles_cache.clone();
            let cpu_pool = self.cpu_pool.clone();

            move |attributes_with_values| {
                cpu_pool.spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| Error::Connection(e.into()))
                        .and_then(move |conn| {
                            let acl = acl_for_id(roles_cache.clone(), user_id);
                            let products_repo = ProductsRepoImpl::new(&conn, acl);
                            let acl = acl_for_id(roles_cache.clone(), user_id);
                            let attr_prod_repo = ProductAttrsRepoImpl::new(&conn, acl);
                            let product = payload.product;
                            conn.transaction::<(Product), Error, _>(move || {
                                products_repo
                                    .create(product)
                                    .map_err(Error::from)
                                    .map(move |product| (product, attributes_with_values))
                                    .and_then(move |(product, attributes_with_values)| {
                                        let product_id = product.id;
                                        let res: Result<Vec<ProdAttr>, Error> = attributes_with_values
                                            .into_iter()
                                            .map(|(attr_id, attr_value)| {
                                                let new_attr = NewProdAttr {
                                                    prod_id: product_id,
                                                    attr_id: attr_id,
                                                    value: attr_value.value,
                                                    value_type: attr_value.value_type,
                                                    meta_field: attr_value.meta_field,
                                                };
                                                attr_prod_repo.create(new_attr).map_err(Error::from)
                                            })
                                            .collect();
                                        res.and_then(|_| Ok(product))
                                    })
                            })
                        })
                })
            }
        }))
    }

    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateProductWithAttributes) -> ServiceFuture<Product> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let attributes = {
            let attrs = payload.attributes.clone();
            join_all(attrs.into_iter().map(move |attr| {
                let attribute_el = AttributesSearchRepoImpl::new(client_handle.clone(), address.clone());
                let name = attr.name.clone();
                Box::new(
                    attribute_el
                        .find_by_name(SearchAttribute { name: name })
                        .map_err(Error::from)
                        .and_then(|el_attribute| future::ok((el_attribute.id, attr))),
                )
            }))
        };

        Box::new(attributes.and_then({
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id;
            let roles_cache = self.roles_cache.clone();
            let cpu_pool = self.cpu_pool.clone();

            move |attributes_with_values| {
                cpu_pool.spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| Error::Connection(e.into()))
                        .and_then(move |conn| {
                            let acl = acl_for_id(roles_cache.clone(), user_id);
                            let products_repo = ProductsRepoImpl::new(&conn, acl);
                            let acl = acl_for_id(roles_cache.clone(), user_id);
                            let attr_prod_repo = ProductAttrsRepoImpl::new(&conn, acl);
                            let product = payload.product;
                            conn.transaction::<(Product), Error, _>(move || {
                                products_repo
                                    .update(product_id, product)
                                    .map_err(Error::from)
                                    .map(move |product| (product, attributes_with_values))
                                    .and_then(move |(product, attributes_with_values)| {
                                        let product_id = product.id;
                                        let res: Result<Vec<ProdAttr>, Error> = attributes_with_values
                                            .into_iter()
                                            .map(|(attr_id, attr_value)| {
                                                let update_attr = UpdateProdAttr {
                                                    prod_id: product_id,
                                                    attr_id: attr_id,
                                                    value: attr_value.value,
                                                    value_type: attr_value.value_type,
                                                    meta_field: attr_value.meta_field,
                                                };
                                                attr_prod_repo.update(update_attr).map_err(Error::from)
                                            })
                                            .collect();
                                        res.and_then(|_| Ok(product))
                                    })
                            })
                        })
                })
            }
        }))
    }
}
