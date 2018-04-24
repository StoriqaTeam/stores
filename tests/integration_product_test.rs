extern crate serde_json;
include!("integration_tests_setup.rs");

use std::str::FromStr;
use std::time::SystemTime;

use futures::future;
use futures::Future;
use hyper::header::{ContentLength, ContentType};
use hyper::Uri;
use hyper::{Method, Request};

use stores_lib::models::*;

pub fn create_new_base_product(name: &str) -> NewBaseProduct {
    NewBaseProduct {
        name: serde_json::from_str(name).unwrap(),
        store_id: 1,
        short_description: serde_json::from_str("{}").unwrap(),
        long_description: None,
        seo_title: None,
        seo_description: None,
        currency_id: 1,
        category_id: 1,
        slug: "slug".to_string(),
    }
}
pub fn create_product(id: i32, base_product_id: i32) -> Product {
    Product {
        id: id,
        base_product_id: base_product_id,
        is_active: true,
        discount: None,
        photo_main: None,
        vendor_code: "vendor code".to_string(),
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
        vendor_code: "vendor code".to_string(),
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
        product: Some(create_update_product()),
        attributes: Some(vec![]),
    }
}

static MOCK_BASE_PRODUCT_NAME_JSON: &'static str = r##"[{"lang": "en","text": "Base Product"}]"##;

#[test]
fn products_crud() {
    let mut context = setup();

    //create base product
    let mut url = Uri::from_str(&format!("{}/base_products", context.base_url)).unwrap();

    let new_base_product = create_new_base_product(serde_json::from_str(MOCK_BASE_PRODUCT_NAME_JSON).unwrap());
    let mut body: String = serde_json::to_string(&new_base_product).unwrap().to_string();

    let mut req = Request::new(Method::Post, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.set_body(body);

    let mut code = context
        .core
        .run(context.client.request(req).and_then(|res| future::ok(res.status().as_u16())))
        .unwrap();
    assert!(code >= 200 && code <= 299);

    //create
    url = Uri::from_str(&format!("{}/products", context.base_url)).unwrap();

    let new_product = create_new_product_with_attributes(1);
    body = serde_json::to_string(&new_product).unwrap().to_string();

    req = Request::new(Method::Post, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.set_body(body);

    code = context
        .core
        .run(context.client.request(req).and_then(|res| future::ok(res.status().as_u16())))
        .unwrap();
    assert!(code >= 200 && code <= 299);

    //read
    url = Uri::from_str(&format!("{}/products/1", context.base_url)).unwrap();

    req = Request::new(Method::Get, url.clone());
    code = context
        .core
        .run(context.client.request(req).and_then(|res| future::ok(res.status().as_u16())))
        .unwrap();
    assert!(code >= 200 && code <= 299);

    //update
    url = Uri::from_str(&format!("{}/products/1", context.base_url)).unwrap();

    let update_product = create_update_product_with_attributes();
    body = serde_json::to_string(&update_product).unwrap().to_string();

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
    url = Uri::from_str(&format!("{}/products/1", context.base_url)).unwrap();

    req = Request::new(Method::Delete, url.clone());
    code = context
        .core
        .run(context.client.request(req).and_then(|res| future::ok(res.status().as_u16())))
        .unwrap();
    assert!(code >= 200 && code <= 299);
}
