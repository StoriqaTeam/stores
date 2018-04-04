//! Products Services, presents CRUD operations with product

use futures_cpupool::CpuPool;
use diesel::Connection;

use stq_http::client::ClientHandle;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;

use r2d2::{ManageConnection, Pool};

use models::*;
use super::types::ServiceFuture;
use repos::ReposFactory;
use super::error::ServiceError;

pub trait ProductsService {
    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<Product>;
    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<Product>;
    /// Creates base product
    fn create(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product>;
    /// Lists product variants limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> ServiceFuture<Vec<Product>>;
    /// Updates  product
    fn update(&self, product_id: i32, payload: UpdateProductWithAttributes) -> ServiceFuture<Product>;
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
    fn get(&self, product_id: i32) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

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
                    let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                    products_repo.find(product_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

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
                    let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                    products_repo
                        .deactivate(product_id)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> ServiceFuture<Vec<Product>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

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
                    let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                    products_repo.list(from, count).map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new product
    fn create(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let cpu_pool = self.cpu_pool.clone();
        let repo_factory = self.repo_factory.clone();

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
                    let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                    let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
                    let attr_repo = repo_factory.create_attributes_repo(&*conn, user_id);
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

        let cpu_pool = self.cpu_pool.clone();
        let repo_factory = self.repo_factory.clone();

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
                    let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                    let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
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

#[cfg(test)]
pub mod tests {
    use std::time::SystemTime;
    use std::sync::Arc;

    use futures_cpupool::CpuPool;
    use tokio_core::reactor::Handle;
    use r2d2;

    use stq_http::client::Config as HttpConfig;
    use stq_http;
    use tokio_core::reactor::Core;

    use repos::repo_factory::tests::*;
    use services::*;
    use models::*;
    use config::Config;

    fn create_product_service(
        user_id: Option<i32>,
        handle: Arc<Handle>,
    ) -> ProductsServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create connection pool");
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
            vendor_code: None,
            cashback: None,
            additional_photos: None,
            price: 0f64,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }

    pub fn create_new_product_with_attributes(base_product_id: i32) -> NewProductWithAttributes {
        NewProductWithAttributes {
            product: create_new_product(base_product_id),
            attributes: vec![],
        }
    }

    pub fn create_new_product(base_product_id: i32) -> NewProduct {
        NewProduct {
            base_product_id: base_product_id,
            discount: None,
            photo_main: None,
            vendor_code: None,
            cashback: None,
            additional_photos: None,
            price: 0f64,
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
        }
    }

    pub fn create_update_product_with_attributes() -> UpdateProductWithAttributes {
        UpdateProductWithAttributes {
            product: create_update_product(),
            attributes: vec![],
        }
    }

    #[test]
    fn test_get_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_product_service(Some(MOCK_USER_ID), handle);
        let work = service.get(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
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
    fn test_update() {
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
    fn test_deactivate() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_product_service(Some(MOCK_USER_ID), handle);
        let work = service.deactivate(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.is_active, false);
    }

}
