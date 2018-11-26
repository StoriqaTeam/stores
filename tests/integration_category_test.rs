extern crate serde_json;
include!("integration_tests_setup.rs");

use std::str::FromStr;

use hyper::header::{Authorization, ContentLength, ContentType};
use hyper::Uri;
use hyper::{Method, Request};

use stq_types::{CategoryId, CategorySlug};

use stores_lib::models::*;

pub fn create_new_category(name: &str) -> NewCategory {
    NewCategory {
        name: serde_json::from_str(name).unwrap(),
        meta_field: None,
        parent_id: CategoryId(1),
        uuid: uuid::Uuid::new_v4(),
        slug: None,
    }
}

pub fn create_update_category(name: &str) -> UpdateCategory {
    UpdateCategory {
        name: Some(serde_json::from_str(name).unwrap()),
        meta_field: None,
        parent_id: Some(CategoryId(1)),
        level: Some(3),
        slug: Some(CategorySlug(name.to_string())),
    }
}

static MOCK_CATEGORY_NAME_JSON: &'static str = r##"[{"lang": "en","text": "Category"}]"##;

#[ignore]
#[test]
fn categories_crud() {
    let mut context = setup();

    //create
    let mut url = Uri::from_str(&format!("{}/categories", context.base_url)).unwrap();

    let new_category = create_new_category(MOCK_CATEGORY_NAME_JSON);
    let mut body: String = serde_json::to_string(&new_category).unwrap().to_string();

    let mut req = Request::new(Method::Post, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.headers_mut().set(Authorization("1".to_string()));
    req.set_body(body);

    let mut code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<Category>(&code);
    assert!(value.is_ok());

    let id = value.unwrap().id;

    //read
    url = Uri::from_str(&format!("{}/categories/{}", context.base_url, id)).unwrap();

    req = Request::new(Method::Get, url.clone());
    code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<Category>(&code);
    assert!(value.is_ok());

    //update
    url = Uri::from_str(&format!("{}/categories/{}", context.base_url, id)).unwrap();

    let update_category = create_update_category(MOCK_CATEGORY_NAME_JSON);
    body = serde_json::to_string(&update_category).unwrap().to_string();

    req = Request::new(Method::Put, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.headers_mut().set(Authorization("1".to_string()));
    req.set_body(body);

    code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<Category>(&code);
    assert!(value.is_ok());
}
