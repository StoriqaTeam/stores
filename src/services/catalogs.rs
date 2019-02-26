//! Rocket Retail Services, provides data from rocket-retail service

use diesel::connection::{AnsiTransactionManager, Connection};
use diesel::pg::Pg;
use r2d2::ManageConnection;

use stq_types::newtypes::UserId;

use super::types::ServiceFuture;
use controller::responses::catalogs::*;
use models::visibility::Visibility;
use repos::repo_factory::ReposFactory;
use services::Service;

pub trait CatalogService {
    fn get_catalog(&self) -> ServiceFuture<CatalogResponse>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CatalogService for Service<T, M, F>
{
    fn get_catalog(&self) -> ServiceFuture<CatalogResponse> {
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            // TODO: security check?
            let base_product_repo = repo_factory.create_base_product_repo(&*conn, Some(UserId(1)));
            let categories_repo = repo_factory.create_categories_repo(&*conn, Some(UserId(1)));
            let stores_repo = repo_factory.create_stores_repo(&*conn, Some(UserId(1)));

            let categories = categories_repo.get_raw_categories()?;
            let categories = categories.into_iter().map(From::from).collect();

            let stores = stores_repo.all(Visibility::Published)?;
            let stores: Vec<CatalogResponseStore> = stores.into_iter().map(From::from).collect();

            let catalog = base_product_repo.get_all_catalog()?;
            let (base_products, products, prod_attrs) = {
                let mut base_products = vec![];
                let mut products = vec![];
                let mut prod_attrs = vec![];

                for bp in catalog {
                    if stores.iter().any(|s| s.id == bp.base_product.store_id) {
                        base_products.push(bp.base_product.into());
                        for p in bp.variants {
                            products.push(p.product.into());
                            for pa in p.attributes {
                                let prod_attr: CatalogResponseProdAttr = pa.into();
                                prod_attrs.push(prod_attr);
                            }
                        }
                    }
                }

                (base_products, products, prod_attrs)
            };

            Ok(CatalogResponse {
                categories,
                stores,
                base_products,
                products,
                prod_attrs,
            })
        })
    }
}
