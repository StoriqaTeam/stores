//! ProductsSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentLength, ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use serde_json;
use stq_http::client::ClientHandle;

use models::{ElasticIndex, ElasticProduct, MostDiscountProducts, MostViewedProducts, SearchOptions, SearchProductsByName, SearchResponse};
use repos::error::RepoError as Error;
use repos::types::RepoFuture;
use super::{log_elastic_req, log_elastic_resp};

/// ProductsSearch repository, responsible for handling products
pub struct ProductsElasticImpl {
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

pub trait ProductsElastic {
    /// Find specific product by name limited by `count` parameters
    fn auto_complete(&self, name: String, count: i32, offset: i32) -> RepoFuture<Vec<String>>;

    /// Find specific product by name limited by `count` parameters
    fn search_by_name(&self, prod: SearchProductsByName, count: i32, offset: i32) -> RepoFuture<Vec<ElasticProduct>>;

    /// Find product by views limited by `count` and `offset` parameters
    fn search_most_viewed(&self, prod: MostViewedProducts, count: i32, offset: i32) -> RepoFuture<Vec<ElasticProduct>>;

    /// Find product by dicount pattern limited by `count` and `offset` parameters
    fn search_most_discount(&self, prod: MostDiscountProducts, count: i32, offset: i32) -> RepoFuture<Vec<ElasticProduct>>;
}

impl ProductsElasticImpl {
    pub fn new(client_handle: ClientHandle, elastic_address: String) -> Self {
        Self {
            client_handle,
            elastic_address,
        }
    }

    fn create_elastic_filters(options: Option<SearchOptions>) -> Vec<serde_json::Value> {
        let mut filters: Vec<serde_json::Value> = vec![];
        let (attr_filters, categories_ids, price_filters) = if let Some(options) = options {
            let attr_filters = options
                .attr_filters
                .into_iter()
                .map(|attr| {
                    if let Some(range) = attr.range {
                        let mut range_map = serde_json::Map::<String, serde_json::Value>::new();
                        if let Some(min) = range.min_value {
                            range_map.insert("gte".to_string(), json!(min));
                        }
                        if let Some(max) = range.max_value {
                            range_map.insert("lte".to_string(), json!(max));
                        }
                        json!({ "bool" : {"must": [{"term": {"variants.attrs.attr_id": attr.id}}, { "range": { "variants.attrs.float_val": range_map}}]}})
                    } else if let Some(equal) = attr.equal {
                        json!({ "bool" : {"must": [{"term": {"variants.attrs.attr_id": attr.id}},{"terms": {"variants.attrs.str_val": equal.values}}]}})
                    } else {
                        json!({})
                    }
                })
                .collect::<Vec<serde_json::Value>>();
            (
                Some(attr_filters),
                Some(options.categories_ids),
                options.price_range,
            )
        } else {
            (None, None, None)
        };

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

        if let Some(attr_filters) = attr_filters {
            if !attr_filters.is_empty() {
                filters.push(attr_filter);
            }
        }

        let category = json!({
                "terms": {"category_id": categories_ids}
            });

        if let Some(ids) = categories_ids {
            if !ids.is_empty() {
                filters.push(category);
            }
        }

        if let Some(price_filters) = price_filters {
            let mut range_map = serde_json::Map::<String, serde_json::Value>::new();
            if let Some(min) = price_filters.min_value {
                range_map.insert("gte".to_string(), json!(min));
            }
            if let Some(max) = price_filters.max_value {
                range_map.insert("lte".to_string(), json!(max));
            }
            let price_filter = json!({
                "nested" : {
                    "path" : "variants",
                    "query" : { "bool" : {"must": { "range": { "variants.price": range_map}}}}
                    }
            });
            filters.push(price_filter);
        }

        let variants_exists = json!({
                "nested": {
                    "path": "variants",
                    "query": {
                    "bool": {
                        "filter": {
                        "exists": {
                            "field": "variants"
                        }
                        }
                    }
                    }
                }
            });

        filters.push(variants_exists);

        filters
    }
}

impl ProductsElastic for ProductsElasticImpl {
    /// Find specific products by name limited by `count` parameters
    fn search_by_name(&self, prod: SearchProductsByName, count: i32, offset: i32) -> RepoFuture<Vec<ElasticProduct>> {
        log_elastic_req(&prod);
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

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();
        if !prod.name.is_empty() {
            query_map.insert("must".to_string(), name_query);
        }

        let filters = ProductsElasticImpl::create_elastic_filters(prod.options);
        if !filters.is_empty() {
            query_map.insert("filter".to_string(), serde_json::Value::Array(filters));
        }

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : query_map
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
                .inspect(|ref res| log_elastic_resp(res))
                .and_then(|res| future::ok(res.into_documents().collect::<Vec<ElasticProduct>>())),
        )
    }

    /// Find product by views limited by `count` and `offset` parameters
    fn search_most_viewed(&self, prod: MostViewedProducts, count: i32, offset: i32) -> RepoFuture<Vec<ElasticProduct>> {
        log_elastic_req(&prod);

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();

        let filters = ProductsElasticImpl::create_elastic_filters(prod.options);
        if !filters.is_empty() {
            query_map.insert("filter".to_string(), serde_json::Value::Array(filters));
        }

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : query_map
            },
            "sort" : [{ "views" : { "order" : "desc"} }]
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
                .inspect(|ref res| log_elastic_resp(res))
                .and_then(|res| future::ok(res.into_documents().collect::<Vec<ElasticProduct>>())),
        )
    }

    /// Find product by dicount pattern limited by `count` and `offset` parameters
    fn search_most_discount(&self, prod: MostDiscountProducts, count: i32, offset: i32) -> RepoFuture<Vec<ElasticProduct>> {
        log_elastic_req(&prod);

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();

        let filters = ProductsElasticImpl::create_elastic_filters(prod.options);
        if !filters.is_empty() {
            query_map.insert("filter".to_string(), serde_json::Value::Array(filters));
        }
        let discount_exists = json!({
                "nested": {
                    "path": "variants",
                    "query": {
                    "bool": {
                        "filter": {
                            "exists": {
                                "field": "variants.discount"
                            }
                        }
                    }
                    }
                }
            });

        query_map.insert("must".to_string(), discount_exists);

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : query_map
            },
            "sort" : [{ "variants.discount" : { "order" : "desc"} }]
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
                .inspect(|ref res| log_elastic_resp(res))
                .and_then(|res| future::ok(res.into_documents().collect::<Vec<ElasticProduct>>())),
        )
    }

    fn auto_complete(&self, name: String, count: i32, _offset: i32) -> RepoFuture<Vec<String>> {
        log_elastic_req(&name);
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
                .inspect(|ref res| log_elastic_resp(res))
                .and_then(|res| future::ok(res.suggested_texts())),
        )
    }
}
