//! ProductsSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use serde_json;
use elastic_responses::{SearchResponse, UpdateResponse};

use models::{ElasticIndex, ElasticProduct, Filter, IndexResponse, SearchProduct};
use repos::error::RepoError as Error;
use super::types::RepoFuture;
use http::client::ClientHandle;

/// ProductsSearch repository, responsible for handling products
pub struct ProductsSearchRepoImpl {
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

pub trait ProductsSearchRepo {
    /// Find specific product by name limited by `count` parameters
    fn search(&self, prod: SearchProduct, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>>;

    /// Creates new product
    fn create(&self, product: ElasticProduct) -> RepoFuture<()>;

    /// Updates specific product
    fn update(&self, product: ElasticProduct) -> RepoFuture<()>;
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
    fn search(&self, prod: SearchProduct, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>> {
        let name_query = json!(
                [
                    {"nested": {
                        "path": "name",
                        "query": {
                            "bool": {
                                "must": {"match": {"text": prod.name}}
                            }  
                        }
                    }},
                    {"nested": {
                        "path": "short_description",
                        "query": {
                            "bool": {
                                "must": {"match": {"text": prod.name}}
                            }  
                        }
                    }},
                    {"nested": {
                        "path": "long_description",
                        "query": {
                            "bool": {
                                "must": {"match": {"text": prod.name}}
                            }  
                        }
                    }}
                ]
            );

        let props = match prod.attr_filters {
            None => json!({}),
            Some(filters) => {
                let filters: Vec<serde_json::Value> = filters
                    .into_iter()
                    .map(|attr| {
                        let attribute_name = attr.name.clone();
                        match attr.filter {
                            Filter::Equal(val) => {
                                json!({ "bool" : {"must": [{"term": {"name": attribute_name}},{"term": {"str_val": val}}]}})
                            }
                            Filter::Lte(val) => {
                                json!({ "bool" : {"must": [{"term": {"name": attribute_name}}, { "range": { "float_val": {"lte": val }}}]}})
                            }
                            Filter::Le(val) => {
                                json!({ "bool" : {"must": [{"term": {"name": attribute_name}}, { "range": { "float_val": {"le": val }}}]}})
                            }
                            Filter::Ge(val) => {
                                json!({ "bool" : {"must": [{"term": {"name": attribute_name}}, { "range": { "float_val": {"ge": val }}}]}})
                            }
                            Filter::Gte(val) => {
                                json!({ "bool" : {"must": [{"term": {"name": attribute_name}}, { "range": { "float_val": {"gte": val }}}]}})
                            }
                        }
                    })
                    .collect();
                json!({
                        "nested" : {
                            "path" : "properties",
                            "filter" : {
                                "bool" : {
                                    "must" : filters
                                }
                            }
                        }
                })
            }
        };

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : {
                    "must" : name_query,
                    "filter" : props,
                }
            }
        }).to_string();

        let url = format!(
            "http://{}/{}/_doc/_search",
            self.elastic_address,
            ElasticIndex::Product
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
    fn create(&self, product: ElasticProduct) -> RepoFuture<()> {
        let body = serde_json::to_string(&product).unwrap();
        let url = format!(
            "http://{}/{}/_doc/{}/_create",
            self.elastic_address,
            ElasticIndex::Product,
            product.id
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
    fn update(&self, product: ElasticProduct) -> RepoFuture<()> {
        let body = json!({
            "doc": product,
        }).to_string();
        let url = format!(
            "http://{}/{}/_doc/{}/_update",
            self.elastic_address,
            ElasticIndex::Product,
            product.id
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
