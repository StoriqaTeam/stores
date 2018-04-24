//! ProductsSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentLength, ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use serde_json;
use stq_http::client::ClientHandle;

use models::*;
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

    /// Find all categories ids where prod exist
    fn aggregate_categories(&self, name: String) -> RepoFuture<Vec<i32>>;

    /// Find price range
    fn aggregate_price(&self, prod: SearchProductsByName) -> RepoFuture<RangeFilter>;
}

impl ProductsElasticImpl {
    pub fn new(client_handle: ClientHandle, elastic_address: String) -> Self {
        Self {
            client_handle,
            elastic_address,
        }
    }

    fn create_elastic_filters(options: Option<ProductsSearchOptions>) -> Vec<serde_json::Value> {
        let mut filters: Vec<serde_json::Value> = vec![];
        let mut variants_map = serde_json::Map::<String, serde_json::Value>::new();
        let mut variants_filters: Vec<serde_json::Value> = vec![];
        let mut variants_must: Vec<serde_json::Value> = vec![];
        let (attr_filters, categories_ids, price_filters) = if let Some(options) = options {
            let attr_filters = options.attr_filters.map(|attrs| {
                attrs
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
                            let lower_case_values = equal
                                .values
                                .into_iter()
                                .map(|val| val.to_lowercase())
                                .collect::<Vec<String>>();
                            json!({ "bool" : {"must": [{"term": {"variants.attrs.attr_id": attr.id}},{"terms": {"variants.attrs.str_val": lower_case_values}}]}})
                        } else {
                            json!({})
                        }
                    })
                    .collect::<Vec<serde_json::Value>>()
            });
            (attr_filters, options.categories_ids, options.price_filter)
        } else {
            (None, None, None)
        };

        let variant_attr_filter = json!({
            "nested":{  
                "path":"variants.attrs",
                "query":{  
                    "bool":{  
                        "should":attr_filters
                    }
                }
            }
        });

        if let Some(attr_filters) = attr_filters {
            if !attr_filters.is_empty() {
                variants_must.push(variant_attr_filter);
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
            let variant_price_filter = json!({
                "range":{  
                    "variants.price":range_map
                }
            });
            variants_must.push(variant_price_filter);
        }

        let variant_exists = json!({
                "exists":{  
                    "field":"variants"
                }
        });
        variants_filters.push(variant_exists);

        variants_map.insert("must".to_string(), serde_json::Value::Array(variants_must));
        variants_map.insert(
            "filter".to_string(),
            serde_json::Value::Array(variants_filters),
        );

        let variants = json!({
            "nested":{  
                "path":"variants",
                "query":{  
                    "bool": variants_map
                },
                "inner_hits": {
                    "_source" : false,
                    "docvalue_fields" : ["variants.prod_id"]
                }
            }
        });

        filters.push(variants);

        if let Some(ids) = categories_ids {
            let category = json!({
                "terms": {"category_id": ids}
            });
            filters.push(category);
        }

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

        let filters = ProductsElasticImpl::create_elastic_filters(prod.options.clone());
        if !filters.is_empty() {
            query_map.insert("filter".to_string(), serde_json::Value::Array(filters));
        }

        let mut sorting: Vec<serde_json::Value> = vec![];
        if let Some(options) = prod.options {
            if let Some(sort_by) = options.sort_by {
                let sort = match sort_by {
                    ProductsSorting::PriceAsc => json!({ "variants.price" : { "order" : "asc", "mode" : "min"} }),
                    ProductsSorting::PriceDesc => json!({ "variants.price" : { "order" : "desc", "mode" : "max"} }),
                    ProductsSorting::Views => json!({ "views" : { "order" : "desc"} }),
                    ProductsSorting::Discount => json!({ "variants.discount" : { "order" : "desc", "missing" : "_last"}}),
                };
                sorting.push(sort);
            }
        }

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : query_map
            },
            "sort" : sorting
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
                .and_then(|res| {
                    let mut prods = vec![];
                    for hit in res.into_hits() {
                        let ids = {
                            hit.inner_hits().clone().and_then(|m| {
                                m.get("variants").and_then(|v| {
                                    v["hits"]["hits"].as_array().and_then(|hh| {
                                        let mut variant_ids = vec![];
                                        for h in hh {
                                            let ids = h["fields"]["variants.prod_id"].as_array();
                                            if let Some(ids) = ids {
                                                for id in ids {
                                                    if let Some(id) = id.as_i64() {
                                                        variant_ids.push(id as i32);
                                                    }
                                                }
                                            }
                                        }
                                        Some(variant_ids)
                                    })
                                })
                            })
                        };

                        let mut prod = hit.into_document();
                        if let Some(mut prod) = prod {
                            prod.matched_variants_ids = ids;
                            prods.push(prod);
                        }
                    }
                    future::ok(prods)
                }),
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

    /// Find all categories ids where prod exist
    fn aggregate_categories(&self, name: String) -> RepoFuture<Vec<i32>> {
        log_elastic_req(&name);
        let name_query = json!({
            "bool" : {
                "should" : [
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
            }
        });

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();
        if !name.is_empty() {
            query_map.insert("must".to_string(), name_query);
        }

        let query = json!({
        "size": 0,
        "query": {
                "bool" : query_map
            },
        "aggregations": {
            "my_agg": {
                "terms": {
                    "field": "category_id"
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
                .and_then(|res| {
                    let mut cats = vec![];
                    for ag in res.aggs() {
                        if let Some(my_agg) = ag.get("my_agg") {
                            if let Some(cat) = my_agg.as_i64() {
                                cats.push(cat as i32);
                            }
                        }
                    }
                    future::ok(cats)
                }),
        )
    }

    fn aggregate_price(&self, prod: SearchProductsByName) -> RepoFuture<RangeFilter> {
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

        if let Some(prod_options) = prod.options {
            if let Some(prod_options_category_id) = prod_options.categories_ids {
                let category = json!({
                    "terms": {"category_id": prod_options_category_id}
                });
                query_map.insert("filter".to_string(), category);
            }
        }

        let query = json!({
        "size": 0,
        "query": {
                "bool" : query_map
            },
        "aggregations": {
            "variants" : {
                "nested" : {
                    "path" : "variants"
                },
                "aggs" : {
                    "min_price" : { "min" : { "field" : "variants.price" } },
                    "max_price" : { "max" : { "field" : "variants.price" } }
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
                .and_then(|res| {
                    let mut price_filters = RangeFilter::default();
                    if let Some(aggs_raw) = res.aggs_raw() {
                        if let Some(max_price) = aggs_raw["variants"]["max_price"]["value"].as_f64() {
                            price_filters.add_value(max_price);
                        };
                        if let Some(min_price) = aggs_raw["variants"]["min_price"]["value"].as_f64() {
                            price_filters.add_value(min_price);
                        };
                    }
                    future::ok(price_filters)
                }),
        )
    }
}
