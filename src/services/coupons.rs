//! Coupons Services, presents CRUD operations with coupons

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use r2d2::ManageConnection;

use failure::Error as FailureError;
use future::IntoFuture;
use uuid::prelude::*;

use stq_types::{BaseProductId, CouponId};

use super::types::ServiceFuture;
use models::*;
use repos::CouponSearch;

use repos::{RepoResult, ReposFactory};
use services::products::calculate_customer_price;
use services::Service;

pub trait CouponsService {
    /// Creates new coupon
    fn create_coupon(&self, payload: NewCoupon) -> ServiceFuture<Coupon>;
    /// Returns all coupons
    fn list_coupons(&self) -> ServiceFuture<Vec<Coupon>>;
    /// Returns coupon by id
    fn get_coupon(&self, id_arg: CouponId) -> ServiceFuture<Option<Coupon>>;
    /// Returns coupon by code
    fn get_coupon_by_code(&self, payload: CouponsSearchCodePayload) -> ServiceFuture<Option<Coupon>>;
    /// Search coupons
    fn find_coupons(&self, search: CouponSearch) -> ServiceFuture<Vec<Coupon>>;
    /// Update coupon
    fn update_coupon(&self, id_arg: CouponId, payload: UpdateCoupon) -> ServiceFuture<Coupon>;
    /// Deletes coupons
    fn delete_coupon(&self, id_arg: CouponId) -> ServiceFuture<Coupon>;
    /// Add base_product to coupon
    fn add_base_product_coupon(&self, id_arg: CouponId, base_product_arg: BaseProductId) -> ServiceFuture<CouponScopeBaseProducts>;
    /// Delete base_product from coupon
    fn delete_base_product_from_coupon(&self, id_arg: CouponId, base_product_arg: BaseProductId) -> ServiceFuture<CouponScopeBaseProducts>;
    /// Find base products for coupon
    fn find_base_products_by_coupon(&self, id_arg: CouponId) -> ServiceFuture<Vec<BaseProductWithVariants>>;
    /// Generate coupon code
    fn generate_coupon_code(&self) -> ServiceFuture<String>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CouponsService for Service<T, M, F>
{
    /// Creates new coupon
    fn create_coupon(&self, payload: NewCoupon) -> ServiceFuture<Coupon> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let coupon_repo = repo_factory.create_coupon_repo(&*conn, user_id);

            coupon_repo
                .create(payload)
                .map_err(|e| e.context("Service Coupons, create endpoint error occurred.").into())
        })
    }

    /// Returns all coupons
    fn list_coupons(&self) -> ServiceFuture<Vec<Coupon>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let coupon_repo = repo_factory.create_coupon_repo(&*conn, user_id);

            coupon_repo
                .list()
                .map_err(|e| e.context("Service Coupons, list endpoint error occurred.").into())
        })
    }

    /// Returns coupon by id
    fn get_coupon(&self, id_arg: CouponId) -> ServiceFuture<Option<Coupon>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let coupon_repo = repo_factory.create_coupon_repo(&*conn, user_id);

            coupon_repo
                .get(id_arg)
                .map_err(|e| e.context("Service Coupons, get_coupon endpoint error occurred.").into())
        })
    }

    /// Returns coupon by code
    fn get_coupon_by_code(&self, payload: CouponsSearchCodePayload) -> ServiceFuture<Option<Coupon>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let coupon_repo = repo_factory.create_coupon_repo(&*conn, user_id);

            coupon_repo
                .get_by_code(payload.code, payload.store_id)
                .map_err(|e| e.context("Service Coupons, get_coupon_by_code endpoint error occurred.").into())
        })
    }

    /// Search coupons
    fn find_coupons(&self, search: CouponSearch) -> ServiceFuture<Vec<Coupon>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let coupon_repo = repo_factory.create_coupon_repo(&*conn, user_id);

            coupon_repo
                .find_by(search)
                .map_err(|e| e.context("Service Coupons, find_coupons endpoint error occurred.").into())
        })
    }

    /// Update coupon
    fn update_coupon(&self, id_arg: CouponId, payload: UpdateCoupon) -> ServiceFuture<Coupon> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let coupon_repo = repo_factory.create_coupon_repo(&*conn, user_id);

            coupon_repo
                .update(id_arg, payload)
                .map_err(|e| e.context("Service Coupons, update_coupon endpoint error occurred.").into())
        })
    }

    /// Deletes coupons
    fn delete_coupon(&self, id_arg: CouponId) -> ServiceFuture<Coupon> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let coupon_repo = repo_factory.create_coupon_repo(&*conn, user_id);

            coupon_repo
                .delete(id_arg)
                .map_err(|e| e.context("Service Coupons, delete_coupon endpoint error occurred.").into())
        })
    }

    /// Add base_product to coupon
    fn add_base_product_coupon(&self, coupon_id: CouponId, base_product_id: BaseProductId) -> ServiceFuture<CouponScopeBaseProducts> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let payload = NewCouponScopeBaseProducts {
            coupon_id,
            base_product_id,
        };

        self.spawn_on_pool(move |conn| {
            let coupon_scope_base_products_repo = repo_factory.create_coupon_scope_base_products_repo(&*conn, user_id);

            coupon_scope_base_products_repo.create(payload).map_err(|e| {
                e.context("Service Coupons, add_base_product_coupon endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Delete base_product from coupon
    fn delete_base_product_from_coupon(&self, id_arg: CouponId, base_product_arg: BaseProductId) -> ServiceFuture<CouponScopeBaseProducts> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let coupon_scope_base_products_repo = repo_factory.create_coupon_scope_base_products_repo(&*conn, user_id);

            coupon_scope_base_products_repo.delete(id_arg, base_product_arg).map_err(|e| {
                e.context("Service Coupons, delete_base_product_from_coupon endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Find base products for coupon
    fn find_base_products_by_coupon(&self, id_arg: CouponId) -> ServiceFuture<Vec<BaseProductWithVariants>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let currency = self.dynamic_context.currency;

        self.spawn_on_pool(move |conn| {
            {
                let coupon_scope_base_products_repo = repo_factory.create_coupon_scope_base_products_repo(&*conn, user_id);
                let base_products_repo = repo_factory.create_base_product_repo(&conn, user_id);
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);

                let base_product_ids = coupon_scope_base_products_repo.find_base_products(id_arg)?;
                let base_products = base_products_repo.find_many(base_product_ids)?;

                let mut results = vec![];
                for base_product in base_products {
                    let raw_products = products_repo.find_with_base_id(base_product.id)?;

                    let result_products = raw_products
                        .into_iter()
                        .map(|raw_product| {
                            calculate_customer_price(&*currency_exchange, &raw_product, currency)
                                .and_then(|customer_price| Ok(Product::new(raw_product, customer_price)))
                        }).collect::<RepoResult<Vec<Product>>>()?;

                    let base = BaseProductWithVariants::new(base_product, result_products);

                    results.push(base);
                }

                Ok(results)
            }.map_err(|e: FailureError| {
                e.context("Service Coupons, find_base_products_by_coupon endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Generate coupon code
    fn generate_coupon_code(&self) -> ServiceFuture<String> {
        let new_uuid = Uuid::new_v4().simple().to_string().to_uppercase();
        let result = Ok(new_uuid.chars().take(Coupon::MIN_GENERATE_LENGTH_CODE).collect::<String>());

        Box::new(result.into_future())
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;
    use std::time;
    use std::time::SystemTime;
    use tokio_core::reactor::Core;

    use stq_types::{CouponCode, StoreId};

    use models::*;
    use repos::repo_factory::tests::*;
    use repos::CouponSearch;
    use services::*;

    pub fn create_new_coupon(code: CouponCode) -> NewCoupon {
        NewCoupon {
            code,
            title: "title".to_string(),
            store_id: StoreId(1),
            scope: CouponScope::BaseProducts,
            percent: 0,
            quantity: 1,
            expired_at: Some(SystemTime::now() + time::Duration::from_secs(3600)),
        }
    }

    #[test]
    fn test_create_coupon() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_coupon = create_new_coupon(CouponCode(MOCK_COUPON_CODE.to_string()));
        let work = service.create_coupon(new_coupon);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, MOCK_COUPON_ID);
    }

    #[test]
    fn test_get_coupon() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_coupon(MOCK_COUPON_ID);
        let result = core.run(work);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_coupon_by_code() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let payload = CouponsSearchCodePayload {
            code: CouponCode(MOCK_COUPON_CODE.to_string()),
            store_id: StoreId(1),
        };
        let work = service.get_coupon_by_code(payload);
        let result = core.run(work);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_coupon() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.list_coupons();
        let result = core.run(work);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_by_coupon() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.find_coupons(CouponSearch::Store(StoreId(1)));
        let result = core.run(work);
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_coupon() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.delete_coupon(MOCK_COUPON_ID);
        let result = core.run(work);
        assert_eq!(result.unwrap().id, MOCK_COUPON_ID);
    }

    #[test]
    fn test_add_base_product_to_coupon() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.add_base_product_coupon(MOCK_COUPON_ID, MOCK_BASE_PRODUCT_ID);
        let result = core.run(work).unwrap();
        assert_eq!(result.coupon_id, MOCK_COUPON_ID);
    }

    #[test]
    fn test_delete_base_product_from_coupon() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.delete_base_product_from_coupon(MOCK_COUPON_ID, MOCK_BASE_PRODUCT_ID);
        let result = core.run(work);
        assert_eq!(result.unwrap().coupon_id, MOCK_COUPON_ID);
    }

    #[test]
    #[ignore]
    fn test_find_base_products_by_coupon() {}

    #[test]
    #[ignore]
    fn test_generate_coupon_code() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.generate_coupon_code();
        let result = core.run(work);
        assert_eq!(result.unwrap().len(), Coupon::MIN_GENERATE_LENGTH_CODE);
    }

}
