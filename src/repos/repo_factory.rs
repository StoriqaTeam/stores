use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use repos::*;
use models::*;
use stq_acl::{Acl, RolesCache, SystemACL};
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
}

impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryImpl {
    fn create_attributes_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<AttributesRepo + 'a> {
        let acl = user_id.map_or(
            Box::new(UnauthorizedAcl::default()) as Box<Acl<Resource, Action, Scope, RepoError, Attribute>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, RepoError, Attribute>>)
            },
        );
        Box::new(AttributesRepoImpl::new(db_conn, acl)) as Box<AttributesRepo>
    }
    fn create_categories_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<CategoriesRepo + 'a> {
        let acl = user_id.map_or(
            Box::new(UnauthorizedAcl::default()) as Box<Acl<Resource, Action, Scope, RepoError, Category>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, RepoError, Category>>)
            },
        );
        Box::new(CategoriesRepoImpl::new(db_conn, acl)) as Box<CategoriesRepo>
    }
    fn create_category_attrs_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<CategoryAttrsRepo + 'a> {
        let acl = user_id.map_or(
            Box::new(UnauthorizedAcl::default()) as Box<Acl<Resource, Action, Scope, RepoError, CatAttr>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, RepoError, CatAttr>>)
            },
        );
        Box::new(CategoryAttrsRepoImpl::new(db_conn, acl)) as Box<CategoryAttrsRepo>
    }
    fn create_base_product_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<BaseProductsRepo + 'a> {
        let acl = user_id.map_or(
            Box::new(UnauthorizedAcl::default()) as Box<Acl<Resource, Action, Scope, RepoError, BaseProduct>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, RepoError, BaseProduct>>)
            },
        );
        Box::new(BaseProductsRepoImpl::new(db_conn, acl)) as Box<BaseProductsRepo>
    }
    fn create_product_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<ProductsRepo + 'a> {
        let acl = user_id.map_or(
            Box::new(UnauthorizedAcl::default()) as Box<Acl<Resource, Action, Scope, RepoError, Product>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, RepoError, Product>>)
            },
        );
        Box::new(ProductsRepoImpl::new(db_conn, acl)) as Box<ProductsRepo>
    }
    fn create_product_attrs_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<ProductAttrsRepo + 'a> {
        let acl = user_id.map_or(
            Box::new(UnauthorizedAcl::default()) as Box<Acl<Resource, Action, Scope, RepoError, ProdAttr>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, RepoError, ProdAttr>>)
            },
        );
        Box::new(ProductAttrsRepoImpl::new(db_conn, acl)) as Box<ProductAttrsRepo>
    }
    fn create_stores_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<StoresRepo + 'a> {
        let acl = user_id.map_or(
            Box::new(UnauthorizedAcl::default()) as Box<Acl<Resource, Action, Scope, RepoError, Store>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, RepoError, Store>>)
            },
        );
        Box::new(StoresRepoImpl::new(db_conn, acl)) as Box<StoresRepo>
    }
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
        Box::new(UserRolesRepoImpl::new(
            db_conn,
            Box::new(SystemACL::default()) as Box<Acl<Resource, Action, Scope, RepoError, UserRole>>,
        )) as Box<UserRolesRepo>
    }
}
