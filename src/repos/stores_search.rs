//! StoresSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use future::IntoFuture;
use serde_json;
use elastic_requests::{CreateRequest, SearchRequest, UpdateRequest};
use elastic_responses::{SearchResponse, UpdateResponse};

use models::{IndexResponse, Store};
use super::error::Error;
use super::types::RepoFuture;
use http::client::ClientHandle;
/// StoresSearch repository, responsible for handling stores
pub struct StoresSearchRepoImpl {
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

pub trait StoresSearchRepo {
    /// Find specific store by ID
    fn find_by_name(&mut self, name: String) -> RepoFuture<Store>;

    /// Creates new store
    fn create(&mut self, store: Store) -> RepoFuture<()>;

    /// Updates specific store
    fn update(&mut self, store: Store) -> RepoFuture<()>;
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
    /// Find specific store by ID
    fn find_by_name(&mut self, name: String) -> RepoFuture<Store> {
        let query = json!({
            "query": {
                "term" : {
                    "name" : name
                }
            }
        }).to_string();
        let req = SearchRequest::for_index_ty(
            "store",
            "_doc",
            query.clone(),
        );
        let url = format!("http://{}{}", self.elastic_address, *req.url);
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        Box::new(
            self.client_handle
                .request::<SearchResponse<Store>>(Method::Get, url, Some(query), Some(headers))
                .map_err(Error::from)
                .and_then(|res| {
                    res.into_documents()
                        .next()
                        .ok_or(Error::NotFound)
                        .into_future()
                }),
        )
    }

    /// Creates new store
    fn create(&mut self, store: Store) -> RepoFuture<()> {
        let body = serde_json::to_string(&store).unwrap();
        let req = CreateRequest::for_index_ty_id("store", "_doc", store.id, body.clone());
        let url = format!("http://{}{}", self.elastic_address, *req.url);
        let mut headers = Headers::new();
        headers.set(ContentType::json());

        Box::new(
            self.client_handle
                .request::<IndexResponse>(Method::Put, url, Some(body), Some(headers))
                .map_err(Error::from)
                .and_then(|res| {
                    if res.created() {
                        future::ok(())
                    } else {
                        future::err(Error::NotFound)
                    }
                }),
        )
    }

    /// Updates specific store
    fn update(&mut self, store: Store) -> RepoFuture<()> {
        let body = json!({
            "doc": store,
        }).to_string();
        let req = UpdateRequest::for_index_ty_id("store", "_doc", store.id, body.clone());
        let url = format!("http://{}{}", self.elastic_address, *req.url);
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
