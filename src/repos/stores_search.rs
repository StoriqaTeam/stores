//! StoresSearch repo, presents CRUD operations with db for users
use std::convert::From;
use hyper::Method;

use future;
use futures::Future;
use serde_json;
use elastic_requests::{CreateRequest, UpdateRequest, SearchRequest};

use models::{Store};
use super::error::Error;
use super::types::{RepoFuture};
use http::client::ClientHandle;


/// StoresSearch repository, responsible for handling stores
pub struct StoresSearchRepoImpl {
    pub client_handle: ClientHandle,
    pub elastic_address: String
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
    pub fn new( client_handle: ClientHandle, elastic_address: String) -> Self {
        Self { client_handle, elastic_address }
    }
}

impl StoresSearchRepo for StoresSearchRepoImpl {
    /// Find specific store by ID
    fn find_by_name(&mut self, name: String) -> RepoFuture<Store> {
        let req = SearchRequest::for_index_ty(
            "store",
            "store",
            format!("{{'query': {{ 'match': {{ 'name': '{}' }} }} }}", name)
        );
        let url = format!("http:://{}{}", self.elastic_address, *req.url);
        let res = self.client_handle.request::<Store>(Method::Get, url, None, None).map_err(Error::from);
        Box::new(res)
    }

    /// Creates new store
    fn create(&mut self, store: Store) -> RepoFuture<()> {
        let body = serde_json::to_string(&store).unwrap();
        let req = CreateRequest::for_index_ty_id("store", "store", store.id, body.clone());
        let url = format!("http:://{}{}", self.elastic_address, *req.url);
        let res = self.client_handle.request::<String>(Method::Post, url, Some(body), None).map_err(Error::from);
        Box::new(res.and_then(|_| future::ok(())))
    }

    /// Updates specific store
    fn update(&mut self, store: Store) -> RepoFuture<()> {
       let body = serde_json::to_string(&store).unwrap();
        let req = UpdateRequest::for_index_ty_id("store", "store", store.id, body.clone());
        let url = format!("http:://{}{}", self.elastic_address, *req.url);
        let res = self.client_handle.request::<String>(Method::Post, url, Some(body), None).map_err(Error::from);
        Box::new(res.and_then(|_| future::ok(())))
    }

}

