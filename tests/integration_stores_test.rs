extern crate serde_json;
include!("integration_tests_setup.rs");

use std::str::FromStr;

use futures::future;
use futures::Future;
use hyper::Uri;
use hyper::{Method, Request};
use hyper::header::{ContentLength, ContentType};

use stores_lib::models::*;

fn create_new_store(name: serde_json::Value) -> NewStore {
    NewStore {
        name: name,
        user_id: 1,
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
        country: None,
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
        rating: None,
        country: None,
    }
}

static MOCK_STORE_NAME_JSON: &'static str = r##"[{"lang": "en","text": "Store"}]"##;

#[test]
fn stores_crud() {
    let mut context = setup();

    //create
    let mut url = Uri::from_str(&format!("{}/stores", context.base_url)).unwrap();

    let new_store = create_new_store(serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap());
    let mut body: String = serde_json::to_string(&new_store).unwrap().to_string();

    let mut req = Request::new(Method::Post, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.set_body(body);

    let mut code = context
        .core
        .run(
            context
                .client
                .request(req)
                .and_then(|res| future::ok(res.status().as_u16())),
        )
        .unwrap();
    assert!(code >= 200 && code <= 299);

    //read
    url = Uri::from_str(&format!("{}/stores/1", context.base_url)).unwrap();

    req = Request::new(Method::Get, url.clone());
    code = context
        .core
        .run(
            context
                .client
                .request(req)
                .and_then(|res| future::ok(res.status().as_u16())),
        )
        .unwrap();
    assert!(code >= 200 && code <= 299);

    //update
    url = Uri::from_str(&format!("{}/stores/1", context.base_url)).unwrap();

    let update_store = create_update_store(serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap());
    body = serde_json::to_string(&update_store).unwrap().to_string();

    req = Request::new(Method::Put, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.set_body(body);

    code = context
        .core
        .run(
            context
                .client
                .request(req)
                .and_then(|res| future::ok(res.status().as_u16())),
        )
        .unwrap();
    assert!(code >= 200 && code <= 299);

    //delete
    url = Uri::from_str(&format!("{}/stores/1", context.base_url)).unwrap();

    req = Request::new(Method::Delete, url.clone());
    code = context
        .core
        .run(
            context
                .client
                .request(req)
                .and_then(|res| future::ok(res.status().as_u16())),
        )
        .unwrap();
    assert!(code >= 200 && code <= 299);
}
