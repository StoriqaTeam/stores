use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use repos::*;
use models::*;
use stq_acl::{Acl, SystemACL};
use repos::error::RepoError;

pub trait ReposFactory<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>
    : Clone + Send + 'static {
    fn create_attributes_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<AttributesRepo + 'a>;
    fn create_categories_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<CategoriesRepo + 'a>;
    fn create_category_attrs_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<CategoryAttrsRepo + 'a>;
    fn create_base_product_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<BaseProductsRepo + 'a>;
    fn create_product_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<ProductsRepo + 'a>;
    fn create_product_attrs_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<ProductAttrsRepo + 'a>;
    fn create_stores_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<StoresRepo + 'a>;
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a>;
}

#[derive(Clone)]
pub struct ReposFactoryImpl {
    roles_cache: RolesCacheImpl,
}

impl ReposFactoryImpl {
    pub fn new(roles_cache: RolesCacheImpl) -> Self {
        Self { roles_cache }
    }

    pub fn get_roles<'a, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        id: i32,
        db_conn: &'a C,
    ) -> Vec<Role> {
        if self.roles_cache.contains(id) {
            self.roles_cache.get(id)
        } else {
            UserRolesRepoImpl::new(
                db_conn,
                Box::new(SystemACL::default()) as Box<Acl<Resource, Action, Scope, RepoError, UserRole>>,
            ).list_for_user(id)
                .and_then(|ref r| {
                    if !r.is_empty() {
                        self.roles_cache.add_roles(id, r);
                    }
                    Ok(r.clone())
                })
                .ok()
                .unwrap_or_default()
        }
    }

    fn get_acl<'a, T, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        db_conn: &'a C,
        user_id: Option<i32>,
    ) -> Box<Acl<Resource, Action, Scope, RepoError, T>> {
        user_id.map_or(
            Box::new(UnauthorizedAcl::default()) as Box<Acl<Resource, Action, Scope, RepoError, T>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, RepoError, T>>)
            },
        )
    }
}

impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryImpl {
    fn create_attributes_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<AttributesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(AttributesRepoImpl::new(db_conn, acl)) as Box<AttributesRepo>
    }
    fn create_categories_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<CategoriesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(CategoriesRepoImpl::new(db_conn, acl)) as Box<CategoriesRepo>
    }
    fn create_category_attrs_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<CategoryAttrsRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(CategoryAttrsRepoImpl::new(db_conn, acl)) as Box<CategoryAttrsRepo>
    }
    fn create_base_product_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<BaseProductsRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(BaseProductsRepoImpl::new(db_conn, acl)) as Box<BaseProductsRepo>
    }
    fn create_product_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<ProductsRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(ProductsRepoImpl::new(db_conn, acl)) as Box<ProductsRepo>
    }
    fn create_product_attrs_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<ProductAttrsRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(ProductAttrsRepoImpl::new(db_conn, acl)) as Box<ProductAttrsRepo>
    }
    fn create_stores_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<StoresRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(StoresRepoImpl::new(db_conn, acl)) as Box<StoresRepo>
    }
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
        Box::new(UserRolesRepoImpl::new(
            db_conn,
            Box::new(SystemACL::default()) as Box<Acl<Resource, Action, Scope, RepoError, UserRole>>,
        )) as Box<UserRolesRepo>
    }
}

#[cfg(test)]
pub mod tests {

    use std::time::SystemTime;
    use std::sync::Arc;
    use std::error::Error;
    use std::fmt;

    use serde_json;
    use futures_cpupool::CpuPool;
    use tokio_core::reactor::Handle;
    use r2d2;
    use r2d2::ManageConnection;
    use diesel::Connection;
    use diesel::ConnectionResult;
    use diesel::QueryResult;
    use diesel::query_builder::AsQuery;
    use diesel::query_builder::QueryFragment;
    use diesel::pg::Pg;
    use diesel::query_builder::QueryId;
    use diesel::sql_types::HasSqlType;
    use diesel::Queryable;
    use diesel::deserialize::QueryableByName;
    use diesel::connection::AnsiTransactionManager;
    use diesel::connection::SimpleConnection;

    use stq_static_resources::Translation;
    use stq_http::client::Config as HttpConfig;
    use stq_http;

    use repos::*;
    use services::*;
    use models::*;
    use config::Config;

    pub const MOCK_REPO_FACTORY: ReposFactoryMock = ReposFactoryMock {};
    pub static MOCK_USER_ID: i32 = 1;
    pub static MOCK_BASE_PRODUCT_ID: i32 = 1;
    pub static MOCK_PRODUCT_ID: i32 = 1;
    pub static MOCK_STORE_NAME_JSON_EXISTED: &'static str = r##"[{"lang": "en","text": "store"}]"##;
    pub static MOCK_STORE_NAME_JSON: &'static str = r##"[{"lang": "de","text": "Store"}]"##;
    pub static MOCK_STORE_NAME: &'static str = "store";
    pub static MOCK_STORE_SLUG: &'static str = "{}";
    pub static MOCK_BASE_PRODUCT_NAME_JSON: &'static str = r##"[{"lang": "en","text": "base product"}]"##;

    #[derive(Default, Copy, Clone)]
    pub struct ReposFactoryMock;

    impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryMock {
        fn create_attributes_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<AttributesRepo + 'a> {
            Box::new(AttributesRepoMock::default()) as Box<AttributesRepo>
        }
        fn create_categories_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<CategoriesRepo + 'a> {
            Box::new(CategoriesRepoMock::default()) as Box<CategoriesRepo>
        }
        fn create_category_attrs_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<CategoryAttrsRepo + 'a> {
            Box::new(CategoryAttrsRepoMock::default()) as Box<CategoryAttrsRepo>
        }
        fn create_base_product_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<BaseProductsRepo + 'a> {
            Box::new(BaseProductsRepoMock::default()) as Box<BaseProductsRepo>
        }
        fn create_product_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<ProductsRepo + 'a> {
            Box::new(ProductsRepoMock::default()) as Box<ProductsRepo>
        }
        fn create_product_attrs_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<ProductAttrsRepo + 'a> {
            Box::new(ProductAttrsRepoMock::default()) as Box<ProductAttrsRepo>
        }
        fn create_stores_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<i32>) -> Box<StoresRepo + 'a> {
            Box::new(StoresRepoMock::default()) as Box<StoresRepo>
        }
        fn create_user_roles_repo<'a>(&self, _db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
            Box::new(UserRolesRepoMock::default()) as Box<UserRolesRepo>
        }
    }

    #[derive(Clone, Default)]
    pub struct AttributesRepoMock;

    impl AttributesRepo for AttributesRepoMock {
        /// Find specific attribute by id
        fn find(&self, id_arg: i32) -> RepoResult<Attribute> {
            Ok(Attribute {
                id: id_arg,
                name: serde_json::from_str("{}").unwrap(),
                value_type: AttributeType::Str,
                meta_field: None,
            })
        }

        /// Creates new attribute
        fn create(&self, payload: NewAttribute) -> RepoResult<Attribute> {
            Ok(Attribute {
                id: 1,
                name: payload.name,
                value_type: AttributeType::Str,
                meta_field: None,
            })
        }

        /// Updates specific attribute
        fn update(&self, attribute_id_arg: i32, payload: UpdateAttribute) -> RepoResult<Attribute> {
            Ok(Attribute {
                id: attribute_id_arg,
                name: payload.name.unwrap(),
                value_type: AttributeType::Str,
                meta_field: None,
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct CategoriesRepoMock;

    impl CategoriesRepo for CategoriesRepoMock {
        /// Find specific category by id
        fn find(&self, id_arg: i32) -> RepoResult<Category> {
            Ok(Category {
                id: id_arg,
                name: serde_json::from_str("{}").unwrap(),
                meta_field: None,
                children: vec![],
                level: 0,
                parent_id: Some(0),
            })
        }

        /// Creates new category
        fn create(&self, payload: NewCategory) -> RepoResult<Category> {
            Ok(Category {
                id: 1,
                name: payload.name,
                meta_field: None,
                children: vec![],
                level: 0,
                parent_id: Some(0),
            })
        }

        /// Updates specific category
        fn update(&self, category_id_arg: i32, payload: UpdateCategory) -> RepoResult<Category> {
            Ok(Category {
                id: category_id_arg,
                name: payload.name.unwrap(),
                meta_field: None,
                children: vec![],
                level: 0,
                parent_id: Some(0),
            })
        }

        /// Returns all categories as a tree
        fn get_all(&self) -> RepoResult<Category> {
            Ok(Category {
                id: 1,
                name: serde_json::from_str("{}").unwrap(),
                meta_field: None,
                children: vec![],
                level: 0,
                parent_id: Some(0),
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct CategoryAttrsRepoMock;

    impl CategoryAttrsRepo for CategoryAttrsRepoMock {
        /// Find category attributes by category ID
        fn find_all_attributes(&self, category_id_arg: i32) -> RepoResult<Vec<CatAttr>> {
            Ok(vec![
                CatAttr {
                    id: 1,
                    cat_id: category_id_arg,
                    attr_id: 1,
                },
            ])
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
    pub struct BaseProductsRepoMock;

    impl BaseProductsRepo for BaseProductsRepoMock {
        /// Find specific base_product by ID
        fn find(&self, base_product_id: i32) -> RepoResult<BaseProduct> {
            Ok(BaseProduct {
                id: base_product_id,
                is_active: true,
                store_id: 1,
                name: serde_json::from_str("{}").unwrap(),
                short_description: serde_json::from_str("{}").unwrap(),
                long_description: None,
                seo_title: None,
                seo_description: None,
                currency_id: 1,
                category_id: 1,
                views: 1,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            })
        }

        /// Returns list of base_products, limited by `from` and `count` parameters
        fn list(&self, from: i32, count: i64) -> RepoResult<Vec<BaseProduct>> {
            let mut base_products = vec![];
            for i in from..(from + count as i32) {
                let base_product = BaseProduct {
                    id: i,
                    is_active: true,
                    store_id: 1,
                    name: serde_json::from_str("{}").unwrap(),
                    short_description: serde_json::from_str("{}").unwrap(),
                    long_description: None,
                    seo_title: None,
                    seo_description: None,
                    currency_id: 1,
                    category_id: 1,
                    views: 1,
                    created_at: SystemTime::now(),
                    updated_at: SystemTime::now(),
                };
                base_products.push(base_product);
            }
            Ok(base_products)
        }

        /// Returns list of base_products by store id, limited by 10
        fn list_by_store(&self, store_id: i32, id: i32) -> RepoResult<Vec<BaseProduct>> {
            let mut base_products = vec![];
            for i in (id + 1)..(id + 10) {
                let base_product = BaseProduct {
                    id: i,
                    is_active: true,
                    store_id: store_id,
                    name: serde_json::from_str("{}").unwrap(),
                    short_description: serde_json::from_str("{}").unwrap(),
                    long_description: None,
                    seo_title: None,
                    seo_description: None,
                    currency_id: 1,
                    category_id: 1,
                    views: 1,
                    created_at: SystemTime::now(),
                    updated_at: SystemTime::now(),
                };
                base_products.push(base_product);
            }
            Ok(base_products)
        }

        /// Creates new base_product
        fn create(&self, payload: NewBaseProduct) -> RepoResult<BaseProduct> {
            Ok(BaseProduct {
                id: 1,
                is_active: true,
                store_id: payload.store_id,
                name: payload.name,
                short_description: payload.short_description,
                long_description: payload.long_description,
                seo_title: payload.seo_title,
                seo_description: payload.seo_description,
                currency_id: payload.currency_id,
                category_id: payload.category_id,
                views: 1,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            })
        }

        /// Updates specific base_product
        fn update(&self, base_product_id: i32, payload: UpdateBaseProduct) -> RepoResult<BaseProduct> {
            Ok(BaseProduct {
                id: base_product_id,
                is_active: true,
                store_id: 1,
                name: payload.name.unwrap(),
                short_description: payload.short_description.unwrap(),
                long_description: payload.long_description,
                seo_title: payload.seo_title,
                seo_description: payload.seo_description,
                currency_id: payload.currency_id.unwrap(),
                category_id: payload.category_id.unwrap(),
                views: 1,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            })
        }

        /// Deactivates specific base_product
        fn deactivate(&self, base_product_id: i32) -> RepoResult<BaseProduct> {
            Ok(BaseProduct {
                id: base_product_id,
                is_active: false,
                store_id: 1,
                name: serde_json::from_str("{}").unwrap(),
                short_description: serde_json::from_str("{}").unwrap(),
                long_description: None,
                seo_title: None,
                seo_description: None,
                currency_id: 1,
                category_id: 1,
                views: 1,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct ProductAttrsRepoMock;

    impl ProductAttrsRepo for ProductAttrsRepoMock {
        /// Find product attributes by product ID
        fn find_all_attributes(&self, product_id_arg: i32) -> RepoResult<Vec<ProdAttr>> {
            Ok(vec![
                ProdAttr {
                    id: 1,
                    prod_id: product_id_arg,
                    base_prod_id: 1,
                    attr_id: 1,
                    value: "value".to_string(),
                    value_type: AttributeType::Str,
                    meta_field: None,
                },
            ])
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
    }

    #[derive(Clone, Default)]
    pub struct UserRolesRepoMock;

    impl UserRolesRepo for UserRolesRepoMock {
        fn list_for_user(&self, user_id_value: i32) -> RepoResult<Vec<Role>> {
            Ok(match user_id_value {
                1 => vec![Role::Superuser],
                _ => vec![Role::User],
            })
        }

        fn create(&self, payload: NewUserRole) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: 123,
                user_id: payload.user_id,
                role: payload.role,
            })
        }

        fn delete(&self, payload: OldUserRole) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: 123,
                user_id: payload.user_id,
                role: payload.role,
            })
        }

        fn delete_by_user_id(&self, user_id_arg: i32) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: 123,
                user_id: user_id_arg,
                role: Role::User,
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct StoresRepoMock;

    impl StoresRepo for StoresRepoMock {
        fn find(&self, store_id: i32) -> RepoResult<Store> {
            let store = create_store(
                store_id,
                serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap(),
            );
            Ok(store)
        }

        fn name_exists(&self, name: Vec<Translation>) -> RepoResult<bool> {
            Ok(name.iter().any(|t| t.text == MOCK_STORE_NAME))
        }

        fn slug_exists(&self, slug: String) -> RepoResult<bool> {
            Ok(slug == MOCK_STORE_SLUG.to_string())
        }

        fn list(&self, from: i32, count: i64) -> RepoResult<Vec<Store>> {
            let mut stores = vec![];
            for i in from..(from + count as i32) {
                let store = create_store(i, serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap());
                stores.push(store);
            }
            Ok(stores)
        }

        fn create(&self, payload: NewStore) -> RepoResult<Store> {
            let store = create_store(1, payload.name);
            Ok(store)
        }

        fn update(&self, store_id: i32, payload: UpdateStore) -> RepoResult<Store> {
            let store = create_store(store_id, payload.name.unwrap());

            Ok(store)
        }

        fn deactivate(&self, store_id: i32) -> RepoResult<Store> {
            let mut store = create_store(
                store_id,
                serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap(),
            );
            store.is_active = false;
            Ok(store)
        }
    }

    #[allow(unused)]
    fn create_store_service(
        user_id: Option<i32>,
        handle: Arc<Handle>,
    ) -> StoresServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        let config = Config::new().unwrap();
        let http_config = HttpConfig {
            http_client_retries: config.client.http_client_retries,
            http_client_buffer_size: config.client.http_client_buffer_size,
        };
        let client = stq_http::client::Client::new(&http_config, &handle);
        let client_handle = client.handle();

        StoresServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            user_id: user_id,
            elastic_address: "127.0.0.1:9200".to_string(),
            client_handle: client_handle,
            repo_factory: MOCK_REPO_FACTORY,
        }
    }

    fn create_store(id: i32, name: serde_json::Value) -> Store {
        Store {
            id: id,
            user_id: 1,
            name: name,
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
        }
    }

    pub fn create_new_store(name: serde_json::Value) -> NewStore {
        NewStore {
            name: name,
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
        }
    }

    #[derive(Clone, Default)]
    pub struct ProductsRepoMock;

    impl ProductsRepo for ProductsRepoMock {
        fn find(&self, product_id: i32) -> RepoResult<Product> {
            let product = create_product(product_id, MOCK_BASE_PRODUCT_ID);
            Ok(product)
        }

        fn find_with_base_id(&self, base_id: i32) -> RepoResult<Vec<Product>> {
            let mut products = vec![];
            let product = create_product(MOCK_PRODUCT_ID, base_id);
            products.push(product);
            Ok(products)
        }

        fn list(&self, from: i32, count: i64) -> RepoResult<Vec<Product>> {
            let mut products = vec![];
            for i in from..(from + count as i32) {
                let product = create_product(i, MOCK_BASE_PRODUCT_ID);
                products.push(product);
            }
            Ok(products)
        }

        fn create(&self, payload: NewProduct) -> RepoResult<Product> {
            let product = create_product(MOCK_PRODUCT_ID, payload.base_product_id);
            Ok(product)
        }

        fn update(&self, product_id: i32, _payload: UpdateProduct) -> RepoResult<Product> {
            let product = create_product(product_id, MOCK_BASE_PRODUCT_ID);

            Ok(product)
        }

        fn deactivate(&self, product_id: i32) -> RepoResult<Product> {
            let mut product = create_product(product_id, MOCK_BASE_PRODUCT_ID);
            product.is_active = false;
            Ok(product)
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

    pub fn create_product(id: i32, base_product_id: i32) -> Product {
        Product {
            id: id,
            base_product_id: base_product_id,
            is_active: true,
            discount: None,
            photo_main: None,
            vendor_code: None,
            cashback: None,
            additional_photos: None,
            price: 0f64,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}
