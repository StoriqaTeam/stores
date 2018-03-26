use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use repos::*;
use models::authorization::*;
use stq_acl::{Acl, RolesCache, SystemACL};
use repos::error::RepoError;

pub trait ReposFactory: Default + Copy + Send + 'static {
    fn create_attributes_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<AttributesRepo + 'a>;
    fn create_categories_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<CategoriesRepo + 'a>;
    fn create_category_attrs_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<CategoryAttrsRepo + 'a>;
    fn create_base_product_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<BaseProductsRepo + 'a>;
    fn create_product_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<ProductsRepo + 'a>;
    fn create_product_attrs_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<ProductAttrsRepo + 'a>;
    fn create_stores_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<StoresRepo + 'a>;
    fn create_user_roles_repo<'a, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        db_conn: &'a C,
    ) -> Box<UserRolesRepo + 'a>;
}

#[derive(Default, Copy, Clone)]
pub struct ReposFactoryImpl;

impl ReposFactory for ReposFactoryImpl {
    fn create_attributes_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<AttributesRepo + 'a> {
        let acl = acl_for_id(roles_cache, user_id);
        Box::new(AttributesRepoImpl::new(db_conn, acl)) as Box<AttributesRepo>
    }
    fn create_categories_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<CategoriesRepo + 'a> {
        let acl = acl_for_id(roles_cache, user_id);
        Box::new(CategoriesRepoImpl::new(db_conn, acl)) as Box<CategoriesRepo>
    }
    fn create_category_attrs_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<CategoryAttrsRepo + 'a> {
        let acl = acl_for_id(roles_cache, user_id);
        Box::new(CategoryAttrsRepoImpl::new(db_conn, acl)) as Box<CategoryAttrsRepo>
    }
    fn create_base_product_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<BaseProductsRepo + 'a> {
        let acl = acl_for_id(roles_cache, user_id);
        Box::new(BaseProductsRepoImpl::new(db_conn, acl)) as Box<BaseProductsRepo>
    }
    fn create_product_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<ProductsRepo + 'a> {
        let acl = acl_for_id(roles_cache, user_id);
        Box::new(ProductsRepoImpl::new(db_conn, acl)) as Box<ProductsRepo>
    }
    fn create_product_attrs_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<ProductAttrsRepo + 'a> {
        let acl = acl_for_id(roles_cache, user_id);
        Box::new(ProductAttrsRepoImpl::new(db_conn, acl)) as Box<ProductAttrsRepo>
    }
    fn create_stores_repo<
        'a,
        C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
    >(
        &self,
        db_conn: &'a C,
        roles_cache: T,
        user_id: Option<i32>,
    ) -> Box<StoresRepo + 'a> {
        let acl = acl_for_id(roles_cache, user_id);
        Box::new(StoresRepoImpl::new(db_conn, acl)) as Box<StoresRepo>
    }
    fn create_user_roles_repo<'a, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        db_conn: &'a C,
    ) -> Box<UserRolesRepo + 'a> {
        Box::new(UserRolesRepoImpl::new(
            db_conn,
            Box::new(SystemACL::default()),
        )) as Box<UserRolesRepo>
    }
}

fn acl_for_id<
    C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    T: RolesCache<C, Role = Role, Error = RepoError> + 'static,
>(
    roles_cache: T,
    user_id: Option<i32>,
) -> Box<Acl<Resource, Action, Scope, RepoError, C>> {
    user_id.map_or(
        Box::new(UnauthorizedAcl::default()) as Box<Acl<Resource, Action, Scope, RepoError, C>>,
        |id| (Box::new(ApplicationAcl::new(roles_cache, id)) as Box<Acl<Resource, Action, Scope, RepoError, C>>),
    )
}
