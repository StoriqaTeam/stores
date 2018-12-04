//! Categories Services, presents CRUD operations with categories

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use r2d2::ManageConnection;

use stq_types::{CategoryId, CategorySlug};

use super::types::ServiceFuture;
use errors::Error;
use models::{Attribute, NewCatAttr, OldCatAttr};
use models::{Category, NewCategory, UpdateCategory};
use repos::types::RepoResult;
use repos::{BaseProductsRepo, BaseProductsSearchTerms, CategoriesRepo, ReposFactory};
use services::Service;

pub trait CategoriesService {
    /// Returns category by ID
    fn get_category(&self, category_id: CategoryId) -> ServiceFuture<Option<Category>>;
    /// Returns category by slug
    fn get_category_by_slug(&self, category_slug: CategorySlug) -> ServiceFuture<Option<Category>>;
    /// Creates new category
    fn create_category(&self, payload: NewCategory) -> ServiceFuture<Category>;
    /// Updates specific category
    fn update_category(&self, category_id: CategoryId, payload: UpdateCategory) -> ServiceFuture<Category>;
    /// Deletes category
    fn delete_category(&self, category_id: CategoryId) -> ServiceFuture<()>;
    /// Returns all categories as a tree
    fn get_all_categories(&self) -> ServiceFuture<Category>;
    /// Returns all category attributes belonging to category
    fn find_all_attributes_for_category(&self, category_id_arg: CategoryId) -> ServiceFuture<Vec<Attribute>>;
    /// Creates new category attribute
    fn add_attribute_to_category(&self, payload: NewCatAttr) -> ServiceFuture<()>;
    /// Deletes category attribute
    fn delete_attribute_from_category(&self, payload: OldCatAttr) -> ServiceFuture<()>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CategoriesService for Service<T, M, F>
{
    /// Returns category by ID
    fn get_category(&self, category_id: CategoryId) -> ServiceFuture<Option<Category>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            categories_repo
                .find(category_id)
                .map_err(|e| e.context("Service Categories, get endpoint error occurred.").into())
        })
    }

    /// Returns category by slug
    fn get_category_by_slug(&self, category_slug: CategorySlug) -> ServiceFuture<Option<Category>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            categories_repo
                .find_by_slug(category_slug)
                .map_err(|e| e.context("Service Categories, get by slug endpoint error occurred.").into())
        })
    }

    /// Creates new category
    fn create_category(&self, new_category: NewCategory) -> ServiceFuture<Category> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            conn.transaction::<(Category), FailureError, _>(move || {
                validate_category_create(&*categories_repo, &new_category)?;
                categories_repo.create(new_category)
            }).map_err(|e| e.context("Service Categories, create endpoint error occurred.").into())
        })
    }

    /// Updates specific category
    fn update_category(&self, category_id: CategoryId, payload: UpdateCategory) -> ServiceFuture<Category> {
        let user_id = self.dynamic_context.user_id;

        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            conn.transaction::<(Category), FailureError, _>(move || {
                validate_category_update(&*categories_repo, category_id, &payload)?;
                categories_repo.update(category_id, payload)
            }).map_err(|e| e.context("Service Categories, update endpoint error occurred.").into())
        })
    }

    /// Deletes category
    fn delete_category(&self, category_id: CategoryId) -> ServiceFuture<()> {
        let user_id = self.dynamic_context.user_id;

        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            let category_attrs_repo = repo_factory.create_category_attrs_repo(&*conn, user_id);
            let base_product_repo = repo_factory.create_base_product_repo(&*conn, user_id);

            conn.transaction::<(), FailureError, _>(move || {
                let category: Category = categories_repo
                    .find(category_id)?
                    .ok_or(format_err!("No such category with id : {}", category_id).context(Error::NotFound))?;
                let category_ids = category_and_children_ids(&category);

                validate_category_delete(&category_ids, &*base_product_repo as &BaseProductsRepo)?;

                category_attrs_repo.delete_all_by_category_ids(&category_ids)?;
                categories_repo.delete_all(&category_ids)?;

                Ok(())
            })
        })
    }

    /// Returns category by ID
    fn get_all_categories(&self) -> ServiceFuture<Category> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            categories_repo
                .get_all_categories()
                .map_err(|e| e.context("Service Categories, get_all_categories endpoint error occurred.").into())
        })
    }

    /// Returns all category attributes belonging to category
    fn find_all_attributes_for_category(&self, category_id_arg: CategoryId) -> ServiceFuture<Vec<Attribute>> {
        let user_id = self.dynamic_context.user_id;

        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let category_attrs_repo = repo_factory.create_category_attrs_repo(&*conn, user_id);
            let attrs_repo = repo_factory.create_attributes_repo(&*conn, user_id);
            let cat_attrs = category_attrs_repo.find_all_attributes(category_id_arg)?;
            cat_attrs
                .into_iter()
                .map(|cat_attr| {
                    let attr = attrs_repo.find(cat_attr.attr_id)?;
                    if let Some(attr) = attr {
                        Ok(attr)
                    } else {
                        Err(format_err!("No such attribute with id : {}", cat_attr.attr_id)
                            .context(Error::NotFound)
                            .into())
                    }
                }).collect::<RepoResult<Vec<Attribute>>>()
                .map_err(|e| e.context("Service Categories, find_all_attributes endpoint error occurred.").into())
        })
    }

    /// Creates new category attribute
    fn add_attribute_to_category(&self, payload: NewCatAttr) -> ServiceFuture<()> {
        let user_id = self.dynamic_context.user_id;

        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let category_attrs_repo = repo_factory.create_category_attrs_repo(&*conn, user_id);
            category_attrs_repo.create(payload).map_err(|e| {
                e.context("Service Categories, add_attribute_to_category endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Deletes category attribute
    fn delete_attribute_from_category(&self, payload: OldCatAttr) -> ServiceFuture<()> {
        let user_id = self.dynamic_context.user_id;

        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let category_attrs_repo = repo_factory.create_category_attrs_repo(&*conn, user_id);
            category_attrs_repo.delete(payload).map_err(|e| {
                e.context("Service Categories, delete_attribute_from_category endpoint error occurred.")
                    .into()
            })
        })
    }
}

fn validate_category_create(categories_repo: &CategoriesRepo, category: &NewCategory) -> Result<(), FailureError> {
    if let Some(slug) = category.slug.clone() {
        if let Some(category_with_same_slug) = categories_repo.find_by_slug(slug)? {
            return Err(format_err!("Category {:?} already has the same slug.", category_with_same_slug)
                .context(Error::Validate(
                    validation_errors!({"category_slug": ["category_slug" => "Existing category has the same slug."]}),
                )).into());
        }
    }
    Ok(())
}

fn validate_category_update(
    categories_repo: &CategoriesRepo,
    category_id: CategoryId,
    category: &UpdateCategory,
) -> Result<(), FailureError> {
    if let Some(slug) = category.slug.clone() {
        if let Some(category_with_same_slug) = categories_repo.find_by_slug(slug)? {
            if category_with_same_slug.id != category_id {
                return Err(format_err!("Category {:?} already has the same slug.", category_with_same_slug)
                    .context(Error::Validate(
                        validation_errors!({"category_slug": ["category_slug" => "Existing category has the same slug."]}),
                    )).into());
            }
        }
    }
    Ok(())
}

fn validate_category_delete(category_ids: &[CategoryId], base_products_repo: &BaseProductsRepo) -> Result<(), FailureError> {
    let base_prods_search_terms = BaseProductsSearchTerms {
        category_ids: Some(category_ids.to_vec()),
        is_active: Some(true),
        ..Default::default()
    };
    let active_base_prods_with_target_category = base_products_repo.search(base_prods_search_terms)?;
    if !active_base_prods_with_target_category.is_empty() {
        return Err(format_err!(
            "Category has {} active base products.",
            active_base_prods_with_target_category.len()
        ).context(Error::Validate(
            validation_errors!({"category_id": ["category_id" => "Category has active base products."]}),
        )).into());
    }
    Ok(())
}

fn category_and_children_ids(category: &Category) -> Vec<CategoryId> {
    let mut ids = Vec::new();
    add_ids(category, &mut ids);
    ids
}

fn add_ids(category: &Category, ids: &mut Vec<CategoryId>) {
    ids.push(category.id);
    category.children.iter().for_each(|child| add_ids(child, ids));
}

#[cfg(test)]
pub mod tests {
    use serde_json;
    use std::sync::Arc;
    use tokio_core::reactor::Core;
    use uuid::Uuid;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    use stq_types::CategoryId;

    pub fn create_new_categories(name: &str) -> NewCategory {
        NewCategory {
            name: serde_json::from_str(name).unwrap(),
            meta_field: None,
            parent_id: CategoryId(1),
            uuid: Uuid::new_v4(),
            slug: None,
        }
    }

    pub fn create_update_categories(name: &str) -> UpdateCategory {
        UpdateCategory {
            name: Some(serde_json::from_str(name).unwrap()),
            meta_field: None,
            parent_id: Some(CategoryId(1)),
            level: Some(0),
            slug: None,
        }
    }

    #[test]
    fn test_get_categories() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_category(CategoryId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, CategoryId(1));
    }

    #[test]
    fn test_create_categories() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_categories = create_new_categories(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.create_category(new_categories);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, CategoryId(1));
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_categories = create_update_categories(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.update_category(CategoryId(1), new_categories);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, CategoryId(1));
    }

    #[test]
    fn test_delete() {
        //given
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_category(CategoryId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, CategoryId(1));
        //when
        let work = service.delete_category(CategoryId(1));
        let result = core.run(work);
        //then
        assert!(result.is_ok());
    }

}
