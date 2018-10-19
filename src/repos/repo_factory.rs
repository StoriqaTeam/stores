use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::*;

use models::*;
use repos::legacy_acl::{Acl, SystemACL};
use repos::*;

pub trait ReposFactory<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>: Clone + Send + 'static {
    fn create_attributes_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<AttributesRepo + 'a>;
    fn create_categories_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CategoriesRepo + 'a>;
    fn create_category_attrs_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CategoryAttrsRepo + 'a>;
    fn create_base_product_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<BaseProductsRepo + 'a>;
    fn create_product_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<ProductsRepo + 'a>;
    fn create_product_attrs_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<ProductAttrsRepo + 'a>;
    fn create_stores_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<StoresRepo + 'a>;
    fn create_wizard_stores_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<WizardStoresRepo + 'a>;
    fn create_moderator_product_comments_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<ModeratorProductRepo + 'a>;
    fn create_moderator_store_comments_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<ModeratorStoreRepo + 'a>;
    fn create_currency_exchange_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CurrencyExchangeRepo + 'a>;
    fn create_custom_attributes_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CustomAttributesRepo + 'a>;
    fn create_user_roles_repo_with_sys_acl<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a>;
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<UserRolesRepo + 'a>;
}

#[derive(Clone)]
pub struct ReposFactoryImpl {
    roles_cache: RolesCacheImpl,
    category_cache: CategoryCacheImpl,
    attribute_cache: AttributeCacheImpl,
}

impl ReposFactoryImpl {
    pub fn new(roles_cache: RolesCacheImpl, category_cache: CategoryCacheImpl, attribute_cache: AttributeCacheImpl) -> Self {
        Self {
            roles_cache,
            category_cache,
            attribute_cache,
        }
    }

    pub fn get_roles<'a, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        id: UserId,
        db_conn: &'a C,
    ) -> Vec<StoresRole> {
        self.create_user_roles_repo_with_sys_acl(db_conn)
            .list_for_user(id)
            .ok()
            .unwrap_or_default()
    }

    fn get_acl<'a, T, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        db_conn: &'a C,
        user_id: Option<UserId>,
    ) -> Box<Acl<Resource, Action, Scope, FailureError, T>> {
        user_id.map_or(
            Box::new(UnauthorizedAcl::default()) as Box<Acl<Resource, Action, Scope, FailureError, T>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, FailureError, T>>)
            },
        )
    }
}

impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryImpl {
    fn create_attributes_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<AttributesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(AttributesRepoImpl::new(db_conn, acl, self.attribute_cache.clone())) as Box<AttributesRepo>
    }
    fn create_categories_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CategoriesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(CategoriesRepoImpl::new(db_conn, acl, self.category_cache.clone())) as Box<CategoriesRepo>
    }
    fn create_category_attrs_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CategoryAttrsRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(CategoryAttrsRepoImpl::new(db_conn, acl, self.category_cache.clone())) as Box<CategoryAttrsRepo>
    }
    fn create_base_product_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<BaseProductsRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(BaseProductsRepoImpl::new(db_conn, acl)) as Box<BaseProductsRepo>
    }
    fn create_product_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<ProductsRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(ProductsRepoImpl::new(db_conn, acl)) as Box<ProductsRepo>
    }
    fn create_product_attrs_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<ProductAttrsRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(ProductAttrsRepoImpl::new(db_conn, acl)) as Box<ProductAttrsRepo>
    }
    fn create_stores_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<StoresRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(StoresRepoImpl::new(db_conn, acl)) as Box<StoresRepo>
    }
    fn create_wizard_stores_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<WizardStoresRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(WizardStoresRepoImpl::new(db_conn, acl)) as Box<WizardStoresRepo>
    }
    fn create_currency_exchange_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CurrencyExchangeRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(CurrencyExchangeRepoImpl::new(db_conn, acl)) as Box<CurrencyExchangeRepo>
    }
    fn create_moderator_product_comments_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<ModeratorProductRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(ModeratorProductRepoImpl::new(db_conn, acl)) as Box<ModeratorProductRepo>
    }
    fn create_moderator_store_comments_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<ModeratorStoreRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(ModeratorStoreRepoImpl::new(db_conn, acl)) as Box<ModeratorStoreRepo>
    }
    fn create_user_roles_repo_with_sys_acl<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
        Box::new(UserRolesRepoImpl::new(
            db_conn,
            Box::new(SystemACL::default()) as Box<Acl<Resource, Action, Scope, FailureError, UserRole>>,
            self.roles_cache.clone(),
        )) as Box<UserRolesRepo>
    }
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<UserRolesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(UserRolesRepoImpl::new(db_conn, acl, self.roles_cache.clone())) as Box<UserRolesRepo>
    }
    fn create_custom_attributes_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<CustomAttributesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(CustomAttributesRepoImpl::new(db_conn, acl)) as Box<CustomAttributesRepo>
    }
}

#[cfg(test)]
pub mod tests {

    use errors::Error as MyError;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fmt;
    use std::sync::Arc;
    use std::time::SystemTime;

    use diesel::connection::AnsiTransactionManager;
    use diesel::connection::SimpleConnection;
    use diesel::deserialize::QueryableByName;
    use diesel::pg::Pg;
    use diesel::query_builder::AsQuery;
    use diesel::query_builder::QueryFragment;
    use diesel::query_builder::QueryId;
    use diesel::sql_types::HasSqlType;
    use diesel::Connection;
    use diesel::ConnectionResult;
    use diesel::QueryResult;
    use diesel::Queryable;
    use futures_cpupool::CpuPool;
    use r2d2;
    use r2d2::ManageConnection;
    use serde_json;
    use tokio_core::reactor::Handle;

    use stq_http;
    use stq_static_resources::*;
    use stq_types::*;

    use config::Config;
    use controller::context::*;
    use models::*;
    use repos::*;
    use services::*;

    pub const MOCK_REPO_FACTORY: ReposFactoryMock = ReposFactoryMock {};
    pub static MOCK_USER_ID: UserId = UserId(1);
    pub static MOCK_BASE_PRODUCT_ID: BaseProductId = BaseProductId(1);
    pub static MOCK_PRODUCT_ID: ProductId = ProductId(1);
    pub static MOCK_STORE_NAME_JSON_EXISTED: &'static str = r##"[{"lang": "en","text": "store"}]"##;
    pub static MOCK_STORE_NAME_JSON: &'static str = r##"[{"lang": "de","text": "Store"}]"##;
    pub static MOCK_STORE_NAME: &'static str = "store";
    pub static MOCK_STORE_SLUG: &'static str = "{}";
    pub static MOCK_BASE_PRODUCT_NAME_JSON: &'static str = r##"[{"lang": "en","text": "base product"}]"##;

    pub fn create_service(
        user_id: Option<UserId>,
        handle: Arc<Handle>,
    ) -> Service<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        let config = Config::new().unwrap();
        let client = stq_http::client::Client::new(&config.to_http_config(), &handle);
        let client_handle = client.handle();
        let static_context = StaticContext::new(db_pool, cpu_pool, client_handle, Arc::new(config), MOCK_REPO_FACTORY);
        let dynamic_context = DynamicContext::new(user_id, Currency::STQ);

        Service::new(static_context, dynamic_context)
    }

    #[derive(Default, Copy, Clone)]
    pub struct ReposFactoryMock;

    impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryMock {
        fn create_attributes_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<AttributesRepo + 'a> {
            Box::new(AttributesRepoMock::default()) as Box<AttributesRepo>
        }
        fn create_categories_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<CategoriesRepo + 'a> {
            Box::new(CategoriesRepoMock::default()) as Box<CategoriesRepo>
        }
        fn create_category_attrs_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<CategoryAttrsRepo + 'a> {
            Box::new(CategoryAttrsRepoMock::default()) as Box<CategoryAttrsRepo>
        }
        fn create_base_product_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<BaseProductsRepo + 'a> {
            Box::new(BaseProductsRepoMock::default()) as Box<BaseProductsRepo>
        }
        fn create_product_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<ProductsRepo + 'a> {
            Box::new(ProductsRepoMock::default()) as Box<ProductsRepo>
        }
        fn create_product_attrs_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<ProductAttrsRepo + 'a> {
            Box::new(ProductAttrsRepoMock::default()) as Box<ProductAttrsRepo>
        }
        fn create_stores_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<StoresRepo + 'a> {
            Box::new(StoresRepoMock::default()) as Box<StoresRepo>
        }
        fn create_wizard_stores_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<WizardStoresRepo + 'a> {
            Box::new(WizardStoresRepoMock::default()) as Box<WizardStoresRepo>
        }
        fn create_currency_exchange_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<CurrencyExchangeRepo + 'a> {
            Box::new(CurrencyExchangeRepoMock::default()) as Box<CurrencyExchangeRepo>
        }
        fn create_moderator_product_comments_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<ModeratorProductRepo + 'a> {
            Box::new(ModeratorProductRepoMock::default()) as Box<ModeratorProductRepo>
        }
        fn create_moderator_store_comments_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<ModeratorStoreRepo + 'a> {
            Box::new(ModeratorStoreRepoMock::default()) as Box<ModeratorStoreRepo>
        }
        fn create_user_roles_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<UserRolesRepo + 'a> {
            Box::new(UserRolesRepoMock::default()) as Box<UserRolesRepo>
        }
        fn create_user_roles_repo_with_sys_acl<'a>(&self, _db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
            Box::new(UserRolesRepoMock::default()) as Box<UserRolesRepo>
        }
        fn create_custom_attributes_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<CustomAttributesRepo + 'a> {
            Box::new(CustomAttributesRepoMock::default()) as Box<CustomAttributesRepo>
        }
    }

    #[derive(Clone, Default)]
    pub struct AttributesRepoMock;

    impl AttributesRepo for AttributesRepoMock {
        /// Find specific attribute by id
        fn find(&self, id_arg: AttributeId) -> RepoResult<Option<Attribute>> {
            Ok(Some(Attribute {
                id: id_arg,
                name: serde_json::from_str("{}").unwrap(),
                value_type: AttributeType::Str,
                meta_field: None,
            }))
        }

        /// List all attributes
        fn list(&self) -> RepoResult<Vec<Attribute>> {
            Ok(vec![])
        }

        /// Creates new attribute
        fn create(&self, payload: NewAttribute) -> RepoResult<Attribute> {
            Ok(Attribute {
                id: AttributeId(1),
                name: payload.name,
                value_type: AttributeType::Str,
                meta_field: None,
            })
        }

        /// Updates specific attribute
        fn update(&self, attribute_id_arg: AttributeId, payload: UpdateAttribute) -> RepoResult<Attribute> {
            Ok(Attribute {
                id: attribute_id_arg,
                name: payload.name.unwrap(),
                value_type: AttributeType::Str,
                meta_field: None,
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct CustomAttributesRepoMock;

    impl CustomAttributesRepo for CustomAttributesRepoMock {
        /// Find custom attributes by base_product_id
        fn find_all_attributes(&self, _base_product_id_arg: BaseProductId) -> RepoResult<Vec<CustomAttribute>> {
            Ok(vec![])
        }

        /// Creates new custom_attribute
        fn create(&self, payload: NewCustomAttribute) -> RepoResult<CustomAttribute> {
            Ok(CustomAttribute {
                id: CustomAttributeId(1),
                base_product_id: payload.base_product_id,
                attribute_id: payload.attribute_id,
            })
        }

        /// List all custom attributes
        fn list(&self) -> RepoResult<Vec<CustomAttribute>> {
            Ok(vec![])
        }

        /// get custom attribute
        fn get_custom_attribute(&self, id_arg: CustomAttributeId) -> RepoResult<Option<CustomAttribute>> {
            Ok(Some(CustomAttribute {
                id: id_arg,
                base_product_id: BaseProductId(1),
                attribute_id: AttributeId(1),
            }))
        }

        /// Delete custom attribute
        fn delete(&self, id_arg: CustomAttributeId) -> RepoResult<CustomAttribute> {
            Ok(CustomAttribute {
                id: id_arg,
                base_product_id: BaseProductId(1),
                attribute_id: AttributeId(1),
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct CategoriesRepoMock;

    impl CategoriesRepo for CategoriesRepoMock {
        /// Find specific category by id
        fn find(&self, id_arg: CategoryId) -> RepoResult<Option<Category>> {
            Ok(Some(Category {
                id: id_arg,
                name: serde_json::from_str("{}").unwrap(),
                meta_field: None,
                children: vec![],
                level: 0,
                parent_id: Some(CategoryId(id_arg.0 - 1)),
                attributes: vec![],
            }))
        }

        /// Creates new category
        fn create(&self, payload: NewCategory) -> RepoResult<Category> {
            Ok(Category {
                id: CategoryId(1),
                name: payload.name,
                meta_field: None,
                children: vec![],
                level: 0,
                parent_id: Some(CategoryId(0)),
                attributes: vec![],
            })
        }

        /// Updates specific category
        fn update(&self, category_id_arg: CategoryId, payload: UpdateCategory) -> RepoResult<Category> {
            Ok(Category {
                id: category_id_arg,
                name: payload.name.unwrap(),
                meta_field: None,
                children: vec![],
                level: 0,
                parent_id: Some(CategoryId(0)),
                attributes: vec![],
            })
        }

        /// Returns all categories as a tree
        fn get_all_categories(&self) -> RepoResult<Category> {
            Ok(create_mock_categories())
        }
    }

    fn create_mock_categories() -> Category {
        let cat_3 = Category {
            id: CategoryId(3),
            name: serde_json::from_str("{}").unwrap(),
            meta_field: None,
            children: vec![],
            level: 3,
            parent_id: Some(CategoryId(2)),
            attributes: vec![],
        };
        let cat_2 = Category {
            id: CategoryId(2),
            name: serde_json::from_str("{}").unwrap(),
            meta_field: None,
            children: vec![cat_3],
            level: 2,
            parent_id: Some(CategoryId(1)),
            attributes: vec![],
        };
        let cat_1 = Category {
            id: CategoryId(1),
            name: serde_json::from_str("{}").unwrap(),
            meta_field: None,
            children: vec![cat_2],
            level: 1,
            parent_id: Some(CategoryId(0)),
            attributes: vec![],
        };
        Category {
            id: CategoryId(0),
            name: serde_json::from_str("{}").unwrap(),
            meta_field: None,
            children: vec![cat_1],
            level: 0,
            parent_id: None,
            attributes: vec![],
        }
    }

    #[derive(Clone, Default)]
    pub struct CategoryAttrsRepoMock;

    impl CategoryAttrsRepo for CategoryAttrsRepoMock {
        /// Find category attributes by category ID
        fn find_all_attributes(&self, category_id_arg: CategoryId) -> RepoResult<Vec<CatAttr>> {
            Ok(vec![CatAttr {
                id: 1,
                cat_id: category_id_arg,
                attr_id: AttributeId(1),
            }])
        }

        /// Creates new category_attribute
        fn create(&self, _payload: NewCatAttr) -> RepoResult<()> {
            Ok(())
        }

        /// Delete attr from category
        fn delete(&self, _payload: OldCatAttr) -> RepoResult<()> {
            Ok(())
        }
    }
    #[derive(Clone, Default)]
    pub struct ModeratorProductRepoMock;

    impl ModeratorProductRepo for ModeratorProductRepoMock {
        /// Find specific comments by base_product ID
        fn find_by_base_product_id(&self, base_product_id: BaseProductId) -> RepoResult<Option<ModeratorProductComments>> {
            Ok(Some(ModeratorProductComments {
                id: 1,
                moderator_id: UserId(1),
                base_product_id,
                comments: "comments".to_string(),
                created_at: SystemTime::now(),
            }))
        }

        /// Creates new comment
        fn create(&self, payload: NewModeratorProductComments) -> RepoResult<ModeratorProductComments> {
            Ok(ModeratorProductComments {
                id: 1,
                moderator_id: payload.moderator_id,
                base_product_id: payload.base_product_id,
                comments: payload.comments,
                created_at: SystemTime::now(),
            })
        }
    }
    #[derive(Clone, Default)]
    pub struct ModeratorStoreRepoMock;

    impl ModeratorStoreRepo for ModeratorStoreRepoMock {
        /// Find specific comments by store ID
        fn find_by_store_id(&self, store_id: StoreId) -> RepoResult<Option<ModeratorStoreComments>> {
            Ok(Some(ModeratorStoreComments {
                id: 1,
                moderator_id: UserId(1),
                store_id,
                comments: "comments".to_string(),
                created_at: SystemTime::now(),
            }))
        }

        /// Creates new comment
        fn create(&self, payload: NewModeratorStoreComments) -> RepoResult<ModeratorStoreComments> {
            Ok(ModeratorStoreComments {
                id: 1,
                moderator_id: payload.moderator_id,
                store_id: payload.store_id,
                comments: payload.comments,
                created_at: SystemTime::now(),
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct WizardStoresRepoMock;

    impl WizardStoresRepo for WizardStoresRepoMock {
        /// Find specific store by user ID
        fn find_by_user_id(&self, user_id: UserId) -> RepoResult<Option<WizardStore>> {
            Ok(Some(WizardStore {
                user_id,
                ..Default::default()
            }))
        }

        /// Creates new wizard store
        fn create(&self, user_id: UserId) -> RepoResult<WizardStore> {
            Ok(WizardStore {
                user_id,
                ..Default::default()
            })
        }

        /// Updates specific wizard store
        fn update(&self, user_id: UserId, payload: UpdateWizardStore) -> RepoResult<WizardStore> {
            Ok(WizardStore {
                user_id,
                id: 1,
                store_id: payload.store_id,
                name: payload.name,
                short_description: payload.short_description,
                default_language: payload.default_language,
                slug: payload.slug,
                country: payload.country,
                country_code: payload.country_code,
                address: payload.address,
                administrative_area_level_1: payload.administrative_area_level_1,
                administrative_area_level_2: payload.administrative_area_level_2,
                locality: payload.locality,
                political: payload.political,
                postal_code: payload.postal_code,
                route: payload.route,
                street_number: payload.street_number,
                place_id: payload.place_id,
                completed: false,
            })
        }

        /// Delete specific wizard store
        fn delete(&self, user_id: UserId) -> RepoResult<WizardStore> {
            Ok(WizardStore {
                user_id,
                ..Default::default()
            })
        }

        fn wizard_exists(&self, user_id: UserId) -> RepoResult<bool> {
            if user_id == MOCK_USER_ID {
                Ok(false)
            } else {
                Ok(true)
            }
        }
    }

    #[derive(Clone, Default)]
    pub struct CurrencyExchangeRepoMock;

    impl CurrencyExchangeRepo for CurrencyExchangeRepoMock {
        /// Get latest currency exchanges
        fn get_latest(&self) -> RepoResult<Option<CurrencyExchange>> {
            Ok(Some(CurrencyExchange {
                id: Default::default(),
                data: Currency::enum_iter()
                    .map(|cur| (cur, serde_json::from_str("{}").unwrap()))
                    .collect(),
                created_at: SystemTime::now(),
            }))
        }

        /// Get latest currency exchanges for currency
        fn get_exchange_for_currency(&self, _currency: Currency) -> RepoResult<Option<HashMap<Currency, ExchangeRate>>> {
            Ok(None)
        }

        /// Adds latest currency to table
        fn update(&self, _payload: NewCurrencyExchange) -> RepoResult<CurrencyExchange> {
            Ok(CurrencyExchange {
                id: Default::default(),
                data: Currency::enum_iter()
                    .map(|cur| (cur, serde_json::from_str("{}").unwrap()))
                    .collect(),
                created_at: SystemTime::now(),
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct BaseProductsRepoMock;

    impl BaseProductsRepo for BaseProductsRepoMock {
        /// Get base_product count
        fn count(&self, only_active: bool) -> RepoResult<i64> {
            Ok(if only_active { 0 } else { 1 })
        }

        /// Find specific base_product by ID
        fn find(&self, base_product_id: BaseProductId) -> RepoResult<Option<BaseProduct>> {
            Ok(Some(BaseProduct {
                id: base_product_id,
                is_active: true,
                store_id: StoreId(1),
                name: serde_json::from_str("{}").unwrap(),
                short_description: serde_json::from_str("{}").unwrap(),
                long_description: None,
                seo_title: None,
                seo_description: None,
                currency: Currency::STQ,
                category_id: CategoryId(1),
                views: 1,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                rating: 0f64,
                slug: "slug".to_string(),
                status: ModerationStatus::Published,
                kafka_update_no: 0,
            }))
        }

        /// Returns list of base_products, limited by `from` and `count` parameters
        fn list(&self, from: BaseProductId, count: i32) -> RepoResult<Vec<BaseProduct>> {
            let mut base_products = vec![];
            for i in from.0..(from.0 + count) {
                let base_product = BaseProduct {
                    id: BaseProductId(i),
                    is_active: true,
                    store_id: StoreId(1),
                    name: serde_json::from_str("{}").unwrap(),
                    short_description: serde_json::from_str("{}").unwrap(),
                    long_description: None,
                    seo_title: None,
                    seo_description: None,
                    currency: Currency::STQ,
                    category_id: CategoryId(1),
                    views: 1,
                    rating: 0f64,
                    created_at: SystemTime::now(),
                    updated_at: SystemTime::now(),
                    slug: "slug".to_string(),
                    status: ModerationStatus::Published,
                    kafka_update_no: 0,
                };
                base_products.push(base_product);
            }
            Ok(base_products)
        }

        /// Returns list of base_products by store id, limited by 10
        fn get_products_of_the_store(
            &self,
            store_id: StoreId,
            skip_base_product_id: Option<BaseProductId>,
            from: BaseProductId,
            count: i32,
        ) -> RepoResult<Vec<BaseProduct>> {
            let mut base_products = vec![];
            for i in (skip_base_product_id.unwrap().0 + from.0)..(skip_base_product_id.unwrap().0 + from.0 + count) {
                let base_product = BaseProduct {
                    id: BaseProductId(i),
                    is_active: true,
                    store_id,
                    name: serde_json::from_str("{}").unwrap(),
                    short_description: serde_json::from_str("{}").unwrap(),
                    long_description: None,
                    seo_title: None,
                    seo_description: None,
                    currency: Currency::STQ,
                    category_id: CategoryId(1),
                    views: 1,
                    created_at: SystemTime::now(),
                    updated_at: SystemTime::now(),
                    rating: 0f64,
                    slug: "slug".to_string(),
                    status: ModerationStatus::Published,
                    kafka_update_no: 0,
                };
                base_products.push(base_product);
            }
            Ok(base_products)
        }

        /// Find specific base_product by ID
        fn count_with_store_id(&self, _store_id: StoreId) -> RepoResult<i32> {
            Ok(1)
        }

        fn slug_exists(&self, _slug_arg: String) -> RepoResult<bool> {
            Ok(false)
        }

        /// Creates new base_product
        fn create(&self, payload: NewBaseProduct) -> RepoResult<BaseProduct> {
            Ok(BaseProduct {
                id: BaseProductId(1),
                is_active: true,
                store_id: payload.store_id,
                name: payload.name,
                short_description: payload.short_description,
                long_description: payload.long_description,
                seo_title: payload.seo_title,
                seo_description: payload.seo_description,
                currency: payload.currency,
                category_id: payload.category_id,
                views: 1,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                rating: 0f64,
                slug: "slug".to_string(),
                status: ModerationStatus::Published,
                kafka_update_no: 0,
            })
        }

        /// Updates specific base_product
        fn update(&self, base_product_id: BaseProductId, payload: UpdateBaseProduct) -> RepoResult<BaseProduct> {
            Ok(BaseProduct {
                id: base_product_id,
                is_active: true,
                store_id: StoreId(1),
                name: serde_json::from_str("{}").unwrap(),
                short_description: serde_json::from_str("{}").unwrap(),
                long_description: payload.long_description,
                seo_title: payload.seo_title,
                seo_description: payload.seo_description,
                currency: Currency::STQ,
                category_id: CategoryId(3),
                views: 1,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                rating: 0f64,
                slug: "slug".to_string(),
                status: ModerationStatus::Published,
                kafka_update_no: 0,
            })
        }

        /// Update views on specific base_product
        fn update_views(&self, base_product_id_arg: BaseProductId) -> RepoResult<Option<BaseProduct>> {
            Ok(Some(BaseProduct {
                id: base_product_id_arg,
                is_active: true,
                store_id: StoreId(1),
                name: serde_json::from_str("{}").unwrap(),
                short_description: serde_json::from_str("{}").unwrap(),
                long_description: None,
                seo_title: None,
                seo_description: None,
                currency: Currency::STQ,
                category_id: CategoryId(3),
                views: 100,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                rating: 0f64,
                slug: "slug".to_string(),
                status: ModerationStatus::Published,
                kafka_update_no: 0,
            }))
        }

        /// Deactivates specific base_product
        fn deactivate(&self, base_product_id: BaseProductId) -> RepoResult<BaseProduct> {
            Ok(BaseProduct {
                id: base_product_id,
                is_active: false,
                store_id: StoreId(1),
                name: serde_json::from_str("{}").unwrap(),
                short_description: serde_json::from_str("{}").unwrap(),
                long_description: None,
                seo_title: None,
                seo_description: None,
                currency: Currency::STQ,
                category_id: CategoryId(3),
                views: 1,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                rating: 0f64,
                slug: "slug".to_string(),
                status: ModerationStatus::Published,
                kafka_update_no: 0,
            })
        }

        fn deactivate_by_store(&self, store_id: StoreId) -> RepoResult<Vec<BaseProduct>> {
            Ok(vec![BaseProduct {
                id: BaseProductId(1),
                is_active: false,
                store_id: store_id,
                name: serde_json::from_str("{}").unwrap(),
                short_description: serde_json::from_str("{}").unwrap(),
                long_description: None,
                seo_title: None,
                seo_description: None,
                currency: Currency::STQ,
                category_id: CategoryId(3),
                views: 1,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                rating: 0f64,
                slug: "slug".to_string(),
                status: ModerationStatus::Published,
                kafka_update_no: 0,
            }])
        }

        fn most_viewed(&self, _prod: MostViewedProducts, _count: i32, _offset: i32) -> RepoResult<Vec<BaseProductWithVariants>> {
            Ok(vec![])
        }

        fn most_discount(&self, _prod: MostDiscountProducts, _count: i32, _offset: i32) -> RepoResult<Vec<BaseProductWithVariants>> {
            Ok(vec![])
        }

        fn convert_from_elastic(&self, _el_products: Vec<ElasticProduct>) -> RepoResult<Vec<BaseProductWithVariants>> {
            Ok(vec![])
        }

        fn moderator_search(
            &self,
            _from: Option<BaseProductId>,
            _skip: i64,
            _count: i64,
            _term: ModeratorBaseProductSearchTerms,
        ) -> RepoResult<Vec<BaseProduct>> {
            Ok(vec![])
        }

        fn set_moderation_status(
            &self,
            _base_product_ids: Vec<BaseProductId>,
            _status_arg: ModerationStatus,
        ) -> RepoResult<Vec<BaseProduct>> {
            Ok(vec![])
        }

        fn get_all_catalog(&self) -> RepoResult<Vec<CatalogWithAttributes>> {
            Ok(vec![])
        }
    }

    #[derive(Clone, Default)]
    pub struct ProductAttrsRepoMock;

    impl ProductAttrsRepo for ProductAttrsRepoMock {
        /// Find product attributes by product ID
        fn find_all_attributes(&self, product_id_arg: ProductId) -> RepoResult<Vec<ProdAttr>> {
            Ok(vec![ProdAttr {
                id: 1,
                prod_id: product_id_arg,
                base_prod_id: BaseProductId(1),
                attr_id: AttributeId(1),
                value: AttributeValue("value".to_string()),
                value_type: AttributeType::Str,
                meta_field: None,
            }])
        }

        /// Find product attributes by product ID
        fn find_all_attributes_by_base(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<ProdAttr>> {
            Ok(vec![ProdAttr {
                id: 1,
                prod_id: ProductId(1),
                base_prod_id: base_product_id_arg,
                attr_id: AttributeId(1),
                value: AttributeValue("value".to_string()),
                value_type: AttributeType::Str,
                meta_field: None,
            }])
        }

        /// Creates new product_attribute
        fn create(&self, payload: NewProdAttr) -> RepoResult<ProdAttr> {
            Ok(ProdAttr {
                id: 1,
                prod_id: payload.prod_id,
                base_prod_id: payload.base_prod_id,
                attr_id: payload.attr_id,
                value: payload.value,
                value_type: payload.value_type,
                meta_field: payload.meta_field,
            })
        }

        /// Updates specific product_attribute
        fn update(&self, payload: UpdateProdAttr) -> RepoResult<ProdAttr> {
            Ok(ProdAttr {
                id: 1,
                prod_id: payload.prod_id,
                base_prod_id: payload.base_prod_id,
                attr_id: payload.attr_id,
                value: payload.value,
                value_type: AttributeType::Str,
                meta_field: payload.meta_field,
            })
        }

        fn delete(&self, id: i32) -> RepoResult<ProdAttr> {
            Ok(ProdAttr {
                id,
                prod_id: ProductId(1),
                base_prod_id: BaseProductId(1),
                attr_id: AttributeId(1),
                value: AttributeValue("value".to_string()),
                value_type: AttributeType::Str,
                meta_field: None,
            })
        }

        fn delete_all_attributes(&self, _product_id_arg: ProductId) -> RepoResult<Vec<ProdAttr>> {
            Ok(vec![])
        }

        fn delete_all_attributes_not_in_list(&self, _product_id_arg: ProductId, _attr_values: Vec<i32>) -> RepoResult<Vec<ProdAttr>> {
            Ok(vec![])
        }
    }

    #[derive(Clone, Default)]
    pub struct UserRolesRepoMock;

    impl UserRolesRepo for UserRolesRepoMock {
        fn list_for_user(&self, user_id_value: UserId) -> RepoResult<Vec<StoresRole>> {
            Ok(match user_id_value.0 {
                1 => vec![StoresRole::Superuser],
                _ => vec![StoresRole::User],
            })
        }

        fn create(&self, payload: NewUserRole) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: RoleId::new(),
                user_id: payload.user_id,
                name: payload.name,
                data: None,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            })
        }

        fn delete_by_user_id(&self, user_id_arg: UserId) -> RepoResult<Vec<UserRole>> {
            Ok(vec![UserRole {
                id: RoleId::new(),
                user_id: user_id_arg,
                name: StoresRole::User,
                data: None,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            }])
        }

        fn delete_by_id(&self, id: RoleId) -> RepoResult<UserRole> {
            Ok(UserRole {
                id,
                user_id: UserId(1),
                name: StoresRole::User,
                data: None,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            })
        }

        fn delete_user_role(&self, user_id_arg: UserId, _name_arg: StoresRole) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: RoleId::new(),
                user_id: user_id_arg,
                name: StoresRole::User,
                data: None,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct StoresRepoMock;

    impl StoresRepo for StoresRepoMock {
        fn count(&self, only_active: bool) -> RepoResult<i64> {
            Ok(if only_active { 0 } else { 1 })
        }

        fn find(&self, store_id: StoreId) -> RepoResult<Option<Store>> {
            let store = create_store(store_id, serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap());
            Ok(Some(store))
        }

        fn name_exists(&self, name: Vec<Translation>) -> RepoResult<bool> {
            Ok(name.iter().any(|t| t.text == MOCK_STORE_NAME))
        }

        fn slug_exists(&self, slug: String) -> RepoResult<bool> {
            Ok(slug == MOCK_STORE_SLUG.to_string())
        }

        fn vendor_code_exists(&self, _store_id: StoreId, _vendor_code: &str) -> RepoResult<Option<bool>> {
            Ok(Some(false))
        }

        fn list(&self, from: StoreId, count: i32) -> RepoResult<Vec<Store>> {
            let mut stores = vec![];
            for i in from.0..(from.0 + count) {
                let store = create_store(StoreId(i), serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap());
                stores.push(store);
            }
            Ok(stores)
        }

        fn create(&self, payload: NewStore) -> RepoResult<Store> {
            let store = create_store(StoreId(1), payload.name);
            Ok(store)
        }

        fn update(&self, store_id: StoreId, payload: UpdateStore) -> RepoResult<Store> {
            let name = if let Some(payload_name) = payload.name {
                payload_name
            } else {
                serde_json::from_str("{}").unwrap()
            };
            let store = create_store(store_id, name);
            Ok(store)
        }

        fn deactivate(&self, store_id: StoreId) -> RepoResult<Store> {
            let mut store = create_store(store_id, serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap());
            store.is_active = false;
            Ok(store)
        }

        fn delete_by_user(&self, _user_id_arg: UserId) -> RepoResult<Option<Store>> {
            Ok(None)
        }

        fn get_by_user(&self, _user_id_arg: UserId) -> RepoResult<Option<Store>> {
            Ok(None)
        }

        fn moderator_search(
            &self,
            from: Option<StoreId>,
            skip: i64,
            count: i64,
            _term: ModeratorStoreSearchTerms,
        ) -> RepoResult<Vec<Store>> {
            let mut stores = vec![];
            let from_id = from.unwrap_or(StoreId(1));
            for i in (from_id.0..).skip(skip as usize).take(count as usize) {
                let store = create_store(StoreId(i), serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap());
                stores.push(store);
            }
            Ok(stores)
        }
        fn set_moderation_status(&self, store_id_arg: StoreId, _status_arg: ModerationStatus) -> RepoResult<Store> {
            let store = create_store(store_id_arg, serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap());
            Ok(store)
        }
    }

    fn create_store(id: StoreId, name: serde_json::Value) -> Store {
        Store {
            id,
            user_id: UserId(1),
            name,
            is_active: true,
            short_description: serde_json::from_str("{}").unwrap(),
            long_description: None,
            slug: "myname".to_string(),
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
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            country: None,
            country_code: None,
            rating: 0f64,
            product_categories: Some(serde_json::from_str("{}").unwrap()),
            status: ModerationStatus::Published,
            administrative_area_level_1: None,
            administrative_area_level_2: None,
            locality: None,
            political: None,
            postal_code: None,
            route: None,
            street_number: None,
            place_id: None,
            kafka_update_no: 0,
        }
    }

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

    #[derive(Clone, Default)]
    pub struct ProductsRepoMock;

    impl ProductsRepo for ProductsRepoMock {
        fn find(&self, product_id: ProductId) -> RepoResult<Option<RawProduct>> {
            let product = create_product(product_id, MOCK_BASE_PRODUCT_ID);
            Ok(Some(product))
        }

        fn find_with_base_id(&self, base_id: BaseProductId) -> RepoResult<Vec<RawProduct>> {
            let mut products = vec![];
            let product = create_product(MOCK_PRODUCT_ID, base_id);
            products.push(product);
            Ok(products)
        }

        fn list(&self, from: i32, count: i32) -> RepoResult<Vec<RawProduct>> {
            let mut products = vec![];
            for i in from..(from + count) {
                let product = create_product(ProductId(i), MOCK_BASE_PRODUCT_ID);
                products.push(product);
            }
            Ok(products)
        }

        fn create(&self, payload: NewProduct) -> RepoResult<RawProduct> {
            if let Some(base_product_id) = payload.base_product_id {
                return Ok(create_product(MOCK_PRODUCT_ID, base_product_id));
            } else {
                return Err(format_err!("Base product id not set.").context(MyError::NotFound).into());
            }
        }

        fn update(&self, product_id: ProductId, _payload: UpdateProduct) -> RepoResult<RawProduct> {
            let product = create_product(product_id, MOCK_BASE_PRODUCT_ID);

            Ok(product)
        }

        fn deactivate(&self, product_id: ProductId) -> RepoResult<RawProduct> {
            let mut product = create_product(product_id, MOCK_BASE_PRODUCT_ID);
            product.is_active = false;
            Ok(product)
        }

        fn update_currency(&self, _currency_arg: Currency, _base_product_id_arg: BaseProductId) -> RepoResult<usize> {
            Ok(1)
        }

        fn find_many(&self, product_ids: Vec<ProductId>) -> RepoResult<Vec<RawProduct>> {
            let mut products = vec![];
            for id in product_ids {
                let product = create_product(id, MOCK_BASE_PRODUCT_ID);
                products.push(product);
            }
            Ok(products)
        }
    }

    #[derive(Default)]
    pub struct MockConnection {
        tr: AnsiTransactionManager,
    }

    impl Connection for MockConnection {
        type Backend = Pg;
        type TransactionManager = AnsiTransactionManager;

        fn establish(_database_url: &str) -> ConnectionResult<MockConnection> {
            Ok(MockConnection::default())
        }

        fn execute(&self, _query: &str) -> QueryResult<usize> {
            unimplemented!()
        }

        fn query_by_index<T, U>(&self, _source: T) -> QueryResult<Vec<U>>
        where
            T: AsQuery,
            T::Query: QueryFragment<Pg> + QueryId,
            Pg: HasSqlType<T::SqlType>,
            U: Queryable<T::SqlType, Pg>,
        {
            unimplemented!()
        }

        fn query_by_name<T, U>(&self, _source: &T) -> QueryResult<Vec<U>>
        where
            T: QueryFragment<Pg> + QueryId,
            U: QueryableByName<Pg>,
        {
            unimplemented!()
        }

        fn execute_returning_count<T>(&self, _source: &T) -> QueryResult<usize>
        where
            T: QueryFragment<Pg> + QueryId,
        {
            unimplemented!()
        }

        fn transaction_manager(&self) -> &Self::TransactionManager {
            &self.tr
        }
    }

    impl SimpleConnection for MockConnection {
        fn batch_execute(&self, _query: &str) -> QueryResult<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    pub struct MockConnectionManager;

    impl ManageConnection for MockConnectionManager {
        type Connection = MockConnection;
        type Error = MockError;

        fn connect(&self) -> Result<MockConnection, MockError> {
            Ok(MockConnection::default())
        }

        fn is_valid(&self, _conn: &mut MockConnection) -> Result<(), MockError> {
            Ok(())
        }

        fn has_broken(&self, _conn: &mut MockConnection) -> bool {
            false
        }
    }

    #[derive(Debug)]
    pub struct MockError {}

    impl fmt::Display for MockError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "SuperError is here!")
        }
    }

    impl Error for MockError {
        fn description(&self) -> &str {
            "I'm the superhero of errors"
        }

        fn cause(&self) -> Option<&Error> {
            None
        }
    }

    pub fn create_product(id: ProductId, base_product_id: BaseProductId) -> RawProduct {
        RawProduct {
            id,
            base_product_id,
            is_active: true,
            discount: None,
            photo_main: None,
            vendor_code: "vendor_code".to_string(),
            cashback: None,
            additional_photos: None,
            price: ProductPrice(0f64),
            currency: Currency::STQ,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            pre_order: false,
            pre_order_days: 0,
            kafka_update_no: 0,
        }
    }
}
