//! ProductsSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use futures::future::*;
use serde_json;
use elastic_responses::SearchResponse;
use stq_http::client::ClientHandle;
use stq_static_resources::Translation;

use models::{ElasticIndex, ElasticProduct, Filter, SearchProductElastic};
use repos::error::RepoError as Error;
use repos::types::RepoFuture;

/// ProductsSearch repository, responsible for handling products
pub struct ProductsElasticImpl {
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

pub trait ProductsElastic {
    /// Find specific product by name limited by `count` parameters
    fn auto_complete(&self, name: String, count: i64, offset: i64) -> RepoFuture<Vec<String>>;

    /// Find specific product by name limited by `count` parameters
    fn search(&self, prod: SearchProductElastic, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>>;
}

impl ProductsElasticImpl {
    pub fn new(client_handle: ClientHandle, elastic_address: String) -> Self {
        Self {
            client_handle,
            elastic_address,
        }
    }
}

impl ProductsElastic for ProductsElasticImpl {
    /// Find specific products by name limited by `count` parameters
    fn search(&self, prod: SearchProductElastic, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>> {
        let name_query = json!(
                [
                    {"nested": {
                        "path": "name",
                        "query": {
                            "match": {
                                "name.text": prod.name
                            }
                        }
                    }},
                    {"nested": {
                        "path": "short_description",
                        "query": {
                            "match": {
                                "short_description.text": prod.name
                            }
                        }
                    }},
                    {"nested": {
                        "path": "long_description",
                        "query": {
                            "match": {
                                "long_description.text": prod.name
                            }
                        }
                    }}
                ]
            );

        let filters = prod.attr_filters
            .into_iter()
            .map(|(attribute_id, attr)| match attr.filter {
                Filter::Equal(val) => json!({ "bool" : {"must": [{"term": {"id": attribute_id}},{"term": {"str_val": val}}]}}),
                Filter::Lte(val) => {
                    json!({ "bool" : {"must": [{"term": {"id": attribute_id}}, { "range": { "float_val": {"lte": val }}}]}})
                }
                Filter::Le(val) => json!({ "bool" : {"must": [{"term": {"id": attribute_id}}, { "range": { "float_val": {"le": val }}}]}}),
                Filter::Ge(val) => json!({ "bool" : {"must": [{"term": {"id": attribute_id}}, { "range": { "float_val": {"ge": val }}}]}}),
                Filter::Gte(val) => {
                    json!({ "bool" : {"must": [{"term": {"id": attribute_id}}, { "range": { "float_val": {"gte": val }}}]}})
                }
            })
            .collect::<Vec<serde_json::Value>>();
        let props = json!({
                        "nested" : {
                            "path" : "properties",
                            "filter" : {
                                "bool" : {
                                    "must" : filters
                                }
                            }
                        }
                });

        let category = if !prod.categories_ids.is_empty() {
            json!({
                "query" : {
                        "bool" : {
                            "must" : {"term": {"category_id": prod.categories_ids}}
                        }
                    }
            })
        } else {
            json!({})
        };

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : {
                    "must" : name_query,
                    "filter" : props,
                    "filter" : category,
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

    fn auto_complete(&self, name: String, count: i64, offset: i64) -> RepoFuture<Vec<String>> {
        let name_query = json!(
                [
                    {"nested": {
                        "path": "name",
                        "query": {
                            "match": {
                                "name.text": name
                            }
                        }
                    }},
                    {"nested": {
                        "path": "short_description",
                        "query": {
                            "match": {
                                "short_description.text": name
                            }
                        }
                    }},
                    {"nested": {
                        "path": "long_description",
                        "query": {
                            "match": {
                                "long_description.text": name
                            }
                        }
                    }}
                ]
            );

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : {
                    "must" : name_query,
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
                .and_then(|res| {
                    res.into_documents()
                        .map(move |el_product| {
                            serde_json::from_value::<Vec<Translation>>(el_product.name)
                                .map_err(|e| Error::Unknown(e.into()))
                                .and_then(|translations| {
                                    translations
                                        .into_iter()
                                        .find(|transl| transl.text.contains(&name))
                                        .ok_or(Error::NotFound)
                                        .map(|t| t.text)
                                })
                        })
                        .collect::<Result<Vec<String>, Error>>()
                        .into_future()
                }),
        )
    }
}
