extern crate futures;
extern crate hyper;
extern crate serde_json;
extern crate stores_lib;
extern crate tokio_core;

use std::sync::Arc;
use std::time::SystemTime;

use futures::Stream;
use tokio_core::reactor::Core;

use stores_lib::http::client::Client;
use stores_lib::config::Config;
use stores_lib::repos::{StoresSearchRepo, StoresSearchRepoImpl};
use stores_lib::models::*;

fn create_store(id: i32, name: String) -> Store {
    Store {
        id: id,
        name: name,
        is_active: true,
        currency_id: 1,
        short_description: "short description".to_string(),
        long_description: Some("long description".to_string()),
        slug: "slug-new-store-12323".to_string(),
        cover: None,
        logo: None,
        phone: "+79138889900".to_string(),
        email: "teamer777@gmail.com".to_string(),
        address: "lenina street 11".to_string(),
        facebook_url: None,
        twitter_url: None,
        instagram_url: None,
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
        user_id: 6,
    }
}

#[test]
fn test_create() {
    let addr = "127.0.0.1:9200".to_string();
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let config = Config::new().unwrap();
    let client = Client::new(&config, &handle);
    let client_handle = client.handle();
    let client_stream = client.stream();
    handle.spawn(client_stream.for_each(|_| Ok(())));
    let mut repo = StoresSearchRepoImpl::new(client_handle, addr);
    let store = create_store(11, "New store 11".to_string());
    let work = repo.create(store);
    let _result = core.run(work).unwrap();
}

#[test]
#[ignore]
fn test_update() {
    let addr = "127.0.0.1:9200".to_string();
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let config = Config::new().unwrap();
    let client = Client::new(&config, &handle);
    let client_handle = client.handle();
    let client_stream = client.stream();
    handle.spawn(client_stream.for_each(|_| Ok(())));
    let mut repo = StoresSearchRepoImpl::new(client_handle, addr);
    let store = create_store(101, "new store 2 ".to_string());
    let work = repo.update(store);
    let _result = core.run(work).unwrap();
}


#[test]
#[ignore]
fn test_find() {
    let addr = "127.0.0.1:9200".to_string();
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let config = Config::new().unwrap();
    let client = Client::new(&config, &handle);
    let client_handle = client.handle();
    let client_stream = client.stream();
    handle.spawn(client_stream.for_each(|_| Ok(())));
    let mut repo = StoresSearchRepoImpl::new(client_handle, addr);
    let work = repo.find_by_name("store".to_string());
    let _result = core.run(work).unwrap();
}
