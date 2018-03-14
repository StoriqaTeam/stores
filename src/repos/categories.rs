use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;

use models::{Category, NewCategory, RawCategory, UpdateCategory};
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

    /// Returns all categories as a tree
    fn get_all(&self) -> RepoResult<Category>;
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
            .first::<RawCategory>(&**self.db_conn)
            .map_err(Error::from)
            .and_then(|category: RawCategory| {
                acl::check(
                    &*self.acl,
                    &Resource::Categories,
                    &Action::Read,
                    &[],
                    Some(self.db_conn),
                ).and_then(|_| Ok(category))
            })
            .and_then(|found_category| {
                categories
                    .load::<RawCategory>(&**self.db_conn)
                    .map_err(Error::from)
                    .map(|cats| (found_category, cats))
            })
            .and_then(|(found_category, cats)| {
                let id_arg = found_category.id;
                let mut result: Category = found_category.into();
                let children = create_tree(&cats, Some(id_arg));
                result.children = children;
                Ok(result)
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
                .get_result::<RawCategory>(&**self.db_conn)
                .map_err(Error::from)
        })
        .and_then(|created_category| {
                let result: Category = created_category.into();
                Ok(result)
            })
    }

    /// Updates specific category
    fn update(&self, category_id_arg: i32, payload: UpdateCategory) -> RepoResult<Category> {
        let query = categories.find(category_id_arg);

        query
            .first::<RawCategory>(&**self.db_conn)
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
                    .get_result::<RawCategory>(&**self.db_conn)
                    .map_err(Error::from)
            })
            .and_then(|updated_category| {
                categories
                    .load::<RawCategory>(&**self.db_conn)
                    .map_err(Error::from)
                    .map(|cats| (updated_category, cats))
            })
            .and_then(|(updated_category, cats)| {
                let id_arg = updated_category.id;
                let mut result: Category = updated_category.into();
                let children = create_tree(&cats, Some(id_arg));
                result.children = children;
                Ok(result)
            })
    }

    fn get_all(&self) -> RepoResult<Category> {
        acl::check(
            &*self.acl,
            &Resource::Categories,
            &Action::Read,
            &[],
            Some(self.db_conn),
        ).and_then(|_| {
            categories
                .load::<RawCategory>(&**self.db_conn)
                .map_err(Error::from)
        })
            .and_then(|cats| {
                let mut root = Category::default();
                let children = create_tree(&cats, None);
                root.children = children;
                Ok(root)
            })
    }
}

fn create_tree(cats: &[RawCategory], parent_id_arg: Option<i32>) -> Vec<Category> {
    let mut branch = vec![];
    for cat in cats {
        if cat.parent_id == parent_id_arg {
            let childs = create_tree(cats, Some(cat.id));
            let mut cat_tree: Category = cat.into();
            cat_tree.children = childs;
            branch.push(cat_tree);
        }
    }
    branch
}
