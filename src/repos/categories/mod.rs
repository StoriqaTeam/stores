//! Repos contains all info about working with categories
use std::collections::HashMap;
use std::hash::BuildHasher;

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::UserId;

use models::attribute::attributes::dsl as Attributes;
use models::authorization::*;
use models::category::categories::dsl::*;
use models::category_attribute::cat_attr_values::dsl as CategoryAttributes;
use models::{Attribute, CatAttr, Category, NewCategory, RawCategory, UpdateCategory};
use repos::acl;
use repos::legacy_acl::{Acl, CheckScope};
use repos::types::RepoResult;

pub mod category_attrs;
pub mod category_cache;

pub use self::category_attrs::*;
pub use self::category_cache::*;

/// Categories repository, responsible for handling categorie_values
pub struct CategoriesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Category>>,
    pub cache: CategoryCacheImpl,
}

pub trait CategoriesRepo {
    /// Find specific category by id
    fn find(&self, id_arg: i32) -> RepoResult<Option<Category>>;

    /// Creates new category
    fn create(&self, payload: NewCategory) -> RepoResult<Category>;

    /// Updates specific category
    fn update(&self, category_id_arg: i32, payload: UpdateCategory) -> RepoResult<Category>;

    /// Returns all categories as a tree
    fn get_all(&self) -> RepoResult<Category>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CategoriesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Category>>, cache: CategoryCacheImpl) -> Self {
        Self { db_conn, acl, cache }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CategoriesRepo for CategoriesRepoImpl<'a, T> {
    /// Find specific category by id
    fn find(&self, id_arg: i32) -> RepoResult<Option<Category>> {
        debug!("Find in categories with id {}.", id_arg);
        acl::check(&*self.acl, Resource::Categories, Action::Read, self, None)?;
        self.get_all().map(|root| get_category(&root, id_arg))
    }

    /// Creates new category
    fn create(&self, payload: NewCategory) -> RepoResult<Category> {
        debug!("Create new category {:?}.", payload);
        self.cache.clear();
        let query_categorie = diesel::insert_into(categories).values(&payload);
        query_categorie
            .get_result::<RawCategory>(self.db_conn)
            .map(|created_category| created_category.into())
            .map_err(From::from)
            .and_then(|category| {
                acl::check(&*self.acl, Resource::Categories, Action::Create, self, Some(&category)).and_then(|_| Ok(category))
            })
            .map_err(|e: FailureError| e.context(format!("Create new category: {:?} error occured", payload)).into())
    }

    /// Updates specific category
    fn update(&self, category_id_arg: i32, payload: UpdateCategory) -> RepoResult<Category> {
        debug!("Updating category with id {} and payload {:?}.", category_id_arg, payload);
        self.cache.clear();
        let query = categories.find(category_id_arg);
        query
            .get_result::<RawCategory>(self.db_conn)
            .map_err(From::from)
            .and_then(|_| acl::check(&*self.acl, Resource::Categories, Action::Update, self, None))
            .and_then(|_| {
                let filter = categories.filter(id.eq(category_id_arg));
                let query = diesel::update(filter).set(&payload);
                query.get_result::<RawCategory>(self.db_conn).map_err(From::from)
            })
            .and_then(|updated_category| {
                categories
                    .load::<RawCategory>(self.db_conn)
                    .map_err(From::from)
                    .map(|cats| (updated_category, cats))
            })
            .map(|(updated_category, cats)| {
                let id_arg = updated_category.id;
                let mut result: Category = updated_category.into();
                let children = create_tree(&cats, Some(id_arg));
                result.children = children;
                result
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Updating category with id {} and payload {:?} error occured",
                    category_id_arg, payload
                )).into()
            })
    }

    fn get_all(&self) -> RepoResult<Category> {
        if let Some(cat) = self.cache.get() {
            debug!("Get all categories from cache request.");
            Ok(cat)
        } else {
            debug!("Get all categories from db request.");
            acl::check(&*self.acl, Resource::Categories, Action::Read, self, None)
                .and_then(|_| {
                    let attrs_hash = Attributes::attributes
                        .load::<Attribute>(self.db_conn)?
                        .into_iter()
                        .map(|attr| (attr.id, attr))
                        .collect::<HashMap<_, _>>();

                    let cat_hash = CategoryAttributes::cat_attr_values.load::<CatAttr>(self.db_conn)?.into_iter().fold(
                        HashMap::<i32, Vec<Attribute>>::new(),
                        |mut hash, cat_attr| {
                            {
                                let cat_with_attrs = hash.entry(cat_attr.cat_id).or_insert_with(Vec::new);
                                let attribute = &attrs_hash[&cat_attr.attr_id];
                                cat_with_attrs.push(attribute.clone());
                            }
                            hash
                        },
                    );

                    let cats = categories.load::<RawCategory>(self.db_conn)?;
                    let mut root = Category::default();
                    let children = create_tree(&cats, None);
                    root.children = children;
                    set_attributes(&mut root, &cat_hash);
                    self.cache.set(root.clone());
                    Ok(root)
                })
                .map_err(|e: FailureError| e.context("Get all categories error occured").into())
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

pub fn remove_unused_categories(mut cat: Category, used_categories_ids: &[i32], stack_level: i32) -> Category {
    let mut children = vec![];
    for cat_child in cat.children {
        if stack_level == 0 {
            if used_categories_ids.iter().any(|used_id| cat_child.id == *used_id) {
                children.push(cat_child);
            }
        } else {
            let new_cat = remove_unused_categories(cat_child, used_categories_ids, stack_level - 1);
            if !new_cat.children.is_empty() {
                children.push(new_cat);
            }
        }
    }
    cat.children = children;
    cat
}

pub fn clear_child_categories(mut cat: Category, stack_level: i32) -> Category {
    if stack_level == 0 {
        cat.children.clear();
    } else {
        let mut cats = vec![];
        for cat_child in cat.children {
            let new_cat = clear_child_categories(cat_child, stack_level - 1);
            cats.push(new_cat);
        }
        cat.children = cats;
    }
    cat
}

pub fn get_parent_category(cat: &Category, child_id: i32, stack_level: i32) -> Option<Category> {
    if stack_level != 0 {
        cat.children
            .iter()
            .find(|cat_child| get_parent_category(cat_child, child_id, stack_level - 1).is_some())
            .and_then(|_| Some(cat.clone()))
    } else if cat.id == child_id {
        Some(cat.clone())
    } else {
        None
    }
}

pub fn get_category(cat: &Category, cat_id: i32) -> Option<Category> {
    if cat.id == cat_id {
        Some(cat.clone())
    } else {
        cat.children.iter().filter_map(|cat_child| get_category(cat_child, cat_id)).next()
    }
}

pub fn get_all_children_till_the_end(cat: Category) -> Vec<Category> {
    if cat.children.is_empty() {
        vec![cat]
    } else {
        let mut kids = vec![];
        for cat_child in cat.children {
            let mut children_kids = get_all_children_till_the_end(cat_child);
            kids.append(&mut children_kids);
        }
        kids
    }
}

pub fn set_attributes<S: BuildHasher>(cat: &mut Category, attrs_hash: &HashMap<i32, Vec<Attribute>, S>) {
    if cat.children.is_empty() {
        let attributes = attrs_hash.get(&cat.id).cloned();
        cat.attributes = attributes;
    } else {
        for cat_child in &mut cat.children {
            set_attributes(cat_child, attrs_hash);
        }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Category>
    for CategoriesRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&Category>) -> bool {
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
    use serde_json;

    fn create_mock_categories() -> Category {
        let cat_3 = Category {
            id: 3,
            name: serde_json::from_str("{}").unwrap(),
            meta_field: None,
            children: vec![],
            level: 3,
            parent_id: Some(2),
            attributes: None,
        };
        let cat_2 = Category {
            id: 2,
            name: serde_json::from_str("{}").unwrap(),
            meta_field: None,
            children: vec![cat_3],
            level: 2,
            parent_id: Some(1),
            attributes: None,
        };
        let cat_1 = Category {
            id: 1,
            name: serde_json::from_str("{}").unwrap(),
            meta_field: None,
            children: vec![cat_2],
            level: 1,
            parent_id: Some(0),
            attributes: None,
        };
        Category {
            id: 0,
            name: serde_json::from_str("{}").unwrap(),
            meta_field: None,
            children: vec![cat_1],
            level: 0,
            parent_id: None,
            attributes: None,
        }
    }

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
        let new_cat = remove_unused_categories(cat, &used, 1);
        assert_eq!(new_cat.children[0].children[0].id, 5);
        assert_eq!(new_cat.children[0].children[1].id, 6);
    }

    #[test]
    fn test_parent_categories() {
        let cat = create_mock_categories();
        let child_id = 3;
        let new_cat = cat.children
            .into_iter()
            .find(|cat_child| get_parent_category(&cat_child, child_id, 2).is_some())
            .unwrap();
        assert_eq!(new_cat.id, 1);
    }

    #[test]
    fn test_get_category() {
        let cat = create_mock_categories();
        let child_id = 3;
        let new_cat = get_category(&cat, child_id).unwrap();
        assert_eq!(new_cat.id, child_id);
    }
}
