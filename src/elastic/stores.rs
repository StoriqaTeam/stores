//! StoresSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentLength, ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use serde_json;
use stq_http::client::ClientHandle;

use models::{CountResponse, ElasticIndex, ElasticStore, SearchResponse, SearchStore};
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

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : query_map
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
}
