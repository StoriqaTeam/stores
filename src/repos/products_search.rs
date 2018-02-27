//! ProductsSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use serde_json;
use elastic_responses::{SearchResponse, UpdateResponse};

use models::{ElasticIndex, ElasticProduct, Filter, IndexResponse, ProdAttr, SearchProduct};
use super::error::Error;
use super::types::RepoFuture;
use http::client::ClientHandle;

/// ProductsSearch repository, responsible for handling products
pub struct ProductsSearchRepoImpl {
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

pub trait ProductsSearchRepo {
    /// Find specific product by name limited by `count` parameters
    fn search(&mut self, prod: SearchProduct, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>>;

    /// Creates new product
    fn create_product(&mut self, product: ElasticProduct) -> RepoFuture<()>;

    /// Creates new product
    fn create_attribute_product_value(&mut self, attr_prod: ProdAttr) -> RepoFuture<()>;

    /// Updates specific product
    fn update(&mut self, product: ElasticProduct) -> RepoFuture<()>;
}

impl ProductsSearchRepoImpl {
    pub fn new(client_handle: ClientHandle, elastic_address: String) -> Self {
        Self {
            client_handle,
            elastic_address,
        }
    }
}

impl ProductsSearchRepo for ProductsSearchRepoImpl {
    /// Find specific products by name limited by `count` parameters
    fn search(&mut self, prod: SearchProduct, count: i64, offset: i64) -> RepoFuture<Vec<ElasticProduct>> {
        let name_query = match prod.name {
            None => json!({
                "match_all": {}
            }),
            Some(name) => json!({
                 "multi_match" : {
                    "query":      name,
                    "fields":     [ "name", "short_description", "long_description" ],
                    "operator":   "or" 
                }
            }),
        };

        let props = match prod.attr_filters {
            None => json!({}),
            Some(filters) => {
                let fil: Vec<serde_json::Value> = filters
                    .into_iter()
                    .map(|attr| {
                        let attribute_name = attr.name.clone();
                        match attr.filter {
                            Filter::Equal(val) => {
                                json!({ "must": [{"term": {"attribute_name": attribute_name}},{"term": {"attribute_value": val}}]})
                            }
                            Filter::Lte(val) => {
                                json!({ "must": [{"term": {"attribute_name": attribute_name}}, { "range": { "attribute_value": {"lte": val }}}]})
                            }
                            Filter::Le(val) => {
                                json!({ "must": [{"term": {"attribute_name": attribute_name}}, { "range": { "attribute_value": {"le": val }}}]})
                            }
                            Filter::Ge(val) => {
                                json!({ "must": [{"term": {"attribute_name": attribute_name}}, { "range": { "attribute_value": {"ge": val }}}]})
                            }
                            Filter::Gte(val) => {
                                json!({ "must": [{"term": {"attribute_name": attribute_name}}, { "range": { "attribute_value": {"gte": val }}}]})
                            }
                        }
                    })
                    .collect();
                json!({
                        "nested" : {
                            "path" : "properties",
                            "filter" : {
                                "bool" : fil
                            }
                        }
                })
            }
        };

        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "bool" : {
                    "must" : name_query,
                    "filter" : props,
                }
            }
        }).to_string();

        let url = format!(
            "http://{}/{}/_doc/_search",
            self.elastic_address,
            ElasticIndex::Product
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticProduct>>(Method::Get, url, Some(query), Some(headers))
                .map_err(Error::from)
                .and_then(|res| future::ok(res.into_documents().collect::<Vec<ElasticProduct>>())),
        )
    }

    /// Creates new attribute product value
    fn create_attribute_product_value(&mut self, new_prod_attr: ProdAttr) -> RepoFuture<()> {
        let body = serde_json::to_string(&new_prod_attr).unwrap();
        let url = format!(
            "http://{}/{}/_doc/{}/_create",
            self.elastic_address,
            ElasticIndex::ProductAttributeValue,
            new_prod_attr.id
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

    /// Creates new product
    fn create_product(&mut self, product: ElasticProduct) -> RepoFuture<()> {
        let body = serde_json::to_string(&product).unwrap();
        let url = format!(
            "http://{}/{}/_doc/{}/_create",
            self.elastic_address,
            ElasticIndex::Product,
            product.id
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

    /// Updates specific product
    fn update(&mut self, product: ElasticProduct) -> RepoFuture<()> {
        let body = json!({
            "doc": product,
        }).to_string();
        let url = format!(
            "http://{}/{}/_doc/{}/_update",
            self.elastic_address,
            ElasticIndex::Product,
            product.id
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
