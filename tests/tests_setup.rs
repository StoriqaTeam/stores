extern crate futures;
extern crate hyper;
extern crate serde_json;
extern crate stores_lib;
extern crate tokio_core;


use stores_lib::repos::stores::StoresRepo;
use stores_lib::repos::products::ProductsRepo;
use stores_lib::repos::types::RepoFuture;
use stores_lib::services::stores::{StoresService, StoresServiceImpl};
use stores_lib::models::{NewStore, Store, UpdateStore};
use stores_lib::services::products::{ProductsService, ProductsServiceImpl};
use stores_lib::models::{NewProduct, Product, UpdateProduct};

#[derive(Clone)]
pub struct StoresRepoMock;

impl StoresRepo for StoresRepoMock {
    fn find(&self, store_id: i32) -> RepoFuture<Store> {
        let store = create_store(store_id, MOCK_NAME.to_string());
        Box::new(futures::future::ok(store))
    }

    fn name_exists(&self, name_arg: String) -> RepoFuture<bool> {
        Box::new(futures::future::ok(name_arg == MOCK_NAME.to_string()))
    }

    fn find_by_name(&self, name_arg: String) -> RepoFuture<Store> {
        let store = create_store(1, name_arg);
        Box::new(futures::future::ok(store))
    }

    fn list(&self, from: i32, count: i64) -> RepoFuture<Vec<Store>> {
        let mut stores = vec![];
        for i in from..(from + count as i32) {
            let store = create_store(i, MOCK_NAME.to_string());
            stores.push(store);
        }
        Box::new(futures::future::ok(stores))
    }

    fn create(&self, payload: NewStore) -> RepoFuture<Store> {
        let store = create_store(1, payload.name);
        Box::new(futures::future::ok(store))
    }

    fn update(&self, store_id: i32, payload: UpdateStore) -> RepoFuture<Store> {
        let store = create_store(store_id, payload.name);

        Box::new(futures::future::ok(store))
    }

    fn deactivate(&self, store_id: i32) -> RepoFuture<Store> {
        let mut store = create_store(store_id, MOCK_NAME.to_string());
        store.is_active = false;
        Box::new(futures::future::ok(store))
    }
}


fn new_store_service(
    stores_repo: StoresRepoMock,
    user_email: Option<String>,
) -> StoresServiceImpl<StoresRepoMock> {
    StoresServiceImpl {
        stores_repo,
        user_email,
    }
}

pub fn create_store_service(user_email: Option<String>) -> StoresServiceImpl<StoresRepoMock> {
    new_store_service(MOCK_STORES, user_email)
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
        pinterest_url: None,
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
        pinterest_url: None,
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
        pinterest_url: None,
    }
}

pub const MOCK_STORES: StoresRepoMock = StoresRepoMock {};
pub static MOCK_NAME: &'static str = "store name";

#[derive(Clone)]
pub struct ProductsRepoMock;

impl ProductsRepo for ProductsRepoMock {
    fn find(&self, product_id: i32) -> RepoFuture<Product> {
        let product = create_product(product_id, MOCK_PRODUCT_NAME.to_string());
        Box::new(futures::future::ok(product))
    }

    fn name_exists(&self, name_arg: String) -> RepoFuture<bool> {
        Box::new(futures::future::ok(
            name_arg == MOCK_PRODUCT_NAME.to_string(),
        ))
    }

    fn find_by_name(&self, name_arg: String) -> RepoFuture<Product> {
        let product = create_product(1, name_arg);
        Box::new(futures::future::ok(product))
    }

    fn list(&self, from: i32, count: i64) -> RepoFuture<Vec<Product>> {
        let mut products = vec![];
        for i in from..(from + count as i32) {
            let product = create_product(i, MOCK_PRODUCT_NAME.to_string());
            products.push(product);
        }
        Box::new(futures::future::ok(products))
    }

    fn create(&self, payload: NewProduct) -> RepoFuture<Product> {
        let product = create_product(1, payload.name);
        Box::new(futures::future::ok(product))
    }

    fn update(&self, product_id: i32, payload: UpdateProduct) -> RepoFuture<Product> {
        let product = create_product(
            product_id,
            payload.name,
        );

        Box::new(futures::future::ok(product))
    }

    fn deactivate(&self, product_id: i32) -> RepoFuture<Product> {
        let mut product = create_product(product_id, MOCK_PRODUCT_NAME.to_string());
        product.is_active = false;
        Box::new(futures::future::ok(product))
    }
}

pub fn new_product_service(
    products_repo: ProductsRepoMock,
    user_email: Option<String>,
) -> ProductsServiceImpl<ProductsRepoMock> {
    ProductsServiceImpl {
        products_repo,
        user_email,
    }
}

pub fn create_product_service(
    users_email: Option<String>,
) -> ProductsServiceImpl<ProductsRepoMock> {
    new_product_service(MOCK_PRODUCTS, users_email)
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
    }
}

pub const MOCK_PRODUCTS: ProductsRepoMock = ProductsRepoMock {};
pub static MOCK_PRODUCT_NAME: &'static str = "show name";
