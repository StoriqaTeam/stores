//! ProductsSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use serde_json;
use elastic_responses::{SearchResponse, UpdateResponse};

use models::{AttributeFilter, ElasticProduct, IndexResponse, SearchProduct};
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
    fn search(&mut self, prod: SearchProduct, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>>;

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
    fn search(&mut self, prod: SearchProduct, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>> {
        let name_query = match prod.name {
            None => json!({
                "match_all": {}
            }),
            Some(name) => json!({
                 "multi_match" : {
                    "query":      name,
                    "fields":     [ "name", "short_description", "long_description" ],
                    "operator":   "or" 
                }
            }),
        };

        let props = match prod.attr_filters {
            None => json!({}),
            Some(filters) => {
                let fil: Vec<serde_json::Value> = filters
                    .into_iter()
                    .map(|filter| match filter {
                        AttributeFilter::EqualBool {
                            attribute_name,
                            attribute_value,
                        } => json!({ "must": [{"term": {"attribute_name": attribute_name}},
                                                 {"term": {"attribute_value": attribute_value}}],
                                }),
                        AttributeFilter::EqualEnum {
                            attribute_name,
                            attribute_value,
                        } => json!({ "must": [{"term": {"attribute_name": attribute_name}},
                                                 {"term": {"attribute_value": attribute_value}}],
                                }),
                        AttributeFilter::MinNum {
                            attribute_name,
                            attribute_value,
                        } => json!({ "must": [{"term": {"attribute_name": attribute_name}},
                                                 { "range": { "value": { "gte": attribute_value }}}],
                                }),
                        AttributeFilter::MaxNum {
                            attribute_name,
                            attribute_value,
                        } => json!({ "must": [{"term": {"attribute_name": attribute_name}},
                                                 { "range": { "value": { "lte": attribute_value }}}],
                                }),
                        AttributeFilter::EqualNum {
                            attribute_name,
                            attribute_value,
                        } => json!({ "must": [{"term": {"attribute_name": attribute_name}},
                                                 {"term": {"attribute_value": attribute_value}}],
                                }),
                        AttributeFilter::RangeNum {
                            attribute_name,
                            attribute_value_min,
                            attribute_value_max,
                        } => json!({ "must": [{"term": {"attribute_name": attribute_name}},
                                                 { "range": { "value": { "gte": attribute_value_min, "lte": attribute_value_max }}}],
                                }),
                    })
                    .collect();
                json!({
                        "nested" : {
                            "path" : "properties",
                            "filter" : {
                                "bool" : fil
                            }
                        }
                })
            }
        };

        let categories = match prod.cat_filters {
            None => json!({}),
            Some(cat) => {
                let categories = cat.iter()
                    .fold("".to_string(), |sum, val| format!("{} {}", sum, val));
                json!({
                    "query": {
                        "simple_query_string" : {
                            "fields" : ["categories"],
                            "query" : categories
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
                    "filter" : categories
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

// curl -XPUT 'stores-es:9200/stores/product?pretty' -H 'Content-Type: application/json' -d'
// {
//   "mappings": {
//     "_doc": {
//       "properties": {
//         "name": {
//             "type": "integer"
//         },
//         "name": {
//             "type": "string"
//         },
//         "properties": {
//             "type": "nested"
//         },
//         "short_description": {
//             "type": "string"
//         },
//         "long_description": {
//             "type": "string"
//         },
//         "categories": {
//             "type": "nested"
//         },
//       }
//     }
//   }
// }
