extern crate serde_json;
include!("integration_tests_setup.rs");

use std::str::FromStr;

use futures::future;
use futures::Future;
use hyper::header::{ContentLength, ContentType};
use hyper::Uri;
use hyper::{Method, Request};

use stores_lib::models::*;

pub fn create_new_attribute(name: &str) -> NewAttribute {
    NewAttribute {
        name: serde_json::from_str(name).unwrap(),
        value_type: AttributeType::Str,
        meta_field: None,
    }
}

pub fn create_update_attribute(name: &str) -> UpdateAttribute {
    UpdateAttribute {
        name: Some(serde_json::from_str(name).unwrap()),
        meta_field: None,
    }
}

static MOCK_ATTRIBUTE_NAME_JSON: &'static str = r##"[{"lang": "en","text": "Base Product"}]"##;

#[test]
fn attributes_crud() {
    let mut context = setup();

    //create
    let mut url = Uri::from_str(&format!("{}/attributes", context.base_url)).unwrap();

    let new_attribute = create_new_attribute(serde_json::from_str(MOCK_ATTRIBUTE_NAME_JSON).unwrap());
    let mut body: String = serde_json::to_string(&new_attribute).unwrap().to_string();

    let mut req = Request::new(Method::Post, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.set_body(body);

    let mut code = context
        .core
        .run(context.client.request(req).and_then(|res| future::ok(res.status().as_u16())))
        .unwrap();
    assert!(code >= 200 && code <= 299);

    //read
    url = Uri::from_str(&format!("{}/attributes/1", context.base_url)).unwrap();

    req = Request::new(Method::Get, url.clone());
    code = context
        .core
        .run(context.client.request(req).and_then(|res| future::ok(res.status().as_u16())))
        .unwrap();
    assert!(code >= 200 && code <= 299);

    //update
    url = Uri::from_str(&format!("{}/attributes/1", context.base_url)).unwrap();

    let update_attribute = create_update_attribute(serde_json::from_str(MOCK_ATTRIBUTE_NAME_JSON).unwrap());
    body = serde_json::to_string(&update_attribute).unwrap().to_string();

    req = Request::new(Method::Put, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.set_body(body);

    code = context
        .core
        .run(context.client.request(req).and_then(|res| future::ok(res.status().as_u16())))
        .unwrap();
    assert!(code >= 200 && code <= 299);

    //delete
    url = Uri::from_str(&format!("{}/attributes/1", context.base_url)).unwrap();

    req = Request::new(Method::Delete, url.clone());
    code = context
        .core
        .run(context.client.request(req).and_then(|res| future::ok(res.status().as_u16())))
        .unwrap();
    assert!(code >= 200 && code <= 299);
}
