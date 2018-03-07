use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;

use models::{Category, NewCategory, UpdateCategory};
use models::category::categories::dsl::*;
use repos::error::RepoError as Error;

use super::types::{DbConnection, RepoResult};
use models::authorization::*;
use super::acl;
use super::acl::BoxedAcl;

/// Categories repository, responsible for handling categorie_values
pub struct CategoriesRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: BoxedAcl,
}

pub trait CategoriesRepo {
    /// Find specific category by id
    fn find(&self, id_arg: i32) -> RepoResult<Category>;

    /// Creates new category
    fn create(&self, payload: NewCategory) -> RepoResult<Category>;

    /// Updates specific category
    fn update(&self, category_id_arg: i32, payload: UpdateCategory) -> RepoResult<Category>;
}

impl<'a> CategoriesRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl: BoxedAcl) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a> CategoriesRepo for CategoriesRepoImpl<'a> {
    /// Find specific category by id
    fn find(&self, id_arg: i32) -> RepoResult<Category> {
        let query = categories.filter(id.eq(id_arg));

        query
            .first::<Category>(&**self.db_conn)
            .map_err(Error::from)
            .and_then(|category: Category| {
                acl::check(
                    &*self.acl,
                    &Resource::Categories,
                    &Action::Read,
                    &[],
                    Some(self.db_conn),
                ).and_then(|_| Ok(category))
            })
    }

    /// Creates new category
    fn create(&self, payload: NewCategory) -> RepoResult<Category> {
        acl::check(
            &*self.acl,
            &Resource::Categories,
            &Action::Create,
            &[],
            Some(self.db_conn),
        ).and_then(|_| {
            let query_categorie = diesel::insert_into(categories).values(&payload);
            query_categorie
                .get_result::<Category>(&**self.db_conn)
                .map_err(Error::from)
        })
    }

    /// Updates specific category
    fn update(&self, category_id_arg: i32, payload: UpdateCategory) -> RepoResult<Category> {
        let query = categories.find(category_id_arg);

        query
            .first::<Category>(&**self.db_conn)
            .map_err(Error::from)
            .and_then(|_| {
                acl::check(
                    &*self.acl,
                    &Resource::Categories,
                    &Action::Update,
                    &[],
                    Some(self.db_conn),
                )
            })
            .and_then(|_| {
                let filter = categories.filter(id.eq(category_id_arg));

                let query = diesel::update(filter).set(&payload);
                query
                    .get_result::<Category>(&**self.db_conn)
                    .map_err(Error::from)
            })
    }
}
