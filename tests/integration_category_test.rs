extern crate serde_json;
include!("integration_tests_setup.rs");

use std::str::FromStr;

use futures::future;
use futures::Future;
use hyper::Uri;
use hyper::{Method, Request};
use hyper::header::{ContentLength, ContentType};

use stores_lib::models::*;

 pub fn create_new_category(name: &str) -> NewCategory {
        NewCategory {
            name: serde_json::from_str(name).unwrap(),
            meta_field: None,
            parent_id: Some(1),
            level: 0,
        }
    }

    pub fn create_update_category(name: &str) -> UpdateCategory {
        UpdateCategory {
            name: Some(serde_json::from_str(name).unwrap()),
            meta_field: None,
            parent_id: Some(1),
            level: Some(0),
        }
    }

 static MOCK_CATEGORY_NAME_JSON: &'static str = r##"[{"lang": "en","text": "Categoryt"}]"##;

#[test]
fn categories_crud() {
    let mut context = setup();
    
    //create
    let mut url = Uri::from_str(&format!("{}/categories", context.base_url)).unwrap();
    
    let new_category = create_new_category(serde_json::from_str(MOCK_CATEGORY_NAME_JSON).unwrap());
    let mut body: String = serde_json::to_string(&new_category).unwrap().to_string();
    
    let mut req = Request::new(Method::Post, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.set_body(body);

    let mut code = context
        .core
        .run(
            context
                .client
                .request(req).and_then(|res| {
                    future::ok(res.status().as_u16())
        }))
        .unwrap();
    assert!(code >= 200 && code <=299);

    //read
    url = Uri::from_str(&format!("{}/categories/1", context.base_url)).unwrap();

    req = Request::new(Method::Get, url.clone());
    code = context
        .core
        .run(
            context
                .client
                .request(req).and_then(|res| {
                    future::ok(res.status().as_u16())
        }))
        .unwrap();
    assert!(code >= 200 && code <=299);

    //update
    url = Uri::from_str(&format!("{}/categories/1", context.base_url)).unwrap();
    
    let update_category = create_update_category(serde_json::from_str(MOCK_CATEGORY_NAME_JSON).unwrap());
    body = serde_json::to_string(&update_category).unwrap().to_string();
    
    req = Request::new(Method::Put, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.set_body(body);

    code = context
        .core
        .run(
            context
                .client
                .request(req).and_then(|res| {
                    future::ok(res.status().as_u16())
        }))
        .unwrap();
    assert!(code >= 200 && code <=299);

    //delete
    url = Uri::from_str(&format!("{}/categories/1", context.base_url)).unwrap();

    req = Request::new(Method::Delete, url.clone());
    code = context
        .core
        .run(
            context
                .client
                .request(req).and_then(|res| {
                    future::ok(res.status().as_u16())
        }))
        .unwrap();
    assert!(code >= 200 && code <=299);
}
