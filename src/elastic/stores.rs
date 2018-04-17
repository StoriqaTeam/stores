//! StoresSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentLength, ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use serde_json;
use stq_http::client::ClientHandle;

use models::{CountResponse, ElasticIndex, ElasticStore, SearchResponse, SearchStore, StoresSearchOptions};
use repos::error::RepoError as Error;
use repos::types::RepoFuture;
use super::{log_elastic_req, log_elastic_resp};

/// StoresSearch repository, responsible for handling stores
pub struct StoresElasticImpl {
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

pub trait StoresElastic {
    /// Find specific store by name limited by `count` parameters
    fn find_by_name(&self, search_store: SearchStore, count: i32, offset: i32) -> RepoFuture<Vec<ElasticStore>>;
    /// Search count of stores by name
    fn search_count(&self, search_store: SearchStore) -> RepoFuture<i32>;
    /// Aggregate countries
    fn aggregate_countries(&self, search_store: SearchStore) -> RepoFuture<Vec<String>>;
    /// Aggregate categories
    fn aggregate_categories(&self, search_store: SearchStore) -> RepoFuture<Vec<i32>>;
    /// Auto complete
    fn auto_complete(&self, name: String, count: i32, offset: i32) -> RepoFuture<Vec<String>>;
}

impl StoresElasticImpl {
    pub fn new(client_handle: ClientHandle, elastic_address: String) -> Self {
        Self {
            client_handle,
            elastic_address,
        }
    }

    fn create_elastic_filters(options: Option<StoresSearchOptions>) -> Vec<serde_json::Value> {
        let mut filters: Vec<serde_json::Value> = vec![];
        let (category_id, country) = if let Some(options) = options {
            (options.category_id, options.country)
        } else {
            (None, None)
        };

        if let Some(country_name) = country {
            let country = json!({
                "term": {"country": country_name}
            });
            filters.push(country);
        }

        if let Some(id) = category_id {
            let category = json!({
                "nested" : {
                    "path" : "product_categories",
                    "query" : { "bool" : {"must": { "term": { "product_categories.category_id": id}}}}
                    }
            });
            filters.push(category);
        }

        filters
    }
}

impl StoresElastic for StoresElasticImpl {
    /// Find specific stores by name limited by `count` parameters
    fn find_by_name(&self, search_store: SearchStore, count: i32, offset: i32) -> RepoFuture<Vec<ElasticStore>> {
        log_elastic_req(&search_store);

        let name_query = json!({
            "nested" : {
                    "path" : "name",
                    "query" : {
                        "match": {
                            "name.text" : search_store.name
                        }
                    }
                }
        });

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();

        if !search_store.name.is_empty() {
            query_map.insert("must".to_string(), name_query);
        }

        let filters = StoresElasticImpl::create_elastic_filters(search_store.options);
        if !filters.is_empty() {
            query_map.insert("filter".to_string(), serde_json::Value::Array(filters));
        }

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : query_map
            },
            "sort" : [
                { "rating" : { "order" : "desc"} },
                { "product_categories" : {"missing" : "_last"} }
            ]
        }).to_string();

        let url = format!(
            "http://{}/{}/_search",
            self.elastic_address,
            ElasticIndex::Store
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));

        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticStore>>(Method::Post, url, Some(query), Some(headers))
                .map_err(Error::from)
                .inspect(|ref res| log_elastic_resp(res))
                .and_then(|res| future::ok(res.into_documents().collect::<Vec<ElasticStore>>())),
        )
    }

    /// Auto Complete
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
            ElasticIndex::Store
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticStore>>(Method::Post, url, Some(query), Some(headers))
                .map_err(Error::from)
                .inspect(|ref res| log_elastic_resp(res))
                .and_then(|res| future::ok(res.suggested_texts())),
        )
    }

    /// Search count of stores by name
    fn search_count(&self, search_store: SearchStore) -> RepoFuture<i32> {
        log_elastic_req(&search_store);
        let name_query = json!({
            "nested" : {
                    "path" : "name",
                    "query" : {
                        "match": {
                            "name.text" : search_store.name
                        }
                    }
                }
        });

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();

        if !search_store.name.is_empty() {
            query_map.insert("must".to_string(), name_query);
        }

        let query = json!({
            "query": {
                "bool" : query_map
            }
        }).to_string();

        let url = format!(
            "http://{}/{}/_count",
            self.elastic_address,
            ElasticIndex::Store
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        Box::new(
            self.client_handle
                .request::<CountResponse>(Method::Post, url, Some(query), Some(headers))
                .map_err(Error::from)
                .inspect(|ref res| log_elastic_resp(res))
                .and_then(|res| future::ok(res.get_count() as i32)),
        )
    }

    /// Aggregate countries
    fn aggregate_countries(&self, search_store: SearchStore) -> RepoFuture<Vec<String>> {
        log_elastic_req(&search_store);
        let name_query = json!({
            "nested" : {
                    "path" : "name",
                    "query" : {
                        "match": {
                            "name.text" : search_store.name
                        }
                    }
                }
        });

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();

        if !search_store.name.is_empty() {
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
                    "field": "country"
                }
            }
        }
        }).to_string();

        let url = format!(
            "http://{}/{}/_search",
            self.elastic_address,
            ElasticIndex::Store
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticStore>>(Method::Post, url, Some(query), Some(headers))
                .map_err(Error::from)
                .inspect(|ref res| log_elastic_resp(res))
                .and_then(|res| {
                    let mut countries = vec![];
                    for ag in res.aggs() {
                        if let Some(my_agg) = ag.get("my_agg") {
                            if let Some(country) = my_agg.as_str() {
                                countries.push(country.to_string());
                            }
                        }
                    }
                    future::ok(countries)
                }),
        )
    }

    /// Aggregate categories
    fn aggregate_categories(&self, search_store: SearchStore) -> RepoFuture<Vec<i32>> {
        log_elastic_req(&search_store);
        let name_query = json!({
            "nested" : {
                    "path" : "name",
                    "query" : {
                        "match": {
                            "name.text" : search_store.name
                        }
                    }
                }
        });

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();

        if !search_store.name.is_empty() {
            query_map.insert("must".to_string(), name_query);
        }

        let query = json!({
        "size": 0,
        "query": {
                "bool" : query_map
            },
        "aggregations": {
            "product_categories" : {
                "nested" : {
                    "path" : "product_categories"
                },
                "aggs" : {
                    "category" : { "terms" : { "field" : "product_categories.category_id" } },
                }
            }
        }
        }).to_string();

        let url = format!(
            "http://{}/{}/_search",
            self.elastic_address,
            ElasticIndex::Store
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticStore>>(Method::Post, url, Some(query), Some(headers))
                .map_err(Error::from)
                .inspect(|ref res| log_elastic_resp(res))
                .and_then(|res| {
                    let mut categories_ids = vec![];
                    if let Some(aggs_raw) = res.aggs_raw() {
                        if let Some(id) = aggs_raw["product_categories"]["category"]["value"].as_i64() {
                            categories_ids.push(id as i32);
                        };
                    }
                    future::ok(categories_ids)
                }),
        )
    }
}
