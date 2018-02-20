extern crate diesel;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate serde_json;
extern crate stores_lib;
extern crate tokio_core;

use std::time::SystemTime;
use std::sync::Arc;

use futures_cpupool::CpuPool;
use tokio_core::reactor::Handle;
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;

use stores_lib::repos::*;
use stores_lib::services::*;
use stores_lib::models::*;
use stores_lib::config::Config;
use stores_lib::http::client::Client;

#[derive(Clone)]
pub struct StoresRepoMock;

impl StoresRepo for StoresRepoMock {
    fn find(&mut self, store_id: i32) -> RepoResult<Store> {
        let store = create_store(store_id, MOCK_STORE_NAME.to_string());
        Ok(store)
    }

    fn name_exists(&mut self, name_arg: String) -> RepoResult<bool> {
        Ok(name_arg == MOCK_STORE_NAME.to_string())
    }

    fn find_by_name(&mut self, name_arg: String) -> RepoResult<Store> {
        let store = create_store(1, name_arg);
        Ok(store)
    }

    fn list(&mut self, from: i32, count: i64) -> RepoResult<Vec<Store>> {
        let mut stores = vec![];
        for i in from..(from + count as i32) {
            let store = create_store(i, MOCK_STORE_NAME.to_string());
            stores.push(store);
        }
        Ok(stores)
    }

    fn create(&mut self, payload: NewStore) -> RepoResult<Store> {
        let store = create_store(1, payload.name);
        Ok(store)
    }

    fn update(&mut self, store_id: i32, payload: UpdateStore) -> RepoResult<Store> {
        let store = create_store(store_id, payload.name);

        Ok(store)
    }

    fn deactivate(&mut self, store_id: i32) -> RepoResult<Store> {
        let mut store = create_store(store_id, MOCK_STORE_NAME.to_string());
        store.is_active = false;
        Ok(store)
    }
}

#[derive(Clone)]
pub struct CacheRolesMock {}

impl RolesCache for CacheRolesMock {
    fn get(&mut self, id: i32, _con: Option<&DbConnection>) -> RepoResult<Vec<Role>> {
        match id {
            1 => Ok(vec![Role::Superuser]),
            _ => Ok(vec![Role::User]),
        }
    }
}

const MOCK_USER_ROLE: CacheRolesMock = CacheRolesMock {};

fn create_store_service(user_id: Option<i32>, handle: Arc<Handle>) -> StoresServiceImpl<CacheRolesMock> {
    let database_url = "127.0.0.1";
    let elastic_address = "127.0.0.1:9200".to_string();
    let manager = ConnectionManager::<PgConnection>::new(database_url.to_string());
    let db_pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create connection pool");
    let cpu_pool = CpuPool::new(1);

    let config = Config::new().unwrap();
    let client = Client::new(&config, &handle);
    let client_handle = client.handle();

    StoresServiceImpl {
        db_pool: db_pool,
        cpu_pool: cpu_pool,
        roles_cache: MOCK_USER_ROLE,
        user_id: user_id,
        elastic_address: elastic_address,
        client_handle: client_handle,
    }
}

pub fn create_store(id: i32, name: String) -> Store {
    Store {
        id: id,
        name: name,
        is_active: true,
        currency_id: 1,
        short_description: "short description".to_string(),
        long_description: None,
        slug: "myname".to_string(),
        cover: None,
        logo: None,
        phone: "1234567".to_string(),
        email: "example@mail.com".to_string(),
        address: "town city street".to_string(),
        facebook_url: None,
        twitter_url: None,
        instagram_url: None,
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
        user_id: MOCK_USER_ID,
    }
}

pub fn create_new_store(name: String) -> NewStore {
    NewStore {
        name: name,
        currency_id: 1,
        short_description: "short description".to_string(),
        long_description: None,
        slug: "myname".to_string(),
        cover: None,
        logo: None,
        phone: "1234567".to_string(),
        email: "example@mail.com".to_string(),
        address: "town city street".to_string(),
        facebook_url: None,
        twitter_url: None,
        instagram_url: None,
        user_id: MOCK_USER_ID,
    }
}

pub fn create_update_store(name: String) -> UpdateStore {
    UpdateStore {
        name: name,
        currency_id: None,
        short_description: None,
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
    }
}

pub const MOCK_STORES: StoresRepoMock = StoresRepoMock {};
pub static MOCK_STORE_NAME: &'static str = "store name";

#[derive(Clone)]
pub struct ProductsRepoMock;

impl ProductsRepo for ProductsRepoMock {
    fn find(&mut self, product_id: i32) -> RepoResult<Product> {
        let product = create_product(product_id, MOCK_USER_ID.to_string());
        Ok(product)
    }

    fn name_exists(&mut self, name_arg: String) -> RepoResult<bool> {
        Ok(name_arg == MOCK_USER_ID.to_string())
    }

    fn find_by_name(&mut self, name_arg: String) -> RepoResult<Product> {
        let product = create_product(1, name_arg);
        Ok(product)
    }

    fn list(&mut self, from: i32, count: i64) -> RepoResult<Vec<Product>> {
        let mut products = vec![];
        for i in from..(from + count as i32) {
            let product = create_product(i, MOCK_USER_ID.to_string());
            products.push(product);
        }
        Ok(products)
    }

    fn create(&mut self, payload: NewProduct) -> RepoResult<Product> {
        let product = create_product(1, payload.name);
        Ok(product)
    }

    fn update(&mut self, product_id: i32, payload: UpdateProduct) -> RepoResult<Product> {
        let product = create_product(product_id, payload.name);

        Ok(product)
    }

    fn deactivate(&mut self, product_id: i32) -> RepoResult<Product> {
        let mut product = create_product(product_id, MOCK_USER_ID.to_string());
        product.is_active = false;
        Ok(product)
    }
}

fn new_product_service(user_id: Option<i32>) -> ProductsServiceImpl<CacheRolesMock> {
    let database_url = "127.0.0.1";
    let manager = ConnectionManager::<PgConnection>::new(database_url.to_string());
    let db_pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create connection pool");
    let cpu_pool = CpuPool::new(1);

    ProductsServiceImpl {
        db_pool: db_pool,
        cpu_pool: cpu_pool,
        roles_cache: MOCK_USER_ROLE,
        user_id: user_id,
    }
}

pub fn create_product_service(user_id: Option<i32>) -> ProductsServiceImpl<CacheRolesMock> {
    new_product_service(user_id)
}

pub fn create_product(id: i32, name: String) -> Product {
    Product {
        id: id,
        name: name,
        is_active: true,
        store_id: 1,
        short_description: "product".to_string(),
        long_description: None,
        price: 1.0,
        currency_id: 1,
        discount: None,
        category: None,
        photo_main: None,
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
        vendor_code: None,
        cashback: None,
        default_language: Language::Russian,
    }
}

pub fn create_new_product(name: String) -> NewProduct {
    NewProduct {
        name: name,
        store_id: 1,
        currency_id: 1,
        short_description: "product".to_string(),
        long_description: None,
        price: 1.0,
        discount: None,
        category: None,
        photo_main: None,
        vendor_code: None,
        cashback: None,
        default_language: Language::Russian,
    }
}

pub fn create_update_product(name: String) -> UpdateProduct {
    UpdateProduct {
        name: name,
        currency_id: None,
        short_description: None,
        long_description: None,
        price: None,
        discount: None,
        category: None,
        photo_main: None,
        vendor_code: None,
        cashback: None,
        default_language: None,
    }
}

pub const MOCK_PRODUCTS: ProductsRepoMock = ProductsRepoMock {};
pub static MOCK_USER_ID: i32 = 1;
pub static MOCK_PRODUCT_NAME: &'static str = "show name";
