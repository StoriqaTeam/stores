extern crate diesel;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate serde_json;
extern crate stores_lib;
extern crate stq_acl;
extern crate stq_http;
extern crate stq_static_resources;
extern crate tokio_core;

use std::time::SystemTime;
use std::sync::Arc;
use std::error::Error;
use std::fmt;

use futures_cpupool::CpuPool;
use tokio_core::reactor::Handle;

use r2d2::ManageConnection;

use diesel::Connection;
use diesel::ConnectionResult;
use diesel::QueryResult;
use diesel::query_builder::AsQuery;
use diesel::query_builder::QueryFragment;
use diesel::pg::Pg;
use diesel::query_builder::QueryId;
use diesel::sql_types::HasSqlType;
use diesel::Queryable;
use diesel::deserialize::QueryableByName;
use diesel::connection::AnsiTransactionManager;
use diesel::connection::SimpleConnection;


use stq_http::client::Config as HttpConfig;
use stq_static_resources::Translation;
use stq_acl::RolesCache;

use stores_lib::repos::*;
use stores_lib::services::*;
use stores_lib::models::*;
use stores_lib::config::Config;

const MOCK_REPO_FACTORY: ReposFactoryMock = ReposFactoryMock {};
static MOCK_USER_ID: i32 = 1;
static MOCK_BASE_PRODUCT_ID: i32 = 1;
static MOCK_PRODUCT_ID: i32 = 1;
static MOCK_STORE_NAME: &'static str = "store name";
static MOCK_STORE_SLUG: &'static str = "store slug";

#[derive(Clone)]
struct CacheRolesMock;

impl RolesCache for CacheRolesMock {
    type Role = Role;

    fn get(&self, user_id: i32) -> Vec<Self::Role> {
         match user_id {
            1 => vec![Role::Superuser],
            _ => vec![Role::User],
        }
    }

    fn clear(&self) {}

    fn remove(&self, _id: i32) {}

    fn contains(&self, id: i32) -> bool {
        match id {
            1|2 => true,
            _ => false,
        }
    }

    fn add_roles(&self, _id: i32, _roles: &Vec<Self::Role>) {}

}

#[derive(Default, Copy, Clone)]
pub struct ReposFactoryMock;

impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryMock {
    fn create_attributes_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<AttributesRepo + 'a> {
        unimplemented!()
    }
    fn create_categories_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<CategoriesRepo + 'a> {
        unimplemented!()
    }
    fn create_category_attrs_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<CategoryAttrsRepo + 'a> {
        unimplemented!()
    }
    fn create_base_product_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<BaseProductsRepo + 'a> {
        unimplemented!()
    }
    fn create_product_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<ProductsRepo + 'a> {
        Box::new(ProductsRepoMock::default()) as Box<ProductsRepo>
    }
    fn create_product_attrs_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<ProductAttrsRepo + 'a> {
        unimplemented!()
    }
    fn create_stores_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<StoresRepo + 'a> {
        Box::new(StoresRepoMock::default()) as Box<StoresRepo>
    }
    fn create_user_roles_repo<'a>(&self, _db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
        unimplemented!()
    }
}

#[derive(Clone, Default)]
pub struct StoresRepoMock;

impl StoresRepo for StoresRepoMock {
    fn find(&self, store_id: i32) -> RepoResult<Store> {
        let store = create_store(store_id, serde_json::from_str(MOCK_STORE_NAME).unwrap());
        Ok(store)
    }

    fn name_exists(&self, name: Vec<Translation>) -> RepoResult<bool> {
        Ok(name.iter().any(|t| t.text == MOCK_STORE_NAME))
    }

    fn slug_exists(&self, slug: String) -> RepoResult<bool> {
        Ok(slug == MOCK_STORE_SLUG.to_string())
    }

    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<Store>> {
        let mut stores = vec![];
        for i in from..(from + count as i32) {
            let store = create_store(i, serde_json::from_str(MOCK_STORE_NAME).unwrap());
            stores.push(store);
        }
        Ok(stores)
    }

    fn create(&self, payload: NewStore) -> RepoResult<Store> {
        let store = create_store(1, payload.name);
        Ok(store)
    }

    fn update(&self, store_id: i32, payload: UpdateStore) -> RepoResult<Store> {
        let store = create_store(store_id, payload.name.unwrap());

        Ok(store)
    }

    fn deactivate(&self, store_id: i32) -> RepoResult<Store> {
        let mut store = create_store(store_id, serde_json::from_str(MOCK_STORE_NAME).unwrap());
        store.is_active = false;
        Ok(store)
    }
}

#[allow(unused)]
fn create_store_service(user_id: Option<i32>, handle: Arc<Handle>) -> StoresServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
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

    StoresServiceImpl {
        db_pool: db_pool,
        cpu_pool: cpu_pool,
        user_id: user_id,
        elastic_address: "127.0.0.1:9200".to_string(),
        client_handle: client_handle,
        repo_factory: MOCK_REPO_FACTORY,
    }
}

fn create_store(id: i32, name: serde_json::Value) -> Store {
    Store {
        id: id,
        user_id: 1,
        name: name,
        is_active: true,
        short_description: serde_json::from_str("{}").unwrap(),
        long_description: None,
        slug: "myname".to_string(),
        cover: None,
        logo: None,
        phone: Some("1234567".to_string()),
        email: Some("example@mail.com".to_string()),
        address: Some("town city street".to_string()),
        facebook_url: None,
        twitter_url: None,
        instagram_url: None,
        default_language: "en".to_string(),
        slogan: Some("fdsf".to_string()),
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
    }
}

pub fn create_new_store(name: serde_json::Value) -> NewStore {
    NewStore {
        name: name,
        user_id: MOCK_USER_ID,
        short_description: serde_json::from_str("{}").unwrap(),
        long_description: None,
        slug: "slug".to_string(),
        cover: None,
        logo: None,
        phone: Some("1234567".to_string()),
        email: Some("example@mail.com".to_string()),
        address: Some("town city street".to_string()),
        facebook_url: None,
        twitter_url: None,
        instagram_url: None,
        default_language: "en".to_string(),
        slogan: Some("fdsf".to_string()),
    }
}

pub fn create_update_store(name: serde_json::Value) -> UpdateStore {
    UpdateStore {
        name: Some(name),
        short_description: serde_json::from_str("{}").unwrap(),
        long_description: None,
        slug: None,
        cover: None,
        logo: None,
        phone: None,
        email: None,
        address: None,
        facebook_url: None,
        twitter_url: None,
        instagram_url: None,
        default_language: None,
        slogan: None,
    }
}

#[derive(Clone, Default)]
pub struct ProductsRepoMock;

impl ProductsRepo for ProductsRepoMock {
    fn find(&self, product_id: i32) -> RepoResult<Product> {
        let product = create_product(product_id, MOCK_BASE_PRODUCT_ID);
        Ok(product)
    }

    fn find_with_base_id(&self, base_id: i32) -> RepoResult<Vec<Product>> {
        let mut products = vec![];
        let product = create_product(MOCK_PRODUCT_ID, base_id);
        products.push(product);
        Ok(products)
    }

    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<Product>> {
        let mut products = vec![];
        for i in from..(from + count as i32) {
            let product = create_product(i, MOCK_BASE_PRODUCT_ID);
            products.push(product);
        }
        Ok(products)
    }

    fn create(&self, payload: NewProduct) -> RepoResult<Product> {
        let product = create_product(MOCK_PRODUCT_ID, payload.base_product_id);
        Ok(product)
    }

    fn update(&self, product_id: i32, _payload: UpdateProduct) -> RepoResult<Product> {
        let product = create_product(product_id, MOCK_BASE_PRODUCT_ID);

        Ok(product)
    }

    fn deactivate(&self, product_id: i32) -> RepoResult<Product> {
        let mut product = create_product(product_id, MOCK_BASE_PRODUCT_ID);
        product.is_active = false;
        Ok(product)
    }
}

#[allow(unused)]
fn create_product_service(user_id: Option<i32>, handle: Arc<Handle>) -> ProductsServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
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
    }
}

pub fn create_update_product() -> UpdateProduct {
    UpdateProduct {
        discount: None,
        photo_main: None,
        vendor_code: None,
        cashback: None,
    }
}

pub fn create_update_product_with_attributes() -> UpdateProductWithAttributes {
    UpdateProductWithAttributes {
        product: create_update_product(),
        attributes: vec![],
    }
}

#[derive(Default)]
struct MockConnection;

impl Connection for MockConnection {
    type Backend = Pg;
    type TransactionManager = AnsiTransactionManager;

    fn establish(_database_url: &str) -> ConnectionResult<MockConnection> {
        Ok(MockConnection {})
    }

    fn execute(&self, _query: &str) -> QueryResult<usize> {
        unimplemented!()
    }

    fn query_by_index<T, U>(&self, _source: T) -> QueryResult<Vec<U>>
    where
        T: AsQuery,
        T::Query: QueryFragment<Pg> + QueryId,
        Pg: HasSqlType<T::SqlType>,
        U: Queryable<T::SqlType, Pg>,
    {
        unimplemented!()
    }

    fn query_by_name<T, U>(&self, _source: &T) -> QueryResult<Vec<U>>
    where
        T: QueryFragment<Pg> + QueryId,
        U: QueryableByName<Pg>,
    {
        unimplemented!()
    }

    fn execute_returning_count<T>(&self, _source: &T) -> QueryResult<usize>
    where
        T: QueryFragment<Pg> + QueryId,
    {
        unimplemented!()
    }

    fn transaction_manager(&self) -> &Self::TransactionManager {
        unimplemented!()
    }
}

impl SimpleConnection for MockConnection {
    fn batch_execute(&self, _query: &str) -> QueryResult<()> {
        Ok(())
    }
}

#[derive(Default)]
struct MockConnectionManager;

impl ManageConnection for MockConnectionManager {
    type Connection = MockConnection;
    type Error = MockError;

    fn connect(&self) -> Result<MockConnection, MockError> {
        Ok(MockConnection {})
    }

    fn is_valid(&self, _conn: &mut MockConnection) -> Result<(), MockError> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut MockConnection) -> bool {
        false
    }
}

#[derive(Debug)]
struct MockError {}

impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SuperError is here!")
    }
}

impl Error for MockError {
    fn description(&self) -> &str {
        "I'm the superhero of errors"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
