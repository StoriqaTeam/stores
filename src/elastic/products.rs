//! ProductsSearch repo, presents CRUD operations with db for users
use errors::Error;
use failure::Fail;
use futures::Future;
use hyper::header::{ContentLength, ContentType, Headers};
use hyper::Method;
use serde_json;

use stq_http::client::ClientHandle;
use stq_static_resources::ModerationStatus;
use stq_types::ProductId;

use super::{log_elastic_req, log_elastic_resp};
use models::*;
use repos::types::RepoFuture;

/// ProductsSearch repository, responsible for handling products
pub struct ProductsElasticImpl {
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

pub trait ProductsElastic {
    /// Find specific product by name limited by `count` parameters
    fn auto_complete(&self, name: AutoCompleteProductName, count: i32, offset: i32) -> RepoFuture<Vec<String>>;

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

    /// Find count
    fn count(&self, prod: SearchProductsByName) -> RepoFuture<i32>;
}

impl ProductsElasticImpl {
    pub fn new(client_handle: ClientHandle, elastic_address: String) -> Self {
        Self {
            client_handle,
            elastic_address,
        }
    }

    fn create_products_from_search_response(res: SearchResponse<ElasticProduct>) -> Vec<ElasticProduct> {
        let mut prods = vec![];
        for hit in res.into_hits() {
            let ids = {
                hit.inner_hits().clone().and_then(|inner_hits| {
                    inner_hits.get("variants").and_then(|variants| {
                        variants["hits"]["hits"].as_array().and_then(|hits_inside_inner_hits| {
                            let mut variant_ids = vec![];
                            for hit_inside_inner_hits in hits_inside_inner_hits {
                                let ids = hit_inside_inner_hits["fields"]["variants.prod_id"].as_array();
                                if let Some(ids) = ids {
                                    for id in ids {
                                        if let Some(id) = id.as_i64() {
                                            variant_ids.push(ProductId(id as i32));
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
        prods
    }

    fn create_variants_map_filters(options: &Option<ProductsSearchOptions>) -> serde_json::Map<String, serde_json::Value> {
        let mut variants_map = serde_json::Map::<String, serde_json::Value>::new();
        let mut variants_must: Vec<serde_json::Value> = vec![];
        let (attr_filters, price_filters, currency_map) = if let Some(options) = options.clone() {
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
                            let lower_case_values = equal.values.into_iter().map(|val| val.to_lowercase()).collect::<Vec<String>>();
                            json!({ "bool" : {"must": [{"term": {"variants.attrs.attr_id": attr.id}},{"terms": {"variants.attrs.str_val": lower_case_values}}]}})
                        } else {
                            json!({})
                        }
                    })
                    .collect::<Vec<serde_json::Value>>()
            });
            (attr_filters, options.price_filter, options.currency_map)
        } else {
            (None, None, None)
        };

        let variant_attr_filter = json!({
            "nested":{  
                "path":"variants.attrs",
                "query":{  
                    "bool":{  
                        "should": attr_filters
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
            if let Some(currency_map) = currency_map {
                let variant_price_filter = json!({
                    "script" : {
                        "script" : {
                            "source" : r###"
                                def cur = doc['variants.currency'].value;
                                def koef = params.cur_map[cur];
                                def price = doc['variants.price'].value * koef;
                                return (params.min == null || price >= params.min) && (params.max == null || price <= params.max);
                            "###,
                            "lang"   : "painless",
                            "params" : {
                                "cur_map" : currency_map,
                                "min" : price_filters.min_value,
                                "max" : price_filters.max_value
                            }
                        }
                    }
                });
                variants_must.push(variant_price_filter);
            } else {
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
        }

        let mut variants_filters: Vec<serde_json::Value> = vec![];
        let variant_exists = json!({
                "exists":{  
                    "field":"variants"
                }
        });
        variants_filters.push(variant_exists);

        if let Some(options) = options.clone() {
            if let Some(sort_by) = options.sort_by {
                if sort_by == ProductsSorting::Discount {
                    let variant_discount_exists = json!({
                        "exists": {
                            "field": "variants.discount"
                        }
                    });
                    variants_filters.push(variant_discount_exists);
                }
            }
        }

        if !variants_must.is_empty() {
            variants_map.insert("must".to_string(), serde_json::Value::Array(variants_must));
        }
        if !variants_filters.is_empty() {
            variants_map.insert("filter".to_string(), serde_json::Value::Array(variants_filters));
        }
        variants_map
    }

    fn create_category_filter(options: Option<ProductsSearchOptions>) -> Option<serde_json::Value> {
        options.and_then(|o| o.categories_ids).map(|ids| {
            json!({
                "terms": {"category_id": ids}
            })
        })
    }

    fn create_store_filter(options: Option<ProductsSearchOptions>) -> Option<serde_json::Value> {
        options.and_then(|o| o.store_id).map(|id| {
            json!({
                "term": {"store_id": id}
            })
        })
    }

    fn create_status_filter(options: Option<ProductsSearchOptions>) -> Option<serde_json::Value> {
        options.and_then(|o| o.status).map(|status| {
            json!({
                "term": {"status": status.to_string()}
            })
        })
    }

    fn create_sorting(options: Option<ProductsSearchOptions>) -> Vec<serde_json::Value> {
        let mut sorting: Vec<serde_json::Value> = vec![];
        if let Some(options) = options {
            if let Some(sort_by) = options.sort_by {
                let sort = match sort_by {
                    ProductsSorting::PriceAsc => json!(
                        {
                            "variants.price" : {
                                "mode" :  "min",
                                "order" : "asc",
                                "nested": {
                                    "path": "variants"
                                }
                            }
                        }
                    ),
                    ProductsSorting::PriceDesc => json!({
                            "variants.price" : {
                                "mode" :  "max",
                                "order" : "desc",
                                "nested": {
                                    "path": "variants"
                                }
                            }
                        }),
                    ProductsSorting::Views => json!({ "views" : { "order" : "desc"} }),
                    ProductsSorting::Discount => json!({
                            "variants.discount" : {
                                "mode" :  "max",
                                "order" : "desc",
                                "nested": {
                                    "path": "variants"
                                }
                            }
                        }),
                };
                sorting.push(sort);
            }
        }
        sorting
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

        let mut filters: Vec<serde_json::Value> = vec![];
        let variants_map = ProductsElasticImpl::create_variants_map_filters(&prod.options);

        let sorting_in_variants = prod
            .options
            .clone()
            .and_then(|options| options.sort_by)
            .map(|sort_by| match sort_by {
                ProductsSorting::PriceAsc => json!(
                        [{"variants.price" : "asc"}]
                    ),
                ProductsSorting::PriceDesc => json!(
                        [{"variants.price" : "desc"}]
                        ),
                ProductsSorting::Views => json!([]),
                ProductsSorting::Discount => json!(
                        [{"variants.discount" : "desc"}]
                    ),
            })
            .unwrap_or_else(|| serde_json::Value::Array(vec![]));

        let variants = json!({
            "nested":{  
                "path":"variants",
                "query":{  
                    "bool": variants_map
                },
                "inner_hits": {
                    "_source" : false,
                    "docvalue_fields" : ["variants.prod_id"],
                    "sort" : sorting_in_variants
                }
            }
        });
        filters.push(variants);

        let categories_filter = ProductsElasticImpl::create_category_filter(prod.options.clone());
        if let Some(categories_filter) = categories_filter {
            filters.push(categories_filter);
        }

        let store_filter = ProductsElasticImpl::create_store_filter(prod.options.clone());
        if let Some(store_filter) = store_filter {
            filters.push(store_filter);
        }

        let status_filter = ProductsElasticImpl::create_status_filter(prod.options.clone());
        if let Some(status_filter) = status_filter {
            filters.push(status_filter);
        }

        query_map.insert("filter".to_string(), serde_json::Value::Array(filters));

        let sorting = ProductsElasticImpl::create_sorting(prod.options.clone());

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : query_map
            },
            "sort" : sorting
        }).to_string();

        let url = format!("http://{}/{}/_search", self.elastic_address, ElasticIndex::Product);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        trace!("search_by_name query = '{}'", query);
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticProduct>>(Method::Post, url, Some(query), Some(headers))
                .inspect(|ref res| log_elastic_resp(res))
                .map(ProductsElasticImpl::create_products_from_search_response)
                .map_err(move |e| {
                    e.context(format!(
                        "Search product by name error occured. Prod: {:?}, count: {:?}, offset: {:?}",
                        prod, count, offset
                    )).context(Error::ElasticSearch)
                        .into()
                }),
        )
    }

    /// Find product by views limited by `count` and `offset` parameters
    fn search_most_viewed(&self, prod: MostViewedProducts, count: i32, offset: i32) -> RepoFuture<Vec<ElasticProduct>> {
        log_elastic_req(&prod);

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();

        let mut filters: Vec<serde_json::Value> = vec![];
        let variants_map = ProductsElasticImpl::create_variants_map_filters(&prod.options);
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

        let categories_filter = ProductsElasticImpl::create_category_filter(prod.options.clone());
        if let Some(categories_filter) = categories_filter {
            filters.push(categories_filter);
        }

        let store_filter = ProductsElasticImpl::create_store_filter(prod.options.clone());
        if let Some(store_filter) = store_filter {
            filters.push(store_filter);
        }

        let status_filter = ProductsElasticImpl::create_status_filter(prod.options.clone());
        if let Some(status_filter) = status_filter {
            filters.push(status_filter);
        }

        query_map.insert("filter".to_string(), serde_json::Value::Array(filters));

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : query_map
            },
            "sort" : [{ "views" : { "order" : "desc"} }]
        }).to_string();

        let url = format!("http://{}/{}/_search", self.elastic_address, ElasticIndex::Product);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        trace!("search_most_viewed query = '{}'", query);
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticProduct>>(Method::Post, url, Some(query), Some(headers))
                .inspect(|ref res| log_elastic_resp(res))
                .map(ProductsElasticImpl::create_products_from_search_response)
                .map_err(move |e| {
                    e.context(format!(
                        "Search most viewed product error occured. Prod: {:?}, count: {:?}, offset: {:?}",
                        prod, count, offset
                    )).context(Error::ElasticSearch)
                        .into()
                }),
        )
    }

    /// Find product by dicount pattern limited by `count` and `offset` parameters
    fn search_most_discount(&self, prod: MostDiscountProducts, count: i32, offset: i32) -> RepoFuture<Vec<ElasticProduct>> {
        log_elastic_req(&prod);

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();

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

        let mut filters: Vec<serde_json::Value> = vec![];
        let variants = json!({
            "nested":{  
                "path":"variants",
                "query":{  
                    "bool": {
                        "filter": {
                            "exists": {
                                "field": "variants.discount"
                            }
                        }
                    }
                },
                "inner_hits": {
                    "_source" : false,
                    "docvalue_fields" : ["variants.prod_id"]
                }
            }
        });
        filters.push(variants);

        let categories_filter = ProductsElasticImpl::create_category_filter(prod.options.clone());
        if let Some(categories_filter) = categories_filter {
            filters.push(categories_filter);
        }

        let store_filter = ProductsElasticImpl::create_store_filter(prod.options.clone());
        if let Some(store_filter) = store_filter {
            filters.push(store_filter);
        }

        let status_filter = ProductsElasticImpl::create_status_filter(prod.options.clone());
        if let Some(status_filter) = status_filter {
            filters.push(status_filter);
        }

        query_map.insert("filter".to_string(), serde_json::Value::Array(filters));

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : query_map
            },
            "sort" : [{ 
                "variants.discount" : {
                    "mode" :  "max",
                    "order" : "desc",
                    "nested": {
                        "path": "variants"
                    }
                }
            }]
        }).to_string();

        let url = format!("http://{}/{}/_search", self.elastic_address, ElasticIndex::Product);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        trace!("search_most_discount query = '{}'", query);
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticProduct>>(Method::Post, url, Some(query), Some(headers))
                .inspect(|ref res| log_elastic_resp(res))
                .map(ProductsElasticImpl::create_products_from_search_response)
                .map_err(move |e| {
                    e.context(format!(
                        "Search most discount product error occured. Prod: {:?}, count: {:?}, offset: {:?}",
                        prod, count, offset
                    )).context(Error::ElasticSearch)
                        .into()
                }),
        )
    }

    fn auto_complete(&self, name: AutoCompleteProductName, count: i32, _offset: i32) -> RepoFuture<Vec<String>> {
        log_elastic_req(&name);

        let store = if let Some(store_id) = name.store_id {
            if let Some(status) = name.status {
                json!([format!("{}_{}", store_id, status)]) // workaround because elastic doesn't afford to ANY contexts
            } else {
                let statuses: Vec<String> = ModerationStatus::enum_iter().map(|m| format!("{}_{}", store_id, m)).collect();
                json!(statuses)
            }
        } else {
            if let Some(status) = name.status {
                json!([status])
            } else {
                json!([])
            }
        };

        let suggest = json!({
            "name-suggest" : {
                "prefix" : name.name,
                "completion" : {
                    "field" : "suggest_2",
                    "size" : count,
                    "skip_duplicates": true, 
                    "fuzzy": true,
                    "contexts": {
                        "store_and_status": store 
                    }
                }
            }
        });

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();
        query_map.insert("_source".to_string(), serde_json::Value::Bool(false));
        query_map.insert("suggest".to_string(), suggest);
        let query = serde_json::Value::Object(query_map).to_string();
        trace!("auto_complete query = '{}'", query);
        let url = format!("http://{}/{}/_search", self.elastic_address, ElasticIndex::Product);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticProduct>>(Method::Post, url, Some(query), Some(headers))
                .inspect(|ref res| log_elastic_resp(res))
                .map(|res| res.suggested_texts())
                .map_err(move |e| {
                    e.context(format!(
                        "Auto complete product name error occured. Name: {:?}, count: {}, offset: {}",
                        name, count, _offset
                    )).context(Error::ElasticSearch)
                        .into()
                }),
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

        let mut filters: Vec<serde_json::Value> = vec![];
        filters.push(json!({ "term": {"status": "published"}}));
        query_map.insert("filter".to_string(), serde_json::Value::Array(filters));

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

        let url = format!("http://{}/{}/_search", self.elastic_address, ElasticIndex::Product);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        trace!("aggregate_categories query = '{}'", query);
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticProduct>>(Method::Post, url, Some(query), Some(headers))
                .inspect(|ref res| log_elastic_resp(res))
                .map(|res| {
                    let mut cats = vec![];
                    for ag in res.aggs() {
                        if let Some(my_agg) = ag.get("my_agg") {
                            if let Some(cat) = my_agg.as_i64() {
                                cats.push(cat as i32);
                            }
                        }
                    }
                    cats
                })
                .map_err(move |e| {
                    e.context(format!("Aggregate categories for products error occured. Name: {:?}", name))
                        .context(Error::ElasticSearch)
                        .into()
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

        let mut filters: Vec<serde_json::Value> = vec![];

        if let Some(prod_options) = prod.options.clone() {
            if let Some(prod_options_category_id) = prod_options.categories_ids {
                let category = json!({
                    "terms": {"category_id": prod_options_category_id}
                });
                filters.push(category);
            }
        }

        let status_filter = ProductsElasticImpl::create_status_filter(prod.options.clone());
        if let Some(status_filter) = status_filter {
            filters.push(status_filter);
        }

        query_map.insert("filter".to_string(), serde_json::Value::Array(filters));

        let currency_map = prod.options.clone().and_then(|o| o.currency_map);

        let query = if let Some(currency_map) = currency_map {
            json!({
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
                            "min_price" : { 
                                "min" : { 
                                "script": {
                                            "lang": "painless",
                                            "params": { "cur_map": currency_map },
                                            "source": r###"
                                                def cur = doc['variants.currency.keyword'].value;
                                                def koef = params.cur_map[cur];
                                                return doc['variants.price'].value * koef;
                                            "###,
                                        }
                                    }
                            },
                            "max_price" : { 
                                "max" : { 
                                "script": {
                                            "lang": "painless",
                                            "params": { "cur_map": currency_map },
                                            "source": r###"
                                                def cur = doc['variants.currency.keyword'].value;
                                                def koef = params.cur_map[cur];
                                                return doc['variants.price'].value * koef;
                                            "###,
                                        }
                                    }
                                }
                            }
                        }
                    }
            }).to_string()
        } else {
            json!({
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
            }).to_string()
        };

        let url = format!("http://{}/{}/_search", self.elastic_address, ElasticIndex::Product);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        trace!("aggregate_price query = '{}'", query);
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticProduct>>(Method::Post, url, Some(query), Some(headers))
                .inspect(|ref res| log_elastic_resp(res))
                .map(|res| {
                    let mut price_filters = RangeFilter::default();
                    if let Some(aggs_raw) = res.aggs_raw() {
                        if let Some(max_price) = aggs_raw["variants"]["max_price"]["value"].as_f64() {
                            price_filters.add_value(max_price);
                        };
                        if let Some(min_price) = aggs_raw["variants"]["min_price"]["value"].as_f64() {
                            price_filters.add_value(min_price);
                        };
                    }
                    price_filters
                })
                .map_err(move |e| {
                    e.context(format!("Aggregate price name error occured. Prod: {:?}", prod))
                        .context(Error::ElasticSearch)
                        .into()
                }),
        )
    }

    fn count(&self, prod: SearchProductsByName) -> RepoFuture<i32> {
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

        let mut filters: Vec<serde_json::Value> = vec![];

        let store_filter = ProductsElasticImpl::create_store_filter(prod.options.clone());
        if let Some(store_filter) = store_filter {
            filters.push(store_filter);
        }

        if let Some(prod_options) = prod.options.clone() {
            if let Some(prod_options_category_id) = prod_options.categories_ids {
                let category = json!({
                    "terms": {"category_id": prod_options_category_id}
                });
                filters.push(category);
            }
        }

        let status_filter = ProductsElasticImpl::create_status_filter(prod.options.clone());
        if let Some(status_filter) = status_filter {
            filters.push(status_filter);
        }

        query_map.insert("filter".to_string(), serde_json::Value::Array(filters));

        let query = json!({
                "query": {
                        "bool" : query_map
                    },
            }).to_string();

        let url = format!("http://{}/{}/_count", self.elastic_address, ElasticIndex::Product);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        trace!("count query = '{}'", query);
        Box::new(
            self.client_handle
                .request::<CountResponse>(Method::Post, url, Some(query), Some(headers))
                .inspect(|ref res| log_elastic_resp(res))
                .map(|res| res.get_count() as i32)
                .map_err(move |e| {
                    e.context(format!("Search base product count error occured. Base product: {:?}", prod))
                        .context(Error::ElasticSearch)
                        .into()
                }),
        )
    }
}
