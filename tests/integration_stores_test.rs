extern crate serde_json;
include!("integration_tests_setup.rs");

use std::str::FromStr;

use hyper::header::{Authorization, ContentLength, ContentType};
use hyper::Uri;
use hyper::{Method, Request};

use stq_types::*;

use stores_lib::models::*;

fn create_new_store(name: &str, short_description: &str) -> NewStore {
    NewStore {
        name: serde_json::from_str(name).unwrap(),
        user_id: UserId(1),
        short_description: serde_json::from_str(short_description).unwrap(),
        long_description: None,
        slug: rand::thread_rng().gen_ascii_chars().take(10).collect::<String>().to_lowercase(),
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
        administrative_area_level_1: None,
        administrative_area_level_2: None,
        locality: None,
        political: None,
        postal_code: None,
        route: None,
        street_number: None,
        place_id: None,
    }
}

pub fn create_update_store(name: &str, short_description: &str) -> UpdateStore {
    UpdateStore {
        name: Some(serde_json::from_str(name).unwrap()),
        short_description: Some(serde_json::from_str(short_description).unwrap()),
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
        product_categories: None,
        status: None,
        administrative_area_level_1: None,
        administrative_area_level_2: None,
        locality: None,
        political: None,
        postal_code: None,
        route: None,
        street_number: None,
        place_id: None,
    }
}

static MOCK_STORE_NAME_JSON: &'static str = r##"[{"lang": "en","text": "Store"}]"##;
static MOCK_SHORT_DESCRIPTION_JSON: &'static str = r##"[{"lang": "en","text": "Short Description"}]"##;

#[test]
fn stores_crud() {
    let mut context = setup();

    //create
    let mut url = Uri::from_str(&format!("{}/stores", context.base_url)).unwrap();

    let new_store = create_new_store(MOCK_STORE_NAME_JSON, MOCK_SHORT_DESCRIPTION_JSON);
    let mut body: String = serde_json::to_string(&new_store).unwrap().to_string();

    let mut req = Request::new(Method::Post, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.headers_mut().set(Authorization("1".to_string()));
    req.set_body(body);

    let mut code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<Store>(&code);
    assert!(value.is_ok());

    let id = value.unwrap().id;

    //read
    url = Uri::from_str(&format!("{}/stores/{}", context.base_url, id)).unwrap();

    req = Request::new(Method::Get, url.clone());
    code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<Store>(&code);
    assert!(value.is_ok());

    //update
    url = Uri::from_str(&format!("{}/stores/{}", context.base_url, id)).unwrap();

    let update_store = create_update_store(MOCK_STORE_NAME_JSON, MOCK_SHORT_DESCRIPTION_JSON);
    body = serde_json::to_string(&update_store).unwrap().to_string();

    req = Request::new(Method::Put, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.headers_mut().set(Authorization("1".to_string()));
    req.set_body(body);

    code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<Store>(&code);
    assert!(value.is_ok());

    //delete
    url = Uri::from_str(&format!("{}/stores/{}", context.base_url, id)).unwrap();

    req = Request::new(Method::Delete, url.clone());
    req.headers_mut().set(Authorization("1".to_string()));
    code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<Store>(&code);
    assert!(value.is_ok());
}
