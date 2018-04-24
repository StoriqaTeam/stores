include!("integration_tests_setup.rs");

use futures::future::Future;
use hyper::Uri;
use std::str::FromStr;
use stq_http::request_util::read_body;

#[test]
fn healthcheck_returns_ok() {
    let mut context = setup();
    let url = Uri::from_str(&format!("{}/healthcheck", context.base_url)).unwrap();
    let response = context
        .core
        .run(context.client.get(url).and_then(|resp| read_body(resp.body())))
        .unwrap();
    assert_eq!(response, "\"Ok\"");
}
