//! Stores Services, presents CRUD operations with stores
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use futures::future::*;
use r2d2::ManageConnection;

use stq_static_resources::ModerationStatus;
use stq_types::{StoreId, UserId};

use super::types::ServiceFuture;
use elastic::{StoresElastic, StoresElasticImpl};
use errors::Error;
use models::{Category, ModeratorStoreSearchTerms, NewStore, SearchStore, Store, UpdateStore, Visibility};
use repos::remove_unused_categories;
use repos::ReposFactory;
use services::Service;

pub trait StoresService {
    /// Returns total store count
    fn count(&self, visibility: Option<Visibility>) -> ServiceFuture<i64>;
    /// Find stores by name limited by `count` parameters
    fn find_store_by_name(self, search_store: SearchStore, count: i32, offset: i32) -> ServiceFuture<Vec<Store>>;
    /// search filters count
    fn search_store_filters_count(&self, search_store: SearchStore) -> ServiceFuture<i32>;
    /// search filters country
    fn search_store_filters_country(&self, search_store: SearchStore) -> ServiceFuture<Vec<String>>;
    /// search filters category
    fn search_store_filters_category(self, search_store: SearchStore) -> ServiceFuture<Category>;
    /// Find stores auto complete limited by `count` parameters
    fn store_auto_complete(&self, name: String, count: i32, offset: i32) -> ServiceFuture<Vec<String>>;
    /// Returns store by ID
    fn get_store(&self, store_id: StoreId, visibility: Option<Visibility>) -> ServiceFuture<Option<Store>>;
    /// Returns products count
    fn get_store_products_count(&self, store_id: StoreId, visibility: Option<Visibility>) -> ServiceFuture<i32>;
    /// Deactivates specific store
    fn deactivate_store(&self, store_id: StoreId) -> ServiceFuture<Store>;
    /// Get store by user id
    fn get_store_by_user(&self, user_id: UserId) -> ServiceFuture<Option<Store>>;
    /// Deactivates store by user id
    fn delete_store_by_user(&self, user_id: UserId) -> ServiceFuture<Option<Store>>;
    /// Creates new store
    fn create_store(&self, payload: NewStore) -> ServiceFuture<Store>;
    /// Lists stores limited by `from` and `count` parameters
    fn list_stores(&self, from: StoreId, count: i32, visibility: Option<Visibility>) -> ServiceFuture<Vec<Store>>;
    /// Updates specific store
    fn update_store(&self, store_id: StoreId, payload: UpdateStore) -> ServiceFuture<Store>;
    /// Checks that slug exists
    fn store_slug_exists(&self, slug: String) -> ServiceFuture<bool>;
    /// Search stores limited by `from`, `skip` and `count` parameters
    fn moderator_search_stores(
        &self,
        from: Option<StoreId>,
        skip: i64,
        count: i64,
        term: ModeratorStoreSearchTerms,
    ) -> ServiceFuture<Vec<Store>>;
    /// Set moderation status for specific store
    fn set_store_moderation_status(&self, store_id: StoreId, status: ModerationStatus) -> ServiceFuture<Store>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > StoresService for Service<T, M, F>
{
    /// Returns total store count
    fn count(&self, visibility: Option<Visibility>) -> ServiceFuture<i64> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let visibility = visibility.unwrap_or(Visibility::Active);

        debug!("Getting store count with visibility = {:?}", visibility);

        self.spawn_on_pool(move |conn| {
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            stores_repo
                .count(visibility)
                .map_err(|e: FailureError| e.context("Service `stores`, `count` endpoint error occurred.").into())
        })
    }

    fn store_auto_complete(&self, name: String, count: i32, offset: i32) -> ServiceFuture<Vec<String>> {
        let client_handle = self.static_context.client_handle.clone();
        let address = self.static_context.config.server.elastic.clone();
        let stores_names = {
            let stores_el = StoresElasticImpl::new(client_handle, address);
            stores_el.auto_complete(name, count, offset)
        };

        Box::new(stores_names.map_err(|e| e.context("Service Stores, auto_complete endpoint error occurred.").into()))
    }

    /// Find stores by name
    fn find_store_by_name(self, search_store: SearchStore, count: i32, offset: i32) -> ServiceFuture<Vec<Store>> {
        let client_handle = self.static_context.client_handle.clone();
        let address = self.static_context.config.server.elastic.clone();
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let stores = {
            let stores_el = StoresElasticImpl::new(client_handle, address);
            stores_el.find_by_name(search_store, count, offset)
        };

        Box::new(
            stores
                .and_then(move |el_stores| {
                    self.spawn_on_pool(move |conn| {
                        let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                        el_stores
                            .into_iter()
                            .map(|el_store| {
                                let store = stores_repo.find(el_store.id, Visibility::Published)?;
                                store.ok_or(
                                    format_err!("Not found such store id : {}", el_store.id)
                                        .context(Error::NotFound)
                                        .into(),
                                )
                            }).collect()
                    })
                }).map_err(|e| e.context("Service Stores, find_by_name endpoint error occurred.").into()),
        )
    }

    /// search filters count
    fn search_store_filters_count(&self, search_store: SearchStore) -> ServiceFuture<i32> {
        let client_handle = self.static_context.client_handle.clone();
        let address = self.static_context.config.server.elastic.clone();
        let search_filters = {
            let stores_el = StoresElasticImpl::new(client_handle, address);
            stores_el.search_count(search_store)
        };

        Box::new(search_filters.map_err(|e| e.context("Service Stores, search_filters_count endpoint error occurred.").into()))
    }

    /// search filters country
    fn search_store_filters_country(&self, search_store: SearchStore) -> ServiceFuture<Vec<String>> {
        let client_handle = self.static_context.client_handle.clone();
        let address = self.static_context.config.server.elastic.clone();
        let search_filters = {
            let stores_el = StoresElasticImpl::new(client_handle, address);
            stores_el.aggregate_countries(search_store)
        };

        Box::new(search_filters.map_err(|e| e.context("Service Stores, search_filters_country endpoint error occurred.").into()))
    }

    /// search filters category
    fn search_store_filters_category(self, search_store: SearchStore) -> ServiceFuture<Category> {
        let client_handle = self.static_context.client_handle.clone();
        let address = self.static_context.config.server.elastic.clone();
        let stores_el = StoresElasticImpl::new(client_handle, address);
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        Box::new(
            stores_el
                .aggregate_categories(search_store)
                .and_then(move |categories_ids| {
                    self.spawn_on_pool(move |conn| {
                        let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                        let root = categories_repo.get_all_categories()?;
                        let new_cat = remove_unused_categories(root, &categories_ids);
                        Ok(new_cat)
                    })
                }).map_err(|e| e.context("Service Stores, search_filters_category endpoint error occurred.").into()),
        )
    }

    /// Returns store by ID
    fn get_store(&self, store_id: StoreId, visibility: Option<Visibility>) -> ServiceFuture<Option<Store>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let visibility = visibility.unwrap_or(Visibility::Published);

        self.spawn_on_pool(move |conn| {
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            stores_repo
                .find(store_id, visibility)
                .map_err(|e| e.context("Service Stores, get endpoint error occurred.").into())
        })
    }

    /// Returns products count
    fn get_store_products_count(&self, store_id: StoreId, visibility: Option<Visibility>) -> ServiceFuture<i32> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let visibility = visibility.unwrap_or(Visibility::Published);

        debug!("Get product count in store with id = {:?}, visibility = {:?}", store_id, visibility);

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            base_products_repo
                .count_with_store_id(store_id, visibility)
                .map_err(|e| e.context("Service Stores, get_products_count endpoint error occurred.").into())
        })
    }

    /// Deactivates specific store
    fn deactivate_store(&self, store_id: StoreId) -> ServiceFuture<Store> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            {
                let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let wizard_stores_repo = repo_factory.create_wizard_stores_repo(&*conn, user_id);
                conn.transaction::<Store, FailureError, _>(move || {
                    let deactive_store = stores_repo.deactivate(store_id)?;

                    let base_products = base_products_repo.deactivate_by_store(store_id)?;

                    for base_product in &base_products {
                        products_repo.deactivate_by_base_product(base_product.id)?;
                    }

                    let _wizard_store = wizard_stores_repo.delete(deactive_store.user_id);

                    Ok(deactive_store)
                })
            }.map_err(|e: FailureError| e.context("Service Stores, deactivate endpoint error occurred.").into())
        })
    }

    /// Delete store by user id
    fn delete_store_by_user(&self, user_id_arg: UserId) -> ServiceFuture<Option<Store>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            stores_repo
                .delete_by_user(user_id_arg)
                .map_err(|e| e.context("Service Stores, delete_by_user endpoint error occurred.").into())
        })
    }

    /// Get store by user id
    fn get_store_by_user(&self, user_id_arg: UserId) -> ServiceFuture<Option<Store>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            stores_repo
                .get_by_user(user_id_arg)
                .map_err(|e| e.context("Service Stores, get_by_user endpoint error occurred.").into())
        })
    }

    /// Lists users limited by `from` and `count` parameters
    fn list_stores(&self, from: StoreId, count: i32, visibility: Option<Visibility>) -> ServiceFuture<Vec<Store>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let visibility = visibility.unwrap_or(Visibility::Published);

        self.spawn_on_pool(move |conn| {
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            stores_repo
                .list(from, count, visibility)
                .map_err(|e| e.context("Service Stores, list endpoint error occurred.").into())
        })
    }

    /// Creates new store
    fn create_store(&self, payload: NewStore) -> ServiceFuture<Store> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        self.spawn_on_pool(move |conn| {
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            conn.transaction::<Store, FailureError, _>(move || {
                let store = stores_repo.get_by_user(payload.user_id)?;
                if store.is_some() {
                    Err(format_err!("Store already exists. User can have only one store.")
                        .context(Error::Validate(
                            validation_errors!({"store": ["store" => "Current user already has a store."]}),
                        )).into())
                } else {
                    let exists = stores_repo.slug_exists(payload.slug.to_string())?;
                    if exists {
                        Err(format_err!("Store with slug '{}' already exists.", payload.slug)
                            .context(Error::Validate(
                                validation_errors!({"slug": ["slug" => "Store with this slug already exists"]}),
                            )).into())
                    } else {
                        stores_repo.create(payload)
                    }
                }
            }).map_err(|e| e.context("Service Stores, create endpoint error occurred.").into())
        })
    }

    /// Updates specific store
    fn update_store(&self, store_id: StoreId, payload: UpdateStore) -> ServiceFuture<Store> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            {
                let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                let store = stores_repo.find(store_id, Visibility::Active)?;
                let store = store.ok_or(format_err!("Not found such store id : {}", store_id).context(Error::NotFound))?;
                if let Some(slug) = payload.slug.clone() {
                    if store.slug != slug {
                        let exists = stores_repo.slug_exists(slug.clone())?;
                        if exists {
                            return Err(format_err!("Store with slug '{}' already exists.", slug)
                                .context(Error::Validate(
                                    validation_errors!({"slug": ["slug" => "Store with this slug already exists"]}),
                                )).into());
                        }
                    }
                }
                let payload = payload.reset_moderation_status();
                stores_repo.update(store_id, payload)
            }.map_err(|e| e.context("Service Stores, update endpoint error occurred.").into())
        })
    }

    /// Checks that slug exists
    fn store_slug_exists(&self, slug: String) -> ServiceFuture<bool> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        self.spawn_on_pool(move |conn| {
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            stores_repo
                .slug_exists(slug)
                .map_err(|e| e.context("Service Stores, slug_exists endpoint error occurred.").into())
        })
    }

    /// Search stores limited by `from`, `skip` and `count` parameters
    fn moderator_search_stores(
        &self,
        from: Option<StoreId>,
        skip: i64,
        count: i64,
        term: ModeratorStoreSearchTerms,
    ) -> ServiceFuture<Vec<Store>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!(
            "Searching for stores (from id: {:?}, skip: {}, count: {}) with payload: {:?}",
            from, skip, count, term
        );

        self.spawn_on_pool(move |conn| {
            let stores_repo = repo_factory.create_stores_repo(&conn, user_id);
            stores_repo
                .moderator_search(from, skip, count, term)
                .map_err(|e: FailureError| e.context("Service `stores`, `moderator_search` endpoint error occurred.").into())
        })
    }

    /// Set moderation status for specific store
    fn set_store_moderation_status(&self, store_id: StoreId, status: ModerationStatus) -> ServiceFuture<Store> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        debug!("Set moderation status {} for store {}", status, &store_id);

        self.spawn_on_pool(move |conn| {
            let stores_repo = repo_factory.create_stores_repo(&conn, user_id);
            stores_repo
                .set_moderation_status(store_id, status)
                .map_err(|e: FailureError| e.context("Service stores, set_moderation_status endpoint error occurred.").into())
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use serde_json;
    use tokio_core::reactor::Core;

    use stq_types::*;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    pub fn create_new_store(name: serde_json::Value) -> NewStore {
        NewStore {
            name,
            user_id: MOCK_USER_ID,
            short_description: serde_json::from_str("{}").unwrap(),
            long_description: None,
            slug: "slug".to_string(),
            cover: None,
            logo: None,
            phone: Some("1234567".to_string()),
            email: Some("example@mail.com".to_string()),
            address: Some("town city street".to_string()),
            facebook_url: None,
            twitter_url: None,
            instagram_url: None,
            default_language: "en".to_string(),
            slogan: Some("fdsf".to_string()),
            country: None,
            country_code: None,
            administrative_area_level_1: None,
            administrative_area_level_2: None,
            locality: None,
            political: None,
            postal_code: None,
            route: None,
            street_number: None,
            place_id: None,
        }
    }

    pub fn create_update_store(name: serde_json::Value) -> UpdateStore {
        UpdateStore {
            name: Some(name),
            short_description: serde_json::from_str("{}").unwrap(),
            long_description: None,
            slug: None,
            cover: None,
            logo: None,
            phone: None,
            email: None,
            address: None,
            facebook_url: None,
            twitter_url: None,
            instagram_url: None,
            default_language: None,
            slogan: None,
            rating: None,
            country: None,
            country_code: None,
            product_categories: None,
            status: None,
            administrative_area_level_1: None,
            administrative_area_level_2: None,
            locality: None,
            political: None,
            postal_code: None,
            route: None,
            street_number: None,
            place_id: None,
        }
    }

    #[test]
    fn test_get_store() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_store(StoreId(1), Some(Visibility::Active));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, StoreId(1));
    }

    #[test]
    fn test_list() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.list_stores(StoreId(1), 5, Some(Visibility::Active));
        let result = core.run(work).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_create_store() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_store = create_new_store(serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap());
        let work = service.create_store(new_store);
        let result = core.run(work).unwrap();
        assert_eq!(
            result.name,
            serde_json::from_str::<serde_json::Value>(MOCK_STORE_NAME_JSON).unwrap()
        );
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_store = create_update_store(serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap());
        let work = service.update_store(StoreId(1), new_store);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, StoreId(1));
        assert_eq!(
            result.name,
            serde_json::from_str::<serde_json::Value>(MOCK_STORE_NAME_JSON).unwrap()
        );
    }

    #[test]
    fn test_deactivate() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.deactivate_store(StoreId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.id, StoreId(1));
        assert_eq!(result.is_active, false);
    }

}
