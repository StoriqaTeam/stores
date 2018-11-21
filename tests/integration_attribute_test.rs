extern crate serde_json;
include!("integration_tests_setup.rs");

use std::str::FromStr;

use hyper::header::{Authorization, ContentLength, ContentType};
use hyper::Uri;
use hyper::{Method, Request};

use stores_lib::models::*;

use stq_http::request_util::Currency as CurrencyHeader;
use stq_static_resources::*;

pub fn create_new_attribute(name: &str) -> CreateAttributePayload {
    CreateAttributePayload {
        name: serde_json::from_str(name).unwrap(),
        value_type: AttributeType::Str,
        meta_field: Some(AttributeMetaField {
            values: Some(vec!["45".to_string(), "46".to_string()]),
            translated_values: None,
            ui_element: serde_json::Value::Null,
        }),
        values: None,
        uuid: uuid::Uuid::new_v4(),
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
    let new_attribute = create_new_attribute(MOCK_ATTRIBUTE_NAME_JSON);
    let mut body: String = serde_json::to_string(&new_attribute).unwrap().to_string();

    let mut req = Request::new(Method::Post, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.headers_mut().set(Authorization("1".to_string()));
    req.headers_mut().set(CurrencyHeader("STQ".to_string()));
    req.set_body(body);

    let mut code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    println!("Attribute string: {:?}", code);
    let value = serde_json::from_str::<Attribute>(&code);
    println!("Attributes {:?}", value);
    assert!(value.is_ok());

    let id = value.unwrap().id;
    //read
    url = Uri::from_str(&format!("{}/attributes/{}", context.base_url, id)).unwrap();

    let mut req = Request::new(Method::Get, url.clone());
    req.headers_mut().set(CurrencyHeader("STQ".to_string()));

    code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    //println!("Attribute string: {:?}", code);
    let value = serde_json::from_str::<Attribute>(&code);
    //println!("Attribute {:?}", value);
    assert!(value.is_ok());

    //update
    url = Uri::from_str(&format!("{}/attributes/{}", context.base_url, id)).unwrap();

    let update_attribute = create_update_attribute(MOCK_ATTRIBUTE_NAME_JSON);
    body = serde_json::to_string(&update_attribute).unwrap().to_string();

    req = Request::new(Method::Put, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.headers_mut().set(Authorization("1".to_string()));
    req.headers_mut().set(CurrencyHeader("STQ".to_string()));
    req.set_body(body);

    code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<Attribute>(&code);
    assert!(value.is_ok());
}
