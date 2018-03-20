//! ProductsSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentLength, ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use serde_json;
use stq_http::client::ClientHandle;

use models::{ElasticIndex, ElasticProduct, Filter, SearchProduct, SearchResponse};
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
    fn search(&self, prod: SearchProduct, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>>;
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
    fn search(&self, prod: SearchProduct, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>> {
        let name_query = json!({
            "bool" : {
                "should" : [
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
            }
        });

        let attr_filters = prod.attr_filters
            .into_iter()
            .map(|attr| match attr.filter {
                Filter::Equal(val) => {
                    json!({ "bool" : {"must": [{"term": {"variants.attrs.attr_id": attr.id}},{"term": {"variants.attrs.str_val": val}}]}})
                }
                Filter::Lte(val) => {
                    json!({ "bool" : {"must": [{"term": {"variants.attrs.attr_id": attr.id}}, { "range": { "variants.attrs.float_val": {"lte": val }}}]}})
                }
                Filter::Gte(val) => {
                    json!({ "bool" : {"must": [{"term": {"variants.attrs.attr_id": attr.id}}, { "range": { "variants.attrs.float_val": {"gte": val }}}]}})
                }
            })
            .collect::<Vec<serde_json::Value>>();

        let attr_filter = json!({
                "nested" : {
                            "path" : "variants",
                            "query" : {
                                "bool" : {
                                    "must" : {
											"nested": {
 						                       "path": "variants.attrs",
                        						"query": {
                            						"bool" : {
                                    					"must" : attr_filters 
                                                            }
                        							    }
                    						        }	
                                            }
                                        }
                                    }
                            }        
        });

        let category = json!({
                "terms": {"category_id": prod.categories_ids}
            });

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();
        if !prod.name.is_empty() {
            query_map.insert("must".to_string(), name_query);
        }
        if !attr_filters.is_empty() {
            query_map.insert("filter".to_string(), attr_filter);
        }
        if !prod.categories_ids.is_empty() {
            query_map.insert("filter".to_string(), category);
        }

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : query_map
            }
        }).to_string();

        println!("{}", query);

        let url = format!(
            "http://{}/{}/_search",
            self.elastic_address,
            ElasticIndex::Product
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticProduct>>(Method::Post, url, Some(query), Some(headers))
                .map_err(Error::from)
                .and_then(|res| future::ok(res.into_documents().collect::<Vec<ElasticProduct>>())),
        )
    }

    fn auto_complete(&self, name: String, count: i64, _offset: i64) -> RepoFuture<Vec<String>> {
        let query = json!({
            "suggest": {
                "name-suggest" : {
                    "prefix" : name,
                    "completion" : {
                        "field" : "suggest",
                        "size" : count
                    }
                }
            }
        }).to_string();

        let url = format!(
            "http://{}/{}/_search",
            self.elastic_address,
            ElasticIndex::Product
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticProduct>>(Method::Post, url, Some(query), Some(headers))
                .map_err(Error::from)
                .and_then(|res| future::ok(res.suggested_texts())),
        )
    }
}
