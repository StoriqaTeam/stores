//! Repos contains all info about working with categories
use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use stq_acl::{Acl, CheckScope};

use models::{Category, NewCategory, RawCategory, UpdateCategory};
use models::category::categories::dsl::*;
use models::authorization::*;
use repos::types::RepoResult;
use repos::error::RepoError as Error;
use repos::acl;

pub mod category_attrs;
pub mod category_cache;

pub use self::category_attrs::*;
pub use self::category_cache::*;

/// Categories repository, responsible for handling categorie_values
pub struct CategoriesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, Error, Category>>,
    pub cache: CategoryCacheImpl,
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

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CategoriesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, Error, Category>>, cache: CategoryCacheImpl) -> Self {
        Self {
            db_conn,
            acl,
            cache,
        }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CategoriesRepo for CategoriesRepoImpl<'a, T> {
    /// Find specific category by id
    fn find(&self, id_arg: i32) -> RepoResult<Category> {
        debug!("Find in categories with id {}.", id_arg);
        let query = categories.filter(id.eq(id_arg));

        query
            .first::<RawCategory>(self.db_conn)
            .map_err(Error::from)
            .and_then(|category: RawCategory| {
                acl::check(&*self.acl, &Resource::Categories, &Action::Read, self, None).and_then(|_| Ok(category))
            })
            .and_then(|found_category| {
                categories
                    .load::<RawCategory>(self.db_conn)
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
        debug!("Create new category {:?}.", payload);
        self.cache.clear();
        let query_categorie = diesel::insert_into(categories).values(&payload);
        query_categorie
            .get_result::<RawCategory>(self.db_conn)
            .map_err(Error::from)
            .and_then(|created_category| {
                let result: Category = created_category.into();
                Ok(result)
            })
            .and_then(|category| {
                acl::check(
                    &*self.acl,
                    &Resource::Categories,
                    &Action::Create,
                    self,
                    Some(&category),
                ).and_then(|_| Ok(category))
            })
    }

    /// Updates specific category
    fn update(&self, category_id_arg: i32, payload: UpdateCategory) -> RepoResult<Category> {
        debug!(
            "Updating category with id {} and payload {:?}.",
            category_id_arg, payload
        );
        self.cache.clear();
        let query = categories.find(category_id_arg);
        query
            .first::<RawCategory>(self.db_conn)
            .map_err(Error::from)
            .and_then(|_| {
                acl::check(
                    &*self.acl,
                    &Resource::Categories,
                    &Action::Update,
                    self,
                    None,
                )
            })
            .and_then(|_| {
                let filter = categories.filter(id.eq(category_id_arg));

                let query = diesel::update(filter).set(&payload);
                query
                    .get_result::<RawCategory>(self.db_conn)
                    .map_err(Error::from)
            })
            .and_then(|updated_category| {
                categories
                    .load::<RawCategory>(self.db_conn)
                    .map_err(Error::from)
                    .map(|cats| (updated_category, cats))
            })
            .and_then(|(updated_category, cats)| {
                let id_arg = updated_category.id;
                let mut result: Category = updated_category.into();
                let children = create_tree(&cats, Some(id_arg));
                result.children = children;
                self.cache.set(result.clone());
                Ok(result)
            })
    }

    fn get_all(&self) -> RepoResult<Category> {
        debug!("get all categories request.");
        if self.cache.is_some() {
            self.cache.get()
        } else {
            acl::check(&*self.acl, &Resource::Categories, &Action::Read, self, None)
                .and_then(|_| {
                    categories
                        .load::<RawCategory>(self.db_conn)
                        .map_err(Error::from)
                })
                .and_then(|cats| {
                    let mut root = Category::default();
                    let children = create_tree(&cats, None);
                    root.children = children;
                    self.cache.set(root.clone());
                    Ok(root)
                })
        }
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

pub fn remove_unused_categories(mut cat: Category, used_categories_ids: &[i32]) -> Category {
    let mut children = vec![];
    for cat_child in cat.children.into_iter() {
        if cat_child.children.is_empty() {
            if used_categories_ids
                .iter()
                .any(|used_id| cat_child.id == *used_id)
            {
                children.push(cat_child);
            }
        } else {
            let new_cat = remove_unused_categories(cat_child, used_categories_ids);
            if !new_cat.children.is_empty() {
                children.push(new_cat);
            }
        }
    }
    cat.children = children;
    cat
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Category>
    for CategoriesRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: i32, scope: &Scope, _obj: Option<&Category>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::*;

    #[test]
    fn test_unused() {
        let mut cat = Category::default();
        cat.id = 1;
        for i in 2..4 {
            let mut cat_child = Category::default();
            cat_child.id = i;
            cat_child.parent_id = Some(1);
            for j in 1..3 {
                let mut cat_child_child = Category::default();
                cat_child_child.id = 2 * i + j;
                cat_child_child.parent_id = Some(i);
                cat_child.children.push(cat_child_child);
            }
            cat.children.push(cat_child);
        }

        let used = vec![5, 6];
        remove_unused_categories(&mut cat, &used);
        assert_eq!(cat.children[0].children[0].id, 5);
        assert_eq!(cat.children[0].children[1].id, 6);
    }
}
