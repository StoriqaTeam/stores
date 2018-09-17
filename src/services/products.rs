//! Products Services, presents CRUD operations with product
use std::collections::HashMap;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use errors::Error;
use stq_http::client::ClientHandle;
use stq_static_resources::Currency;
use stq_types::{BaseProductId, ProductId, ProductSellerPrice, StoreId, UserId};

use super::types::ServiceFuture;
use models::*;
use repos::ReposFactory;

pub trait ProductsService {
    /// Returns product by ID
    fn get(&self, product_id: ProductId) -> ServiceFuture<Option<Product>>;
    /// Returns product seller price by ID
    fn get_seller_price(&self, product_id: ProductId) -> ServiceFuture<Option<ProductSellerPrice>>;
    /// Returns store_id by ID
    fn get_store_id(&self, product_id: ProductId) -> ServiceFuture<Option<StoreId>>;
    /// Deactivates specific product
    fn deactivate(&self, product_id: ProductId) -> ServiceFuture<Product>;
    /// Creates base product
    fn create(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product>;
    /// Lists product variants limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> ServiceFuture<Vec<Product>>;
    /// Updates  product
    fn update(&self, product_id: ProductId, payload: UpdateProductWithAttributes) -> ServiceFuture<Product>;
    /// Get by base product id
    fn find_with_base_id(&self, base_product_id: BaseProductId) -> ServiceFuture<Vec<Product>>;
    /// Get by base product id
    fn find_attributes(&self, product_id: ProductId) -> ServiceFuture<Vec<AttrValue>>;
}

/// Products services, responsible for Product-related CRUD operations
pub struct ProductsServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub user_id: Option<UserId>,
    pub currency: Currency,
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
        user_id: Option<UserId>,
        client_handle: ClientHandle,
        elastic_address: String,
        repo_factory: F,
        currency: Currency,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            user_id,
            client_handle,
            elastic_address,
            repo_factory,
            currency,
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
    fn get(&self, product_id: ProductId) -> ServiceFuture<Option<Product>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();
        let currency = self.currency;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                            products_repo.find(product_id).and_then(move |product| {
                                if let Some(mut product) = product {
                                    currency_exchange.get_exchange_for_currency(currency).map(|currencies_map| {
                                        if let Some(currency_map) = currencies_map {
                                            product.price.0 *= currency_map[&product.currency].0;
                                            product.currency = currency;
                                        }
                                        Some(product)
                                    })
                                } else {
                                    Ok(None)
                                }
                            })
                        })
                }).map_err(|e| e.context("Service Product, get endpoint error occured.").into()),
        )
    }

    /// Returns product seller price by ID
    fn get_seller_price(&self, product_id: ProductId) -> ServiceFuture<Option<ProductSellerPrice>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                            products_repo.find(product_id).and_then(move |product| {
                                if let Some(product) = product {
                                    base_products_repo.find(product.base_product_id).map(|base_product| {
                                        if let Some(base_product) = base_product {
                                            Some(ProductSellerPrice {
                                                price: product.price,
                                                currency: base_product.currency,
                                            })
                                        } else {
                                            None
                                        }
                                    })
                                } else {
                                    Ok(None)
                                }
                            })
                        })
                }).map_err(|e| e.context("Service Product, get endpoint error occured.").into()),
        )
    }

    /// Returns store_id by ID
    fn get_store_id(&self, product_id: ProductId) -> ServiceFuture<Option<StoreId>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
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
                }).map_err(|e| e.context("Service Product, get_store_id endpoint error occured.").into()),
        )
    }

    /// Deactivates specific product
    fn deactivate(&self, product_id: ProductId) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
                            conn.transaction::<(Product), FailureError, _>(move || {
                                products_repo
                                    .deactivate(product_id)
                                    .and_then(|p| prod_attr_repo.delete_all_attributes(p.id).map(|_| p))
                            })
                        })
                }).map_err(|e| e.context("Service Product, deactivate endpoint error occured.").into()),
        )
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> ServiceFuture<Vec<Product>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let currency = self.currency;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                            products_repo.list(from, count).and_then(move |products| {
                                products
                                    .into_iter()
                                    .map(|mut product| {
                                        currency_exchange.get_exchange_for_currency(currency).map(|currencies_map| {
                                            if let Some(currency_map) = currencies_map {
                                                product.price.0 *= currency_map[&product.currency].0;
                                                product.currency = currency;
                                            }
                                            product
                                        })
                                    }).collect()
                            })
                        })
                }).map_err(|e| e.context("Service Product, list endpoint error occured.").into()),
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
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
                            let attr_repo = repo_factory.create_attributes_repo(&*conn, user_id);
                            let product = payload.product;
                            let attributes = payload.attributes;

                            conn.transaction::<(Product), FailureError, _>(move || {
                                // fill currency id taken from base_product first
                                base_products_repo
                                    .find(product.base_product_id)
                                    .and_then(move |base_product| {
                                        if let Some(base_product) = base_product {
                                            Ok((product, base_product.currency).into())
                                        } else {
                                            Err(format_err!("Base product with id {} not found.", product.base_product_id)
                                                .context(Error::NotFound)
                                                .into())
                                        }
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
                                                let mut hash = HashMap::<ProductId, HashMap<i32, String>>::default();
                                                for attr in base_attrs {
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
                                                    Err(format_err!("Product with attributes {:?} already exists", attributes)
                                                        .context(Error::Validate(
                                                            validation_errors!({"attributes": ["attributes" => "Product with this attributes already exists"]}),
                                                        ))
                                                        .into())
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
                                                                Err(format_err!("Not found such attribute id : {}", attr_value.attr_id)
                                                                    .context(Error::NotFound)
                                                                    .into())
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
    fn update(&self, product_id: ProductId, payload: UpdateProductWithAttributes) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let cpu_pool = self.cpu_pool.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(
            cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
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
                                            Err(format_err!("Not found such product id : {}", product_id)
                                                .context(Error::NotFound)
                                                .into())
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
                                            let mut hash = HashMap::<ProductId, HashMap<i32, String>>::default();
                                            for attr in base_attrs {
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
                                                Err(format_err!("Product with attributes {:?} already exists", attributes).context(
                                                    Error::Validate(
                                                    validation_errors!({"attributes": ["attributes" => "Product with this attributes already exists"]}),
                                                )).into())
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
                                                                Err(format_err!("Not found such attribute id : {}", attr_value.attr_id).context(Error::NotFound).into())
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
    fn find_with_base_id(&self, base_product_id: BaseProductId) -> ServiceFuture<Vec<Product>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let currency = self.currency;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                            let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                            products_repo.find_with_base_id(base_product_id).and_then(move |products| {
                                products
                                    .into_iter()
                                    .map(|mut product| {
                                        currency_exchange.get_exchange_for_currency(currency).map(|currencies_map| {
                                            if let Some(currency_map) = currencies_map {
                                                product.price.0 *= currency_map[&product.currency].0;
                                                product.currency = currency;
                                            }
                                            product
                                        })
                                    }).collect()
                            })
                        })
                }).map_err(|e| e.context("Service Product, find_with_base_id endpoint error occured.").into()),
        )
    }

    /// Get by base product id
    fn find_attributes(&self, product_id: ProductId) -> ServiceFuture<Vec<AttrValue>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
                            prod_attr_repo
                                .find_all_attributes(product_id)
                                .map(|pr_attrs| pr_attrs.into_iter().map(|pr_attr| pr_attr.into()).collect())
                        })
                }).map_err(|e| e.context("Service Product, find_attributes endpoint error occured.").into()),
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
    use stq_static_resources::Currency;
    use stq_types::*;

    use tokio_core::reactor::Core;

    use config::Config;
    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    fn create_product_service(
        user_id: Option<UserId>,
        handle: Arc<Handle>,
    ) -> ProductsServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        let config = Config::new().unwrap();
        let http_config = config.to_http_config();
        let client = stq_http::client::Client::new(&http_config, &handle);
        let client_handle = client.handle();

        ProductsServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            user_id: user_id,
            client_handle: client_handle,
            elastic_address: "".to_string(),
            repo_factory: MOCK_REPO_FACTORY,
            currency: Currency::STQ,
        }
    }

    pub fn create_product(id: ProductId, base_product_id: BaseProductId) -> Product {
        Product {
            id: id,
            base_product_id: base_product_id,
            is_active: true,
            discount: None,
            photo_main: None,
            vendor_code: "vendor_code".to_string(),
            cashback: None,
            additional_photos: None,
            price: ProductPrice(0f64),
            currency: Currency::STQ,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }

    pub fn create_new_product_with_attributes(base_product_id: BaseProductId) -> NewProductWithAttributes {
        NewProductWithAttributes {
            product: create_new_product(base_product_id),
            attributes: vec![AttrValue {
                attr_id: 1,
                value: "String".to_string(),
                meta_field: None,
            }],
        }
    }

    pub fn create_new_product(base_product_id: BaseProductId) -> NewProductWithoutCurrency {
        NewProductWithoutCurrency {
            base_product_id: base_product_id,
            discount: None,
            photo_main: None,
            vendor_code: "vendor_code".to_string(),
            cashback: None,
            additional_photos: None,
            price: ProductPrice(0f64),
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
            currency: None,
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
        let work = service.get(ProductId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, ProductId(1));
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
        let work = service.update(ProductId(1), new_product);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, ProductId(1));
        assert_eq!(result.base_product_id, MOCK_BASE_PRODUCT_ID);
    }

    #[test]
    fn test_deactivate_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_product_service(Some(MOCK_USER_ID), handle);
        let work = service.deactivate(ProductId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.id, ProductId(1));
        assert_eq!(result.is_active, false);
    }

}
