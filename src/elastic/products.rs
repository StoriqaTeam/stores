//! ProductsSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentLength, ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use serde_json;
use stq_http::client::ClientHandle;

use models::{ElasticIndex, ElasticProduct, Filter, MostDiscountProducts, MostViewedProducts, SearchProductsByName, SearchResponse};
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
    fn search_by_name(&self, prod: SearchProductsByName, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>>;

    /// Find product by views limited by `count` and `offset` parameters
    fn search_most_viewed(&self, prod: MostViewedProducts, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>>;

    /// Find product by dicount pattern limited by `count` and `offset` parameters
    fn search_most_discount(&self, prod: MostDiscountProducts, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>>;
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
    fn search_by_name(&self, prod: SearchProductsByName, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>> {
        debug!("Searching in elastic {:?}.", prod);
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

        let (attr_filters, categories_ids) = if let Some(options) = prod.options {
            let filters = options
                .attr_filters
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
            (Some(filters), Some(options.categories_ids))
        } else {
            (None, None)
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

        let category = json!({
                "terms": {"category_id": categories_ids}
            });

        if let Some(filters) = attr_filters {
            if !filters.is_empty() {
                query_map.insert("filter".to_string(), attr_filter);
            }
        }

        if let Some(ids) = categories_ids {
            if !ids.is_empty() {
                query_map.insert("filter".to_string(), category);
            }
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
                .and_then(|res| {
                    debug!("Result of searching in elastic {:?}.", res);
                    future::ok(res.into_documents().collect::<Vec<ElasticProduct>>())
                }),
        )
    }

    /// Find product by views limited by `count` and `offset` parameters
    fn search_most_viewed(&self, prod: MostViewedProducts, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>> {
        debug!("Searching in elastic {:?}.", prod);
        let max_views_agg = json!({
            "max_views" : { "max" : { "field" : "views" } }
        });

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();
        query_map.insert("aggs".to_string(), max_views_agg);

        let (attr_filters, categories_ids) = if let Some(options) = prod.options {
            let filters = options
                .attr_filters
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
            (Some(filters), Some(options.categories_ids))
        } else {
            (None, None)
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

        let category = json!({
                "terms": {"category_id": categories_ids}
            });

        if let Some(filters) = attr_filters {
            if !filters.is_empty() {
                query_map.insert("filter".to_string(), attr_filter);
            }
        }

        if let Some(ids) = categories_ids {
            if !ids.is_empty() {
                query_map.insert("filter".to_string(), category);
            }
        }

        let query = json!({
            "from" : offset, "size" : count,
            "aggs" : {
                "most_viewed_products" : query_map
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
                .and_then(|res| {
                    debug!("Result of searching in elastic {:?}.", res);
                    future::ok(res.into_documents().collect::<Vec<ElasticProduct>>())
                }),
        )
    }

    /// Find product by dicount pattern limited by `count` and `offset` parameters
    fn search_most_discount(&self, prod: MostDiscountProducts, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>> {
        debug!("Searching in elastic {:?}.", prod);
        let max_views_agg = json!({
           "resellers" : {
                "nested" : {
                    "path" : "variants"
                },
                "aggs" : {
                    "max_discount" : { "max" : { "field" : "variants.discount" } }
                }
            }
        });

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();
        query_map.insert("aggs".to_string(), max_views_agg);

        let (attr_filters, categories_ids) = if let Some(options) = prod.options {
            let filters = options
                .attr_filters
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
            (Some(filters), Some(options.categories_ids))
        } else {
            (None, None)
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

        let category = json!({
                "terms": {"category_id": categories_ids}
            });

        if let Some(filters) = attr_filters {
            if !filters.is_empty() {
                query_map.insert("filter".to_string(), attr_filter);
            }
        }

        if let Some(ids) = categories_ids {
            if !ids.is_empty() {
                query_map.insert("filter".to_string(), category);
            }
        }

        let query = json!({
            "from" : offset, "size" : count,
            "aggs" : {
                "most_viewed_products" : query_map
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
                .and_then(|res| {
                    debug!("Result of searching in elastic {:?}.", res);
                    future::ok(res.into_documents().collect::<Vec<ElasticProduct>>())
                }),
        )
    }

    fn auto_complete(&self, name: String, count: i64, _offset: i64) -> RepoFuture<Vec<String>> {
        debug!("Searching in elastic {:?}.", name);
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
                .and_then(|res| {
                    debug!("Result of searching in elastic {:?}.", res);
                    future::ok(res.suggested_texts())
                }),
        )
    }
}
