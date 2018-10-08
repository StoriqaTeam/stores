//! Categories Services, presents CRUD operations with categories

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use r2d2::ManageConnection;

use super::types::ServiceFuture;
use errors::Error;
use models::{Attribute, NewCatAttr, OldCatAttr};
use models::{Category, NewCategory, UpdateCategory};
use repos::types::RepoResult;
use repos::ReposFactory;
use services::Service;

pub trait CategoriesService {
    /// Returns category by ID
    fn get_category(&self, category_id: i32) -> ServiceFuture<Option<Category>>;
    /// Creates new category
    fn create_category(&self, payload: NewCategory) -> ServiceFuture<Category>;
    /// Updates specific category
    fn update_category(&self, category_id: i32, payload: UpdateCategory) -> ServiceFuture<Category>;
    /// Returns all categories as a tree
    fn get_all_categories(&self) -> ServiceFuture<Category>;
    /// Returns all category attributes belonging to category
    fn find_all_attributes_for_category(&self, category_id_arg: i32) -> ServiceFuture<Vec<Attribute>>;
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
    fn get_category(&self, category_id: i32) -> ServiceFuture<Option<Category>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            categories_repo
                .find(category_id)
                .map_err(|e| e.context("Service Categories, get endpoint error occured.").into())
        })
    }

    /// Creates new category
    fn create_category(&self, new_category: NewCategory) -> ServiceFuture<Category> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            conn.transaction::<(Category), FailureError, _>(move || categories_repo.create(new_category))
                .map_err(|e| e.context("Service Categories, create endpoint error occured.").into())
        })
    }

    /// Updates specific category
    fn update_category(&self, category_id: i32, payload: UpdateCategory) -> ServiceFuture<Category> {
        let user_id = self.dynamic_context.user_id;

        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            categories_repo
                .update(category_id, payload)
                .map_err(|e| e.context("Service Categories, update endpoint error occured.").into())
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
                .map_err(|e| e.context("Service Categories, get_all_categories endpoint error occured.").into())
        })
    }

    /// Returns all category attributes belonging to category
    fn find_all_attributes_for_category(&self, category_id_arg: i32) -> ServiceFuture<Vec<Attribute>> {
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
                .map_err(|e| e.context("Service Categories, find_all_attributes endpoint error occured.").into())
        })
    }

    /// Creates new category attribute
    fn add_attribute_to_category(&self, payload: NewCatAttr) -> ServiceFuture<()> {
        let user_id = self.dynamic_context.user_id;

        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let category_attrs_repo = repo_factory.create_category_attrs_repo(&*conn, user_id);
            category_attrs_repo.create(payload).map_err(|e| {
                e.context("Service Categories, add_attribute_to_category endpoint error occured.")
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
                e.context("Service Categories, delete_attribute_from_category endpoint error occured.")
                    .into()
            })
        })
    }
}

#[cfg(test)]
pub mod tests {
    use serde_json;
    use std::sync::Arc;
    use tokio_core::reactor::Core;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    pub fn create_new_categories(name: &str) -> NewCategory {
        NewCategory {
            name: serde_json::from_str(name).unwrap(),
            meta_field: None,
            parent_id: 1,
        }
    }

    pub fn create_update_categories(name: &str) -> UpdateCategory {
        UpdateCategory {
            name: Some(serde_json::from_str(name).unwrap()),
            meta_field: None,
            parent_id: Some(1),
            level: Some(0),
        }
    }

    #[test]
    fn test_get_categories() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_category(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, 1);
    }

    #[test]
    fn test_create_categories() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_categories = create_new_categories(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.create_category(new_categories);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_categories = create_update_categories(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.update_category(1, new_categories);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
    }

}
