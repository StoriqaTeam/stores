//! StoresSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::Method;
use future;
use futures::Future;
use future::IntoFuture;
use serde_json;
use elastic_requests::{CreateRequest, SearchRequest, UpdateRequest};
use elastic_responses::{IndexResponse, SearchResponse, UpdateResponse};

use models::Store;
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
        let req = SearchRequest::for_index_ty(
            "store",
            "store",
            format!("{{'query': {{ 'match': {{ 'name': '{}' }} }} }}", name),
        );
        let url = format!("http:://{}{}", self.elastic_address, *req.url);
        Box::new(
            self.client_handle
                .request::<SearchResponse<Store>>(Method::Get, url, None, None)
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
        let req = CreateRequest::for_index_ty_id("store", "store", store.id, body.clone());
        let url = format!("http:://{}{}", self.elastic_address, *req.url);
        Box::new(
            self.client_handle
                .request::<IndexResponse>(Method::Post, url, Some(body), None)
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
        let body = serde_json::to_string(&store).unwrap();
        let req = UpdateRequest::for_index_ty_id("store", "store", store.id, body.clone());
        let url = format!("http:://{}{}", self.elastic_address, *req.url);
        Box::new(
            self.client_handle
                .request::<UpdateResponse>(Method::Post, url, Some(body), None)
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
