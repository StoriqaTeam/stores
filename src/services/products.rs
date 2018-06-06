//! Products Services, presents CRUD operations with product
use std::collections::HashMap;

use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use failure::Error as FailureError;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use errors::ControllerError;
use stq_http::client::ClientHandle;

use super::types::ServiceFuture;
use models::*;
use repos::ReposFactory;

pub trait ProductsService {
    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<Option<Product>>;
    /// Returns store_id by ID
    fn get_store_id(&self, product_id: i32) -> ServiceFuture<Option<i32>>;
    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<Product>;
    /// Creates base product
    fn create(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product>;
    /// Lists product variants limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> ServiceFuture<Vec<Product>>;
    /// Updates  product
    fn update(&self, product_id: i32, payload: UpdateProductWithAttributes) -> ServiceFuture<Product>;
    /// Get by base product id
    fn find_with_base_id(&self, base_product_id: i32) -> ServiceFuture<Vec<Product>>;
    /// Get by base product id
    fn find_attributes(&self, product_id: i32) -> ServiceFuture<Vec<AttrValue>>;
}

/// Products services, responsible for Product-related CRUD operations
pub struct ProductsServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub user_id: Option<i32>,
    pub client_handle: ClientHandle,
    pub elastic_address: String,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ProductsServiceImpl<T, M, F>
{
    pub fn new(
        db_pool: Pool<M>,
        cpu_pool: CpuPool,
        user_id: Option<i32>,
        client_handle: ClientHandle,
        elastic_address: String,
        repo_factory: F,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            user_id,
            client_handle,
            elastic_address,
            repo_factory,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ProductsService for ProductsServiceImpl<T, M, F>
{
    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<Option<Product>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| ControllerError::Connection(e.into()).into())
                        .and_then(move |conn| {
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            products_repo.find(product_id)
                        })
                })
                .map_err(|e| e.context("Service Product, get endpoint error occured.").into()),
        )
    }

    /// Returns store_id by ID
    fn get_store_id(&self, product_id: i32) -> ServiceFuture<Option<i32>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| ControllerError::Connection(e.into()).into())
                        .and_then(move |conn| {
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                            products_repo.find(product_id).and_then(move |product| {
                                if let Some(product) = product {
                                    base_products_repo.find(product.base_product_id).map(|base_product| {
                                        if let Some(base_product) = base_product {
                                            Some(base_product.store_id)
                                        } else {
                                            None
                                        }
                                    })
                                } else {
                                    Ok(None)
                                }
                            })
                        })
                })
                .map_err(|e| e.context("Service Product, get_store_id endpoint error occured.").into()),
        )
    }

    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| ControllerError::Connection(e.into()).into())
                        .and_then(move |conn| {
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            products_repo.deactivate(product_id)
                        })
                })
                .map_err(|e| e.context("Service Product, deactivate endpoint error occured.").into()),
        )
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> ServiceFuture<Vec<Product>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| ControllerError::Connection(e.into()).into())
                        .and_then(move |conn| {
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            products_repo.list(from, count)
                        })
                })
                .map_err(|e| e.context("Service Product, list endpoint error occured.").into()),
        )
    }

    /// Creates new product
    fn create(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let cpu_pool = self.cpu_pool.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(
            cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| ControllerError::Connection(e.into()).into())
                        .and_then(move |conn| {
                            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
                            let attr_repo = repo_factory.create_attributes_repo(&*conn, user_id);
                            let mut product = payload.product;
                            let attributes = payload.attributes;

                            conn.transaction::<(Product), FailureError, _>(move || {
                                // fill currency id taken from base_product first
                                base_products_repo
                                    .find(product.base_product_id)
                                    .map(move |base_product| {
                                        product.currency_id = base_product.map(|base_product| base_product.currency_id);
                                        product
                                    })
                                    .and_then(|product| products_repo.create(product))
                                    .map(move |product| (product, attributes))
                                    .and_then(move |(product, attributes)| {
                                        let product_id = product.id;
                                        let base_product_id = product.base_product_id;
                                        // searching for existed product with such attribute values
                                        prod_attr_repo
                                            .find_all_attributes_by_base(base_product_id)
                                            .and_then(|base_attrs| {
                                                let mut hash = HashMap::<i32, HashMap<i32, String>>::default();
                                                for attr in base_attrs.into_iter() {
                                                    let mut prod_attrs =
                                                        hash.entry(attr.prod_id).or_insert_with(HashMap::<i32, String>::default);
                                                    prod_attrs.insert(attr.attr_id, attr.value);
                                                }
                                                let exists = hash.into_iter().any(|(_, prod_attrs)| {
                                                    attributes.iter().all(|attr| {
                                                        if let Some(value) = prod_attrs.get(&attr.attr_id) {
                                                            value == &attr.value
                                                        } else {
                                                            false
                                                        }
                                                    })
                                                });
                                                if exists {
                                                    Err(ControllerError::Validate(
                                                        validation_errors!({"attributes": ["attributes" => "Product with this attributes already exists"]}),
                                                    ).into())
                                                } else {
                                                    Ok(())
                                                }
                                            })
                                            .and_then(|_| -> Result<Vec<ProdAttr>, FailureError> {
                                                attributes
                                                    .into_iter()
                                                    .map(|attr_value| {
                                                        attr_repo.find(attr_value.attr_id).and_then(|attr| {
                                                            if let Some(attr) = attr {
                                                                let new_prod_attr = NewProdAttr::new(
                                                                    product_id,
                                                                    base_product_id,
                                                                    attr_value.attr_id,
                                                                    attr_value.value,
                                                                    attr.value_type,
                                                                    attr_value.meta_field,
                                                                );
                                                                prod_attr_repo.create(new_prod_attr)
                                                            } else {
                                                                error!("Not found such attribute id : {}", attr_value.attr_id);
                                                                Err(ControllerError::NotFound.into())
                                                            }
                                                        })
                                                    })
                                                    .collect()
                                            })
                                            .and_then(|_| Ok(product))
                                    })
                            })
                        })
                })
                .map_err(|e| e.context("Service Product, create endpoint error occured.").into()),
        )
    }

    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateProductWithAttributes) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let cpu_pool = self.cpu_pool.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(
            cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| ControllerError::Connection(e.into()).into())
                        .and_then(move |conn| {
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
                            let attr_repo = repo_factory.create_attributes_repo(&*conn, user_id);
                            let product = payload.product;
                            let attributes = payload.attributes;

                            conn.transaction::<(Product), FailureError, _>(move || {
                                let prod = if let Some(product) = product {
                                    products_repo.update(product_id, product)
                                } else {
                                    products_repo.find(product_id).and_then(|product| {
                                        if let Some(product) = product {
                                            Ok(product)
                                        } else {
                                            error!("Not found such product id : {}", product_id);
                                            Err(ControllerError::NotFound.into())
                                        }
                                    })
                                };
                                prod.map(move |product| (product, attributes))
                                    .and_then(move |(product, attributes)| {
                                        if let Some(attributes) = attributes {
                                            let product_id = product.id;
                                            let base_product_id = product.base_product_id;
                                            // deleting old attributes for this product
                                            prod_attr_repo.delete_all_attributes(product_id)
                                    // searching for existed product with such attribute values
                                    .and_then(|_|
                                        prod_attr_repo
                                            .find_all_attributes_by_base(base_product_id)
                                            )
                                        .and_then(|base_attrs| {
                                            let mut hash = HashMap::<i32, HashMap<i32, String>>::default();
                                            for attr in base_attrs.into_iter() {
                                                let mut prod_attrs =
                                                    hash.entry(attr.prod_id).or_insert_with(HashMap::<i32, String>::default);
                                                prod_attrs.insert(attr.attr_id, attr.value);
                                            }
                                            let exists = hash.into_iter().any(|(_, prod_attrs)| {
                                                attributes.iter().all(|attr| {
                                                    if let Some(value) = prod_attrs.get(&attr.attr_id) {
                                                        value == &attr.value
                                                    } else {
                                                        false
                                                    }
                                                })
                                            });
                                            if exists {
                                                Err(ControllerError::Validate(
                                                    validation_errors!({"attributes": ["attributes" => "Product with this attributes already exists"]}),
                                                ).into())
                                            } else {
                                                Ok(())
                                            }
                                        })
                                        .and_then(|_| -> Result<Vec<ProdAttr>, FailureError> {
                                            attributes
                                                .into_iter()
                                                .map(|attr_value| {
                                                    attr_repo
                                                        .find(attr_value.attr_id)
                                                        .and_then(|attr| {
                                                            if let Some(attr) = attr {
                                                                let new_prod_attr = NewProdAttr::new(
                                                                    product_id,
                                                                    base_product_id,
                                                                    attr_value.attr_id,
                                                                    attr_value.value,
                                                                    attr.value_type,
                                                                    attr_value.meta_field,
                                                                );
                                                                prod_attr_repo.create(new_prod_attr)
                                                            } else {
                                                                error!("Not found such attribute id : {}", attr_value.attr_id);
                                                                Err(ControllerError::NotFound.into())
                                                            }
                                                        })
                                                })
                                                .collect()
                                        })
                                        .and_then(|_| Ok(product))
                                        } else {
                                            Ok(product)
                                        }
                                    })
                            })
                        })
                })
                .map_err(|e| e.context("Service Product, update endpoint error occured.").into()),
        )
    }

    /// Get by base product id
    fn find_with_base_id(&self, base_product_id: i32) -> ServiceFuture<Vec<Product>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| ControllerError::Connection(e.into()).into())
                        .and_then(move |conn| {
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            products_repo.find_with_base_id(base_product_id)
                        })
                })
                .map_err(|e| e.context("Service Product, find_with_base_id endpoint error occured.").into()),
        )
    }

    /// Get by base product id
    fn find_attributes(&self, product_id: i32) -> ServiceFuture<Vec<AttrValue>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| ControllerError::Connection(e.into()).into())
                        .and_then(move |conn| {
                            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
                            prod_attr_repo
                                .find_all_attributes(product_id)
                                .map(|pr_attrs| pr_attrs.into_iter().map(|pr_attr| pr_attr.into()).collect())
                        })
                })
                .map_err(|e| e.context("Service Product, find_attributes endpoint error occured.").into()),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;
    use std::time::SystemTime;

    use futures_cpupool::CpuPool;
    use r2d2;
    use tokio_core::reactor::Handle;

    use stq_http;
    use stq_http::client::Config as HttpConfig;
    use tokio_core::reactor::Core;

    use config::Config;
    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    fn create_product_service(
        user_id: Option<i32>,
        handle: Arc<Handle>,
    ) -> ProductsServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        let config = Config::new().unwrap();
        let http_config = HttpConfig {
            http_client_retries: config.client.http_client_retries,
            http_client_buffer_size: config.client.http_client_buffer_size,
        };
        let client = stq_http::client::Client::new(&http_config, &handle);
        let client_handle = client.handle();

        ProductsServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            user_id: user_id,
            client_handle: client_handle,
            elastic_address: "".to_string(),
            repo_factory: MOCK_REPO_FACTORY,
        }
    }

    pub fn create_product(id: i32, base_product_id: i32) -> Product {
        Product {
            id: id,
            base_product_id: base_product_id,
            is_active: true,
            discount: None,
            photo_main: None,
            vendor_code: "vendor_code".to_string(),
            cashback: None,
            additional_photos: None,
            price: 0f64,
            currency_id: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }

    pub fn create_new_product_with_attributes(base_product_id: i32) -> NewProductWithAttributes {
        NewProductWithAttributes {
            product: create_new_product(base_product_id),
            attributes: vec![
                AttrValue {
                    attr_id: 1,
                    value: "String".to_string(),
                    meta_field: None,
                },
            ],
        }
    }

    pub fn create_new_product(base_product_id: i32) -> NewProduct {
        NewProduct {
            base_product_id: base_product_id,
            discount: None,
            photo_main: None,
            vendor_code: "vendor_code".to_string(),
            cashback: None,
            additional_photos: None,
            price: 0f64,
            currency_id: None,
        }
    }

    pub fn create_update_product() -> UpdateProduct {
        UpdateProduct {
            discount: None,
            photo_main: None,
            vendor_code: None,
            cashback: None,
            additional_photos: None,
            price: None,
            currency_id: None,
        }
    }

    pub fn create_update_product_with_attributes() -> UpdateProductWithAttributes {
        UpdateProductWithAttributes {
            product: Some(create_update_product()),
            attributes: None,
        }
    }

    #[test]
    fn test_get_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_product_service(Some(MOCK_USER_ID), handle);
        let work = service.get(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, 1);
    }

    #[test]
    fn test_list() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_product_service(Some(MOCK_USER_ID), handle);
        let work = service.list(1, 5);
        let result = core.run(work).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_create_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_product_service(Some(MOCK_USER_ID), handle);
        let new_product = create_new_product_with_attributes(MOCK_BASE_PRODUCT_ID);
        let work = service.create(new_product);
        let result = core.run(work).unwrap();
        assert_eq!(result.base_product_id, MOCK_BASE_PRODUCT_ID);
    }

    #[test]
    fn test_update_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_product_service(Some(MOCK_USER_ID), handle);
        let new_product = create_update_product_with_attributes();
        let work = service.update(1, new_product);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.base_product_id, MOCK_BASE_PRODUCT_ID);
    }

    #[test]
    fn test_deactivate_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_product_service(Some(MOCK_USER_ID), handle);
        let work = service.deactivate(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.is_active, false);
    }

}
