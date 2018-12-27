//! StoresSearch repo, presents CRUD operations with db for users
use errors::Error;
use failure::Fail;
use futures::Future;
use hyper::header::{ContentLength, ContentType, Headers};
use hyper::Method;
use serde_json;
use stq_http::client::ClientHandle;

use stq_types::CategoryId;

use super::{log_elastic_req, log_elastic_resp};
use models::{CountResponse, ElasticIndex, ElasticStore, SearchResponse, SearchStore, StoresSearchOptions};
use repos::types::RepoFuture;

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
    fn aggregate_categories(&self, search_store: SearchStore) -> RepoFuture<Vec<CategoryId>>;
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
                "term": {"country.keyword": country_name}
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
        let store_name = search_store.name.to_lowercase();
        let name_query = fuzzy_search_by_name_query(&store_name);

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();

        if !store_name.is_empty() {
            query_map.insert("must".to_string(), name_query);
        }

        let mut filters = StoresElasticImpl::create_elastic_filters(search_store.options.clone());
        filters.push(json!({ "term": {"status": "published"}}));
        let product_categories = json!({
            "nested":{
                "path": "product_categories",
                "query": {
                    "bool": {
                        "filter": {
                            "exists": {
                                "field": "product_categories"
                            }
                        }
                    }
                }
            }
        });
        filters.push(product_categories);
        query_map.insert("filter".to_string(), serde_json::Value::Array(filters));

        let query = if store_name.is_empty() {
            json!({
                "from" : offset, "size" : count,
                "query": {
                    "bool" : query_map
                },
                "sort" : [
                    { "rating" : { "order" : "desc"} }
                ]
            }).to_string()
        } else {
            json!({
                "from" : offset, "size" : count,
                "query": {
                    "bool" : query_map
                }
            }).to_string()
        };

        let url = format!("http://{}/{}/_search", self.elastic_address, ElasticIndex::Store);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));

        trace!("find_by_name query = '{}'", query);
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticStore>>(Method::Post, url, Some(query), Some(headers))
                .inspect(|ref res| log_elastic_resp(res))
                .map(|res| res.into_documents().collect::<Vec<ElasticStore>>())
                .map_err(move |e| {
                    e.context(format!(
                        "Search store by name error occurred. Store: {:?}, count: {:?}, offset: {:?}",
                        search_store, count, offset
                    )).context(Error::ElasticSearch)
                    .into()
                }),
        )
    }

    /// Auto Complete
    fn auto_complete(&self, name: String, count: i32, _offset: i32) -> RepoFuture<Vec<String>> {
        log_elastic_req(&name);
        let name = name.to_lowercase();

        let suggest = json!({
            "name-suggest" : {
                "prefix" : name,
                "completion" : {
                    "field" : "suggest",
                    "size" : count,
                    "skip_duplicates": true,
                    "fuzzy": true,
                    "contexts": {
                        "status": "published"
                    }
                }
            }
        });

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();
        query_map.insert("_source".to_string(), serde_json::Value::Bool(false));
        query_map.insert("suggest".to_string(), suggest);
        let query = serde_json::Value::Object(query_map).to_string();

        let url = format!("http://{}/{}/_search", self.elastic_address, ElasticIndex::Store);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        trace!("auto_complete query = '{}'", query);
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticStore>>(Method::Post, url, Some(query), Some(headers))
                .inspect(|ref res| log_elastic_resp(res))
                .map(|res| res.suggested_texts())
                .map_err(move |e| {
                    e.context(format!(
                        "Auto complete store name error occurred. Name: {:?}, count: {:?}, offset: {:?}",
                        name, count, _offset
                    )).context(Error::ElasticSearch)
                    .into()
                }),
        )
    }

    /// Search count of stores by name
    fn search_count(&self, search_store: SearchStore) -> RepoFuture<i32> {
        log_elastic_req(&search_store);
        let store_name = search_store.name.to_lowercase();
        let name_query = fuzzy_search_by_name_query(&store_name);

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();

        if !store_name.is_empty() {
            query_map.insert("must".to_string(), name_query);
        }

        let mut filters: Vec<serde_json::Value> = vec![];
        filters.push(json!({ "term": {"status": "published"}}));
        query_map.insert("filter".to_string(), serde_json::Value::Array(filters));

        let query = json!({
            "query": {
                "bool" : query_map
            }
        }).to_string();

        let url = format!("http://{}/{}/_count", self.elastic_address, ElasticIndex::Store);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        trace!("search_count query = '{}'", query);
        Box::new(
            self.client_handle
                .request::<CountResponse>(Method::Post, url, Some(query), Some(headers))
                .inspect(|ref res| log_elastic_resp(res))
                .map(|res| res.get_count() as i32)
                .map_err(move |e| {
                    e.context(format!("Search store count error occurred. Store: {:?}", search_store))
                        .context(Error::ElasticSearch)
                        .into()
                }),
        )
    }

    /// Aggregate countries
    fn aggregate_countries(&self, search_store: SearchStore) -> RepoFuture<Vec<String>> {
        log_elastic_req(&search_store);
        let store_name = search_store.name.to_lowercase();
        let name_query = fuzzy_search_by_name_query(&store_name);

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();

        if !store_name.is_empty() {
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
                    "field": "country.keyword"
                }
            }
        }
        }).to_string();

        let url = format!("http://{}/{}/_search", self.elastic_address, ElasticIndex::Store);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        trace!("aggregate_countries query = '{}'", query);
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticStore>>(Method::Post, url, Some(query), Some(headers))
                .inspect(|ref res| log_elastic_resp(res))
                .map(|res| {
                    let mut countries = vec![];
                    for ag in res.aggs() {
                        if let Some(my_agg) = ag.get("my_agg") {
                            if let Some(country) = my_agg.as_str() {
                                countries.push(country.to_string());
                            }
                        }
                    }
                    countries
                }).map_err(move |e| {
                    e.context(format!("Aggregate countries for store error occurred. Store: {:?}", search_store))
                        .context(Error::ElasticSearch)
                        .into()
                }),
        )
    }

    /// Aggregate categories
    fn aggregate_categories(&self, search_store: SearchStore) -> RepoFuture<Vec<CategoryId>> {
        log_elastic_req(&search_store);
        let store_name = search_store.name.to_lowercase();
        let name_query = fuzzy_search_by_name_query(&store_name);

        let mut query_map = serde_json::Map::<String, serde_json::Value>::new();

        if !store_name.is_empty() {
            query_map.insert("must".to_string(), name_query);
        }

        let mut filters: Vec<serde_json::Value> = vec![];
        filters.push(json!({ "term": {"status": "published"}}));
        let product_categories = json!({
            "nested":{
                "path": "product_categories",
                "query": {
                    "bool": {
                        "filter": {
                            "exists": {
                                "field": "product_categories"
                            }
                        }
                    }
                }
            }
        });
        filters.push(product_categories);
        query_map.insert("filter".to_string(), serde_json::Value::Array(filters));

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

        let url = format!("http://{}/{}/_search", self.elastic_address, ElasticIndex::Store);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        trace!("aggregate_categories query = '{}'", query);
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticStore>>(Method::Post, url, Some(query), Some(headers))
                .inspect(|ref res| log_elastic_resp(res))
                .map(|res| {
                    let mut categories_ids = vec![];
                    if let Some(aggs_raw) = res.aggs_raw() {
                        if let Some(buckets) = aggs_raw["product_categories"]["category"]["buckets"].as_array() {
                            for bucket in buckets {
                                if let Some(key) = bucket["key"].as_i64() {
                                    categories_ids.push(CategoryId(key as i32));
                                }
                            }
                        }
                    };
                    categories_ids
                }).map_err(move |e| {
                    e.context(format!("Aggregate categories for stores error occurred. Store: {:?}", search_store))
                        .context(Error::ElasticSearch)
                        .into()
                }),
        )
    }
}

fn fuzzy_search_by_name_query(name: &str) -> serde_json::Value {
    json!({
        "nested" : {
                "path" : "name",
                "query" : {
                    "match": {
                        "name.text":{
                            "query":name,
                            "fuzziness":"AUTO"
                        }
                    }
                }
            }
    })
}
