use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::select;
use diesel::dsl::exists;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;

use models::{ProdAttr, NewProdAttr, UpdateProdAttr};
use models::attribute_product::prod_attr_values::dsl::*;
use super::error::Error;
use super::types::{DbConnection, RepoResult};
use repos::acl::Acl;
use models::authorization::*;

/// ProductAttrs repository, responsible for handling prod_attr_values
pub struct ProductAttrsRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: Box<Acl>,
}

pub trait ProductAttrsRepo {
    /// Find specific product_attribute by product_id
    fn find(&mut self, product_id: i32) -> RepoResult<Vec<ProdAttr>>;

    /// Creates new product_attribute
    fn create(&mut self, payload: NewProdAttr) -> RepoResult<ProdAttr>;

    /// Updates specific product_attribute
    fn update(&mut self, payload: UpdateProdAttr) -> RepoResult<ProdAttr>;
}

impl<'a> ProductAttrsRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl: Box<Acl>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<T: Send + 'static, U: LoadQuery<PgConnection, T> + Send + 'static>(&self, query: U) -> RepoResult<T> {
        query
            .get_result::<T>(&**self.db_conn)
            .map_err(|e| Error::from(e))
    }
}

impl<'a> ProductAttrsRepo for ProductAttrsRepoImpl<'a> {
    /// Find specific product_attribute by ID
    fn find(&mut self, product_id_arg: i32) -> RepoResult<Vec<ProdAttr>>{
        let query = prod_attr_values
            .filter(prod_id.eq(product_id_arg))
            .order(id);

        query
            .get_results(&**self.db_conn)
            .map_err(|e| Error::from(e))
            .and_then(|products_res: Vec<ProdAttr>| {
                let resources = products_res
                    .iter()
                    .map(|product| (product as &WithScope))
                    .collect();
                acl!(
                    resources,
                    self.acl,
                    Resource::Products,
                    Action::Read,
                    Some(self.db_conn)
                ).and_then(|_| Ok(products_res.clone()))
            })
    }

    /// Creates new product_attribute
    fn create(&mut self, payload: NewProdAttr) -> RepoResult<ProdAttr> {
        acl!(
            [payload],
            self.acl,
            Resource::Products,
            Action::Create,
            Some(self.db_conn)
        ).and_then(|_| {
            let query_product_attribute = diesel::insert_into(prod_attr_values).values(&payload);
            query_product_attribute
                .get_result::<Product>(&**self.db_conn)
                .map_err(Error::from)
        })
    }

    /// Updates specific product_attribute
    fn update(&mut self, payload: UpdateProdAttr) -> RepoResult<ProdAttr> {
        self.execute_query(prod_attr_values.find(product_attribute_id_arg))
            .and_then(|product_attribute: Product| {
                acl!(
                    [product_attribute],
                    self.acl,
                    Resource::Products,
                    Action::Update,
                    Some(self.db_conn)
                )
            })
            .and_then(|_| {
                let filter = prod_attr_values
                    .filter(id.eq(product_attribute_id_arg))
                    .filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query
                    .get_result::<Product>(&**self.db_conn)
                    .map_err(|e| Error::from(e))
            })
    }
}
