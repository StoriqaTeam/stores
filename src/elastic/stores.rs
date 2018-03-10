//! StoresSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentType, Headers, ContentLength};
use hyper::Method;
use future;
use futures::Future;
use serde_json;
use elastic_responses::{SearchResponse, UpdateResponse};
use elastic_responses::search::Hit;
use stq_http::client::ClientHandle;
use stq_static_resources::Translation;

use models::{ElasticIndex, ElasticStore, IndexResponse, SearchStore};
use repos::error::RepoError as Error;
use repos::types::RepoFuture;

/// StoresSearch repository, responsible for handling stores
pub struct StoresSearchRepoImpl {
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

pub trait StoresSearchRepo {
    /// Find specific store by name limited by `count` parameters
    fn find_by_name(&self, search_store: SearchStore, count: i64, offset: i64) -> RepoFuture<Vec<ElasticStore>>;

    /// Checks name exists
    fn name_exists(&self, name: Vec<Translation>) -> RepoFuture<bool>;

    /// Creates new store
    fn create(&self, store: ElasticStore) -> RepoFuture<()>;

    /// Updates specific store
    fn update(&self, store: ElasticStore) -> RepoFuture<()>;
}

impl StoresSearchRepoImpl {
    pub fn new(client_handle: ClientHandle, elastic_address: String) -> Self {
        Self {
            client_handle,
            elastic_address,
        }
    }
}

impl StoresSearchRepo for StoresSearchRepoImpl {
    /// Find specific stores by name limited by `count` parameters
    fn find_by_name(&self, search_store: SearchStore, count: i64, offset: i64) -> RepoFuture<Vec<ElasticStore>> {
        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "nested" : {
                    "path" : "name",
                    "query" : {
                        "match": {
                            "name.text" : search_store.name
                        }
                    }
                }
            }
        }).to_string();
        let url = format!(
            "http://{}/{}/_doc/_search",
            self.elastic_address,
            ElasticIndex::Store
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticStore>>(Method::Get, url, Some(query), Some(headers))
                .map_err(Error::from)
                .and_then(|res| future::ok(res.into_documents().collect::<Vec<ElasticStore>>())),
        )
    }

    /// Checks name exists
    fn name_exists(&self, name: Vec<Translation>) -> RepoFuture<bool> {
        let queries = name
            .into_iter()
            .map(|trans| json!({ "bool" : {"must": [{"term": {"name.lang": trans.lang}}, { "term": { "name.text": trans.text}}]}}))
            .collect::<Vec<serde_json::Value>>();

        let query = json!({
                    "query": {
                        "nested" : {
                            "path" : "name",
                            "query": {
                                    "bool": {
                                        "should": queries
                                    }
                            }
                        }
                    }
                }).to_string();

        let url = format!(
            "http://{}/{}/_doc/_search",
            self.elastic_address,
            ElasticIndex::Store
        );
        
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticStore>>(Method::Get, url, Some(query), Some(headers))
                .map_err(Error::from)
                .and_then(|res| {
                    let hits = res.into_hits().into_iter().collect::<Vec<Hit<ElasticStore>>>();
                    future::ok(!hits.is_empty())
                }) 
        )
    }

    /// Creates new store
    fn create(&self, store: ElasticStore) -> RepoFuture<()> {
        let body = serde_json::to_string(&store).unwrap();
        let url = format!(
            "http://{}/{}/_doc/{}/_create",
            self.elastic_address,
            ElasticIndex::Store,
            store.id
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

    /// Updates specific store
    fn update(&self, store: ElasticStore) -> RepoFuture<()> {
        let body = json!({
            "doc": store,
        }).to_string();
        let url = format!(
            "http://{}/{}/_doc/{}/_update",
            self.elastic_address,
            ElasticIndex::Store,
            store.id
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
