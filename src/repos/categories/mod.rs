//! Repos contains all info about working with categories
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::sync::Arc;

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use errors::Error;
use failure::Error as FailureError;

use stq_cache::cache::CacheSingle;
use stq_static_resources::ModerationStatus;
use stq_types::{AttributeId, CategoryId, CategorySlug, UserId};

use models::authorization::*;
use models::{Attribute, BaseProductRaw, CatAttr, Category, InsertCategory, NewCategory, RawCategory, UpdateCategory};
use repos::acl;
use repos::legacy_acl::CheckScope;
use repos::types::{RepoAcl, RepoResult};
use schema::attributes::dsl as Attributes;
use schema::base_products::dsl as BaseProducts;
use schema::cat_attr_values::dsl as CategoryAttributes;
use schema::categories::dsl::*;

pub mod category_attrs;
pub mod category_cache;

pub use self::category_attrs::*;
pub use self::category_cache::*;

/// Categories repository, responsible for handling categories_values
pub struct CategoriesRepoImpl<'a, C, T>
where
    C: CacheSingle<Category>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    pub db_conn: &'a T,
    pub acl: Box<RepoAcl<Category>>,
    pub cache: Arc<CategoryCacheImpl<C>>,
}

const CATEGORY_LEVEL3: i32 = 3;

pub trait CategoriesRepo {
    /// Find specific category by id
    fn find(&self, id_arg: CategoryId) -> RepoResult<Option<Category>>;

    /// Find specific category by slug
    fn find_by_slug(&self, slug_arg: CategorySlug) -> RepoResult<Option<Category>>;

    /// Creates new category
    fn create(&self, payload: NewCategory) -> RepoResult<Category>;

    /// Updates specific category
    fn update(&self, category_id_arg: CategoryId, payload: UpdateCategory) -> RepoResult<Category>;

    /// Deletes specific categories
    fn delete_all(&self, category_ids_arg: &[CategoryId]) -> RepoResult<()>;

    /// Returns all categories as a tree
    fn get_all_categories(&self) -> RepoResult<Category>;

    /// Returns all categories as a tree
    /// Tree contains only categories where exists products
    fn get_all_categories_with_products(&self) -> RepoResult<Category>;

    /// Returns all raw categories
    fn get_raw_categories(&self) -> RepoResult<Vec<RawCategory>>;
}

impl<'a, C, T> CategoriesRepoImpl<'a, C, T>
where
    C: CacheSingle<Category>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    pub fn new(db_conn: &'a T, acl: Box<RepoAcl<Category>>, cache: Arc<CategoryCacheImpl<C>>) -> Self {
        Self { db_conn, acl, cache }
    }

    pub fn get_attributes_hash(&self) -> RepoResult<HashMap<AttributeId, Attribute>> {
        Ok(Attributes::attributes
            .load::<Attribute>(self.db_conn)?
            .into_iter()
            .map(|attr| (attr.id, attr))
            .collect())
    }

    pub fn get_categories_hash(&self) -> RepoResult<HashMap<CategoryId, Vec<Attribute>>> {
        let attrs_hash = self.get_attributes_hash()?;

        Ok(CategoryAttributes::cat_attr_values.load::<CatAttr>(self.db_conn)?.into_iter().fold(
            HashMap::<CategoryId, Vec<Attribute>>::new(),
            |mut hash, cat_attr| {
                {
                    let cat_with_attrs = hash.entry(cat_attr.cat_id).or_insert_with(Vec::new);
                    let attribute = &attrs_hash[&cat_attr.attr_id];
                    cat_with_attrs.push(attribute.clone());
                }
                hash
            },
        ))
    }
}

impl<'a, C, T> CategoriesRepo for CategoriesRepoImpl<'a, C, T>
where
    C: CacheSingle<Category>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    /// Find specific category by id
    fn find(&self, id_arg: CategoryId) -> RepoResult<Option<Category>> {
        debug!("Find in categories with id {}.", id_arg);
        acl::check(&*self.acl, Resource::Categories, Action::Read, self, None)?;
        self.get_all_categories().map(|root| get_category(&root, id_arg))
    }

    /// Find specific category by slug
    fn find_by_slug(&self, slug_arg: CategorySlug) -> RepoResult<Option<Category>> {
        debug!("Find in categories with slug {}.", slug_arg);
        acl::check(&*self.acl, Resource::Categories, Action::Read, self, None)?;
        self.get_all_categories().map(|root| get_category_by_slug(&root, &slug_arg))
    }

    /// Creates new category
    fn create(&self, payload: NewCategory) -> RepoResult<Category> {
        debug!("Create new category {:?}.", payload);
        self.cache.remove();

        let new_category_level = if payload.parent_id == CategoryId(0) {
            Ok(1)
        } else {
            categories
                .find(payload.parent_id)
                .get_result::<RawCategory>(self.db_conn)
                .map_err(|e| Error::from(e).into())
                .and_then(|cat| get_child_category_level(cat.into()))
        };

        let payload_clone = payload.clone();
        let new_category = new_category_level.map(|level_| InsertCategory {
            name: payload_clone.name,
            parent_id: payload_clone.parent_id,
            level: level_,
            meta_field: payload_clone.meta_field,
            is_active: true,
            uuid: payload_clone.uuid,
            slug: payload_clone.slug,
        });

        let created_category = new_category
            .and_then(|new_cat| {
                diesel::insert_into(categories)
                    .values(&new_cat)
                    .get_result::<RawCategory>(self.db_conn)
                    .map(|created_category| created_category.into())
                    .map_err(|e| Error::from(e).into())
            })
            .and_then(|category| {
                acl::check(&*self.acl, Resource::Categories, Action::Create, self, Some(&category)).and_then(|_| Ok(category))
            });

        created_category.map_err(|e: FailureError| e.context(format!("Create new category: {:?} error occurred", payload)).into())
    }

    /// Updates specific category
    fn update(&self, category_id_arg: CategoryId, payload: UpdateCategory) -> RepoResult<Category> {
        debug!("Updating category with id {} and payload {:?}.", category_id_arg, payload);
        self.cache.remove();
        let query = categories.find(category_id_arg);
        query
            .get_result::<RawCategory>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|_| acl::check(&*self.acl, Resource::Categories, Action::Update, self, None))
            .and_then(|_| {
                let filter = categories.filter(id.eq(category_id_arg));
                let query = diesel::update(filter).set(&payload);
                query.get_result::<RawCategory>(self.db_conn).map_err(|e| Error::from(e).into())
            })
            .and_then(|updated_category| {
                categories
                    .load::<RawCategory>(self.db_conn)
                    .map_err(|e| Error::from(e).into())
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
                    "Updating category with id {} and payload {:?} error occurred",
                    category_id_arg, payload
                ))
                .into()
            })
    }

    /// Deletes specific categories
    fn delete_all(&self, category_ids_arg: &[CategoryId]) -> RepoResult<()> {
        debug!("Deleting several({}) categories.", category_ids_arg.len());
        self.cache.remove();

        categories
            .filter(id.eq_any(category_ids_arg))
            .load::<RawCategory>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|raw_cats| {
                raw_cats
                    .into_iter()
                    .map(Category::from)
                    .try_for_each(|cat| acl::check(&*self.acl, Resource::Categories, Action::Delete, self, Some(&cat)))
            })?;

        diesel::update(categories)
            .filter(id.eq_any(category_ids_arg))
            .set(is_active.eq(false))
            .execute(self.db_conn)?;

        Ok(())
    }

    fn get_raw_categories(&self) -> RepoResult<Vec<RawCategory>> {
        acl::check(&*self.acl, Resource::Categories, Action::Read, self, None)
            .and_then(|_| {
                categories
                    .filter(is_active.eq(true))
                    .load::<RawCategory>(self.db_conn)
                    .map_err(|e| Error::from(e).into())
            })
            .map_err(|e: FailureError| e.context("Get raw categories error occurred").into())
    }

    fn get_all_categories(&self) -> RepoResult<Category> {
        if let Some(cat) = self.cache.get() {
            debug!("Get all categories from cache request.");
            Ok(cat)
        } else {
            debug!("Get all categories from db request.");
            acl::check(&*self.acl, Resource::Categories, Action::Read, self, None)
                .and_then(|_| {
                    // TODO: use `get_attributes_hash`
                    let attrs_hash = Attributes::attributes
                        .load::<Attribute>(self.db_conn)?
                        .into_iter()
                        .map(|attr| (attr.id, attr))
                        .collect::<HashMap<_, _>>();

                    // TODO use `get_categories_hash`
                    let cat_hash = CategoryAttributes::cat_attr_values.load::<CatAttr>(self.db_conn)?.into_iter().fold(
                        HashMap::<CategoryId, Vec<Attribute>>::new(),
                        |mut hash, cat_attr| {
                            {
                                let cat_with_attrs = hash.entry(cat_attr.cat_id).or_insert_with(Vec::new);
                                let attribute = &attrs_hash[&cat_attr.attr_id];
                                cat_with_attrs.push(attribute.clone());
                            }
                            hash
                        },
                    );

                    let cats = categories.filter(is_active.eq(true)).load::<RawCategory>(self.db_conn)?;
                    let mut root = Category::default();
                    let children = create_tree(&cats, Some(root.id));
                    root.children = children;
                    set_attributes(&mut root, &cat_hash);
                    self.cache.set(root.clone());
                    Ok(root)
                })
                .map_err(|e: FailureError| e.context("Get all categories error occurred").into())
        }
    }

    /// Returns all categories as a tree
    /// Tree contains only categories where exists products
    /// Without use cache!
    fn get_all_categories_with_products(&self) -> RepoResult<Category> {
        debug!("Get all categories with products from db request.");
        acl::check(&*self.acl, Resource::Categories, Action::Read, self, None)
            .and_then(|_| {
                let cat_hash = self.get_categories_hash()?;

                let data: Vec<(RawCategory, Option<BaseProductRaw>)> = categories
                    .filter(is_active.eq(true))
                    .left_join(
                        BaseProducts::base_products.on(BaseProducts::is_active.eq(true).and(
                            BaseProducts::status
                                .eq(ModerationStatus::Published)
                                .and(id.eq(BaseProducts::category_id)),
                        )),
                    )
                    .load(self.db_conn)?;

                let mut cats: Vec<RawCategory> = data
                    .into_iter()
                    .filter_map(|(cat, base_product)| match (cat.level, base_product) {
                        (cat_level, Some(_)) if cat_level == CATEGORY_LEVEL3 => Some(cat),
                        (cat_level, _) if cat_level < CATEGORY_LEVEL3 => Some(cat),
                        _ => None,
                    })
                    .collect();

                cats.sort();
                cats.dedup();

                let mut root = Category::default();
                let children = create_tree(&cats, Some(root.id));
                root.children = children;
                set_attributes(&mut root, &cat_hash);

                Ok(root)
            })
            .map_err(|e: FailureError| e.context("Get `get_all_categories_with_products` error occurred").into())
    }
}

fn create_tree(cats: &[RawCategory], parent_id_arg: Option<CategoryId>) -> Vec<Category> {
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

pub fn remove_unused_categories(mut cat: Category, used_categories_ids: &[CategoryId]) -> Category {
    let mut children = vec![];
    for cat_child in cat.children {
        if used_categories_ids.iter().any(|used_id| cat_child.id == *used_id) {
            children.push(cat_child);
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

pub fn remove_empty_children_categories(mut cat: Category) -> Category {
    let mut children = vec![];
    for cat_child in cat.children {
        let new_cat = remove_empty_children_categories(cat_child);
        if !new_cat.children.is_empty() || new_cat.level == CATEGORY_LEVEL3 {
            children.push(new_cat);
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

pub fn get_parent_category(cat: &Category, child_id: CategoryId, stack_level: i32) -> Option<Category> {
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

pub fn get_category(cat: &Category, cat_id: CategoryId) -> Option<Category> {
    if cat.id == cat_id {
        Some(cat.clone())
    } else {
        cat.children.iter().filter_map(|cat_child| get_category(cat_child, cat_id)).next()
    }
}

pub fn get_category_by_slug(cat: &Category, cat_slug: &CategorySlug) -> Option<Category> {
    if cat.slug == *cat_slug {
        Some(cat.clone())
    } else {
        cat.children
            .iter()
            .filter_map(|cat_child| get_category_by_slug(cat_child, cat_slug))
            .next()
    }
}

pub fn get_child_category_level(parent_cat: Category) -> RepoResult<i32> {
    if parent_cat.level < Category::MAX_LEVEL_NESTING {
        Ok(parent_cat.level + 1)
    } else {
        Err(format_err!(
            "Parent category with id {} is a leaf category (level: {})",
            parent_cat.id,
            Category::MAX_LEVEL_NESTING
        ))
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

pub fn set_attributes<S: BuildHasher>(cat: &mut Category, attrs_hash: &HashMap<CategoryId, Vec<Attribute>, S>) {
    if cat.children.is_empty() {
        let attributes = attrs_hash.get(&cat.id).cloned();
        cat.attributes = attributes.unwrap_or_default();
    } else {
        for cat_child in &mut cat.children {
            set_attributes(cat_child, attrs_hash);
        }
    }
}

impl<'a, C, T> CheckScope<Scope, Category> for CategoriesRepoImpl<'a, C, T>
where
    C: CacheSingle<Category>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
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

    fn create_mock_category(id_: CategoryId, parent_id_: CategoryId, level_: i32) -> Category {
        Category {
            id: id_,
            is_active: true,
            name: serde_json::from_str("{}").unwrap(),
            meta_field: None,
            children: vec![],
            level: level_,
            parent_id: Some(parent_id_),
            attributes: vec![],
            slug: CategorySlug("1".to_string()),
        }
    }

    fn create_mock_category_level1(id_: CategoryId, parent_id_: CategoryId) -> Category {
        create_mock_category(id_, parent_id_, 1)
    }

    fn create_mock_category_level2(id_: CategoryId, parent_id_: CategoryId) -> Category {
        create_mock_category(id_, parent_id_, 2)
    }

    fn create_mock_category_level3(id_: CategoryId, parent_id_: CategoryId) -> Category {
        create_mock_category(id_, parent_id_, 3)
    }

    static CATEGORY_ID_LEVEL1_WITH_2CHILDREN: CategoryId = CategoryId(200);
    static CATEGORY_ID_LEVEL2_WITH_2CHILDREN: CategoryId = CategoryId(301);
    static CATEGORY_ID_LEVEL3_FOR_TEST: CategoryId = CategoryId(402);

    fn create_mock_categories() -> Category {
        let root_id = CategoryId(100);

        let mut cat_1 = create_mock_category_level1(CATEGORY_ID_LEVEL1_WITH_2CHILDREN, root_id.clone());

        let mut cat_2 = create_mock_category_level2(CategoryId(300), cat_1.id);
        let mut cat_2_1 = create_mock_category_level2(CATEGORY_ID_LEVEL2_WITH_2CHILDREN, cat_1.id);

        let cat_3 = create_mock_category_level3(CategoryId(400), cat_2.id);
        let cat_3_1 = create_mock_category_level3(CategoryId(401), cat_2_1.id);
        let cat_3_2 = create_mock_category_level3(CATEGORY_ID_LEVEL3_FOR_TEST, cat_2_1.id);

        cat_2.children = vec![cat_3];

        cat_2_1.children = vec![cat_3_1, cat_3_2];

        cat_1.children = vec![cat_2, cat_2_1];

        Category {
            // root category
            id: root_id,
            is_active: true,
            name: serde_json::from_str("{}").unwrap(),
            meta_field: None,
            children: vec![cat_1],
            level: 0,
            parent_id: None,
            attributes: vec![],
            slug: CategorySlug("1".to_string()),
        }
    }

    #[test]
    fn test_get_intermediate_category_child_level() {
        let lvl1_category = Category {
            id: CategoryId(1000),
            is_active: true,
            name: serde_json::from_str("{}").unwrap(),
            meta_field: None,
            children: vec![],
            level: 1,
            parent_id: None,
            attributes: vec![],
            slug: CategorySlug("1".to_string()),
        };
        let level_ = get_child_category_level(lvl1_category);
        assert_eq!(Some(2), level_.ok());
    }

    #[test]
    fn test_get_leaf_category_child_level() {
        let lvl3_category = Category {
            id: CategoryId(1000),
            is_active: true,
            name: serde_json::from_str("{}").unwrap(),
            meta_field: None,
            children: vec![],
            level: 3,
            parent_id: None,
            attributes: vec![],
            slug: CategorySlug("1".to_string()),
        };
        let level_ = get_child_category_level(lvl3_category);
        assert!(level_.is_err());
    }

    #[test]
    fn test_unused_categories() {
        let mut cat = Category::default();
        cat.id = CategoryId(1);
        for i in 2..4 {
            let mut cat_child = Category::default();
            cat_child.id = CategoryId(i);
            cat_child.parent_id = Some(CategoryId(1));
            for j in 1..3 {
                let mut cat_child_child = Category::default();
                cat_child_child.id = CategoryId(2 * i + j);
                cat_child_child.parent_id = Some(CategoryId(i));
                cat_child.children.push(cat_child_child);
            }
            cat.children.push(cat_child);
        }

        let used = vec![CategoryId(5), CategoryId(6)];
        let new_cat = remove_unused_categories(cat, &used);
        assert_eq!(new_cat.children[0].children[0].id, CategoryId(5));
        assert_eq!(new_cat.children[0].children[1].id, CategoryId(6));
    }

    #[test]
    fn test_used_only_one_category_from_parent_category_level2() {
        let mut category = create_mock_categories();
        let parent_category_code = CATEGORY_ID_LEVEL2_WITH_2CHILDREN;
        let category_code = CATEGORY_ID_LEVEL3_FOR_TEST;
        let used_codes = vec![category_code];

        {
            let parent_category = get_category(&category, parent_category_code)
                .expect(&format!("Not found parent with code {:?} before run test", parent_category_code));

            assert_eq!(
                parent_category.children.len(),
                2,
                "Mock categories not contains 2 children categories"
            );
        }

        category = remove_unused_categories(category, &used_codes);
        let parent_category = get_category(&category, parent_category_code).expect(&format!(
            "Not found parent_category with code {:?} after test",
            parent_category_code
        ));

        assert_eq!(parent_category.children.len(), 1);
        assert_eq!(parent_category.children[0].id, category_code);
    }

    #[test]
    fn test_used_only_one_category_level2() {
        let mut category = create_mock_categories();
        let select_category_code = CATEGORY_ID_LEVEL2_WITH_2CHILDREN;
        let used_codes = vec![select_category_code];

        let parent_category = get_category(&category, CATEGORY_ID_LEVEL1_WITH_2CHILDREN).expect(&format!(
            "Not found parent with code {:?} before run test",
            CATEGORY_ID_LEVEL1_WITH_2CHILDREN
        ));

        assert_eq!(
            parent_category.children.len(),
            2,
            "Mock categories not contains 2 categories level 2"
        );

        category = remove_unused_categories(category, &used_codes);

        let parent_category = get_category(&category, CATEGORY_ID_LEVEL1_WITH_2CHILDREN).expect(&format!(
            "Not found parent_category with code {:?} after test",
            select_category_code
        ));

        assert_eq!(parent_category.children.len(), 1);
        assert_eq!(parent_category.children[0].id, select_category_code);
    }

    #[test]
    fn test_parent_categories() {
        let cat = create_mock_categories();
        let child_id = CategoryId(400);
        let new_cat = cat
            .children
            .into_iter()
            .find(|cat_child| get_parent_category(&cat_child, child_id, 2).is_some())
            .unwrap();
        assert_eq!(new_cat.id, CategoryId(200));
    }

    #[test]
    fn test_get_category() {
        let cat = create_mock_categories();
        let child_id = CategoryId(300);
        let new_cat = get_category(&cat, child_id).unwrap();
        assert_eq!(new_cat.id, child_id);
    }

    #[test]
    fn test_get_category_not_found() {
        let cat = create_mock_categories();
        let child_id = CategoryId(0);
        let new_cat = get_category(&cat, child_id);
        assert!(new_cat.is_none());
    }
}
