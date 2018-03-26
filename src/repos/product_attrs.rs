use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use stq_acl::*;

use models::{NewProdAttr, ProdAttr, UpdateProdAttr};
use models::attribute_product::prod_attr_values::dsl::*;
use repos::error::RepoError as Error;
use super::types::{DbConnection, RepoResult};
use models::authorization::*;
use super::acl;
use super::acl::BoxedAcl;

/// ProductAttrs repository, responsible for handling prod_attr_values
pub struct ProductAttrsRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: BoxedAcl,
}

pub trait ProductAttrsRepo {
    /// Find product attributes by product ID
    fn find_all_attributes(&self, product_id_arg: i32) -> RepoResult<Vec<ProdAttr>>;

    /// Creates new product_attribute
    fn create(&self, payload: NewProdAttr) -> RepoResult<ProdAttr>;

    /// Updates specific product_attribute
    fn update(&self, payload: UpdateProdAttr) -> RepoResult<ProdAttr>;
}

impl<'a> ProductAttrsRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl: BoxedAcl) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a> ProductAttrsRepo for ProductAttrsRepoImpl<'a> {
    /// Find specific product_attributes by product ID
    fn find_all_attributes(&self, product_id_arg: i32) -> RepoResult<Vec<ProdAttr>> {
        debug!("Find all attributes of product id {}.", product_id_arg);
        let query = prod_attr_values
            .filter(prod_id.eq(product_id_arg))
            .order(id);

        query
            .get_results(&**self.db_conn)
            .map_err(Error::from)
            .and_then(|prod_attrs_res: Vec<ProdAttr>| {
                let resources = prod_attrs_res
                    .iter()
                    .map(|prod_attr| (prod_attr as &WithScope<Scope>))
                    .collect::<Vec<&WithScope<Scope>>>();
                acl::check(
                    &*self.acl,
                    &Resource::ProductAttrs,
                    &Action::Read,
                    &resources,
                    Some(self.db_conn),
                ).and_then(|_| Ok(prod_attrs_res.clone()))
            })
    }

    /// Creates new product_attribute
    fn create(&self, payload: NewProdAttr) -> RepoResult<ProdAttr> {
        debug!("Create new product attribute {:?}.", payload);
        acl::check(
            &*self.acl,
            &Resource::ProductAttrs,
            &Action::Create,
            &[&payload],
            Some(self.db_conn),
        ).and_then(|_| {
            let query_product_attribute = diesel::insert_into(prod_attr_values).values(&payload);
            query_product_attribute
                .get_result::<ProdAttr>(&**self.db_conn)
                .map_err(Error::from)
        })
    }

    fn update(&self, payload: UpdateProdAttr) -> RepoResult<ProdAttr> {
        debug!("Updating product attribute with payload {:?}.", payload);
        let query = prod_attr_values
            .filter(prod_id.eq(payload.prod_id))
            .filter(attr_id.eq(payload.attr_id));

        query
            .first::<ProdAttr>(&**self.db_conn)
            .map_err(Error::from)
            .and_then(|prod_attr: ProdAttr| {
                acl::check(
                    &*self.acl,
                    &Resource::ProductAttrs,
                    &Action::Update,
                    &[&prod_attr],
                    Some(self.db_conn),
                )
            })
            .and_then(|_| {
                let filter = prod_attr_values
                    .filter(prod_id.eq(payload.prod_id))
                    .filter(attr_id.eq(payload.attr_id));

                let query = diesel::update(filter).set(&payload);
                query
                    .get_result::<ProdAttr>(&**self.db_conn)
                    .map_err(Error::from)
            })
    }
}
