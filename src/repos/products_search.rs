//! ProductsSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use serde_json;
use elastic_responses::{SearchResponse, UpdateResponse};

use models::{ElasticProduct, IndexResponse};
use super::error::Error;
use super::types::RepoFuture;
use http::client::ClientHandle;

/// ProductsSearch repository, responsible for handling products
pub struct ProductsSearchRepoImpl {
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

pub static ELASTIC_INDEX: &'static str = "stores";
pub static ELASTIC_TYPE_PRODUCT: &'static str = "product";

pub trait ProductsSearchRepo {
    /// Find specific product by name limited by `count` parameters
    fn find_by_name(&mut self, name: String, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>>;

    /// Creates new product
    fn create(&mut self, product: ElasticProduct) -> RepoFuture<()>;

    /// Updates specific product
    fn update(&mut self, product: ElasticProduct) -> RepoFuture<()>;
}

impl ProductsSearchRepoImpl {
    pub fn new(client_handle: ClientHandle, elastic_address: String) -> Self {
        Self {
            client_handle,
            elastic_address,
        }
    }
}

impl ProductsSearchRepo for ProductsSearchRepoImpl {
    /// Find specific products by name limited by `count` parameters
    fn find_by_name(&mut self, name: String, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>> {
        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                    "bool" : {
                    "should" : [
                        { "match" : { "name" : name } },
                        { "match" : { "short_description" : name } },
                        { "match" : { "long_description" : name } },
                    ],
                    }
            }
        }).to_string();
        let url = format!(
            "http://{}/{}/{}/_search",
            self.elastic_address, ELASTIC_INDEX, ELASTIC_TYPE_PRODUCT
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticProduct>>(Method::Get, url, Some(query), Some(headers))
                .map_err(Error::from)
                .and_then(|res| future::ok(res.into_documents().collect::<Vec<ElasticProduct>>())),
        )
    }

    /// Creates new product
    fn create(&mut self, product: ElasticProduct) -> RepoFuture<()> {
        let body = serde_json::to_string(&product).unwrap();
        let url = format!(
            "http://{}/{}/{}/{}/_create",
            self.elastic_address, ELASTIC_INDEX, ELASTIC_TYPE_PRODUCT, product.id
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());

        Box::new(
            self.client_handle
                .request::<IndexResponse>(Method::Post, url, Some(body), Some(headers))
                .map_err(Error::from)
                .and_then(|res| {
                    if res.is_created() {
                        future::ok(())
                    } else {
                        future::err(Error::NotFound)
                    }
                }),
        )
    }

    /// Updates specific product
    fn update(&mut self, product: ElasticProduct) -> RepoFuture<()> {
        let body = json!({
            "doc": product,
        }).to_string();
        let url = format!(
            "http://{}/{}/{}/{}/_update",
            self.elastic_address, ELASTIC_INDEX, ELASTIC_TYPE_PRODUCT, product.id
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());

        Box::new(
            self.client_handle
                .request::<UpdateResponse>(Method::Post, url, Some(body), Some(headers))
                .map_err(Error::from)
                .and_then(|res| {
                    if res.updated() {
                        future::ok(())
                    } else {
                        future::err(Error::NotFound)
                    }
                }),
        )
    }
}
