//! Coupons Services, presents CRUD operations with coupons

use std::time::SystemTime;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use r2d2::ManageConnection;

use failure::Error as FailureError;
use future::IntoFuture;
use futures::future;

use uuid::prelude::*;

use stq_types::{BaseProductId, CouponId, UserId};

use super::types::ServiceFuture;
use errors::Error;
use models::*;
use repos::CouponSearch;

use repos::{CouponValidate, RepoResult, ReposFactory, UsedCouponSearch};
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
    /// Add used coupon for user
    fn add_used_coupon(&self, coupon_id: CouponId, user_id: UserId) -> ServiceFuture<UsedCoupon>;
    /// Delete coupon for user
    fn delete_used_coupon(&self, coupon_id: CouponId, user_id: UserId) -> ServiceFuture<UsedCoupon>;
    /// Validate coupon by coupon code
    fn validate_coupon_by_code(&self, payload: CouponsSearchCodePayload) -> ServiceFuture<Option<CouponValidate>>;
    /// Validate coupon by coupon id
    fn validate_coupon(&self, id_arg: CouponId) -> ServiceFuture<Option<CouponValidate>>;
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
            conn.transaction::<Coupon, FailureError, _>(move || {
                coupon_repo
                    .create(payload)
                    .map_err(|e| e.context("Service Coupons, create endpoint error occurred.").into())
            })
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
            let base_product_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let coupon_repo = repo_factory.create_coupon_repo(&*conn, user_id);

            conn.transaction::<CouponScopeBaseProducts, FailureError, _>(move || {
                let base_product = base_product_repo.find(base_product_id, Visibility::Active)?;
                let coupon = coupon_repo.get(coupon_id)?;

                match (base_product, coupon) {
                    (Some(ref base_product), Some(ref coupon)) if &base_product.store_id == &coupon.store_id => {
                        //do nothing
                    }
                    _ => {
                        return Err(format_err!(
                            "Coupon {} and base product {} do not belong to same store.",
                            coupon_id,
                            base_product_id
                        ).context(Error::Forbidden)
                        .into())
                    }
                }

                coupon_scope_base_products_repo.create(payload)
            }).map_err(|e| {
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

    /// Add used coupon for user
    fn add_used_coupon(&self, coupon_id_arg: CouponId, user_id_arg: UserId) -> ServiceFuture<UsedCoupon> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let payload = NewUsedCoupon {
            coupon_id: coupon_id_arg,
            user_id: user_id_arg,
        };

        self.spawn_on_pool(move |conn| {
            let used_coupons_repo = repo_factory.create_used_coupons_repo(&*conn, user_id);
            conn.transaction::<UsedCoupon, FailureError, _>(move || {
                used_coupons_repo
                    .create(payload)
                    .map_err(|e| e.context("Service Coupons, create endpoint error occurred.").into())
            })
        })
    }

    /// Delete coupon for user
    fn delete_used_coupon(&self, coupon_id_arg: CouponId, user_id_arg: UserId) -> ServiceFuture<UsedCoupon> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let used_coupons_repo = repo_factory.create_used_coupons_repo(&*conn, user_id);

            used_coupons_repo
                .delete(coupon_id_arg, user_id_arg)
                .map_err(|e: FailureError| e.context("Service Coupons, delete endpoint error occurred.").into())
        })
    }

    /// Validate coupon by coupon code
    fn validate_coupon_by_code(&self, payload: CouponsSearchCodePayload) -> ServiceFuture<Option<CouponValidate>> {
        let repo_factory = self.static_context.repo_factory.clone();

        let user_id = match self.dynamic_context.user_id {
            Some(user_id) => user_id,
            None => {
                return Box::new(future::err(
                    format_err!("Denied request to validate coupon for unauthorized user")
                        .context(Error::Forbidden)
                        .into(),
                ));
            }
        };

        self.spawn_on_pool(move |conn| {
            {
                let used_coupons_repo = repo_factory.create_used_coupons_repo(&*conn, Some(user_id));
                let coupon_repo = repo_factory.create_coupon_repo(&*conn, Some(user_id));

                let coupon = coupon_repo.get_by_code(payload.code, payload.store_id)?;

                if let Some(coupon) = coupon {
                    let search_used_coupon = UsedCouponSearch::Coupon(coupon.id);
                    let used_coupons = used_coupons_repo.find_by(search_used_coupon)?;

                    Ok(Some(validate_coupon(coupon, user_id, used_coupons)))
                } else {
                    Ok(None)
                }
            }.map_err(|e: FailureError| {
                e.context("Service Coupons, validate_coupon_by_code endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Validate coupon by coupon id
    fn validate_coupon(&self, id_arg: CouponId) -> ServiceFuture<Option<CouponValidate>> {
        let repo_factory = self.static_context.repo_factory.clone();

        let user_id = match self.dynamic_context.user_id {
            Some(user_id) => user_id,
            None => {
                return Box::new(future::err(
                    format_err!("Denied request to validate coupon for unauthorized user")
                        .context(Error::Forbidden)
                        .into(),
                ));
            }
        };

        self.spawn_on_pool(move |conn| {
            {
                let used_coupons_repo = repo_factory.create_used_coupons_repo(&*conn, Some(user_id));
                let coupon_repo = repo_factory.create_coupon_repo(&*conn, Some(user_id));

                let coupon = coupon_repo.get(id_arg)?;

                if let Some(coupon) = coupon {
                    let search_used_coupon = UsedCouponSearch::Coupon(coupon.id);
                    let used_coupons = used_coupons_repo.find_by(search_used_coupon)?;

                    Ok(Some(validate_coupon(coupon, user_id, used_coupons)))
                } else {
                    Ok(None)
                }
            }.map_err(|e: FailureError| e.context("Service Coupons, validate_coupon endpoint error occurred.").into())
        })
    }
}

pub fn validate_coupon(coupon: Coupon, user_id: UserId, used_coupons: Vec<UsedCoupon>) -> CouponValidate {
    if !coupon.is_active {
        return CouponValidate::NotActive;
    }

    if let Some(expired_at) = coupon.expired_at {
        let now = SystemTime::now();
        if expired_at < now {
            return CouponValidate::HasExpired;
        }
    }

    let user_used_coupon = used_coupons.iter().find(|c| c.user_id == user_id);
    if user_used_coupon.is_some() {
        return CouponValidate::AlreadyActivated;
    }

    if coupon.quantity == Coupon::INFINITE {
        return CouponValidate::Valid;
    }

    if coupon.quantity < 0 {
        return CouponValidate::NoActivationsAvailable;
    }

    let check_result = match (used_coupons.len(), coupon.quantity as usize) {
        (used_coupons_count, quantity) if used_coupons_count >= quantity => Some(CouponValidate::NoActivationsAvailable),
        (used_coupons_count, quantity) if used_coupons_count < quantity => Some(CouponValidate::Valid),
        (_, _) => unreachable!(),
    };

    match check_result {
        Some(check_result) => check_result,
        None => CouponValidate::Valid,
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use std::time::{self, Duration, SystemTime};
    use tokio_core::reactor::Core;

    use stq_types::*;

    use models::*;
    use repos::repo_factory::tests::*;
    use repos::CouponSearch;
    use repos::CouponValidate;
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
    fn test_generate_coupon_code() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.generate_coupon_code();
        let result = core.run(work);
        assert_eq!(result.unwrap().len(), Coupon::MIN_GENERATE_LENGTH_CODE);
    }

    #[test]
    fn test_validate_coupon_code() {
        // only success run function
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let payload = CouponsSearchCodePayload {
            code: CouponCode(MOCK_COUPON_CODE.to_string()),
            store_id: StoreId(1),
        };
        let work = service.validate_coupon_by_code(payload);
        let result = core.run(work);
        assert_eq!(result.is_ok(), true);
    }

    fn create_test_coupon() -> Coupon {
        Coupon {
            id: MOCK_COUPON_ID,
            code: CouponCode(MOCK_COUPON_CODE.to_string()),
            title: "title".to_string(),
            store_id: StoreId(1),
            scope: CouponScope::BaseProducts,
            percent: 0,
            quantity: 0,
            expired_at: None,
            is_active: true,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }

    fn create_used_coupons() -> Vec<UsedCoupon> {
        vec![UsedCoupon {
            coupon_id: MOCK_COUPON_ID,
            user_id: MOCK_USER_ID,
        }]
    }

    static MOCK_USER_ID_PLUS1: UserId = UserId(MOCK_USER_ID.0 + 1);
    static MOCK_USER_ID_PLUS2: UserId = UserId(MOCK_USER_ID.0 + 2);

    #[test]
    fn test_validate_infinity_coupon() {
        let test_coupon = create_test_coupon();
        let used_coupons = create_used_coupons();

        let mut infinity_coupon = test_coupon;
        infinity_coupon.quantity = Coupon::INFINITE;
        assert_eq!(
            CouponValidate::Valid,
            validate_coupon(infinity_coupon, MOCK_USER_ID_PLUS1, used_coupons)
        );
    }

    #[test]
    fn test_validate_not_active_coupon() {
        let test_coupon = create_test_coupon();
        let used_coupons = create_used_coupons();

        let mut not_active_coupon = test_coupon;
        not_active_coupon.is_active = false;
        assert_eq!(
            CouponValidate::NotActive,
            validate_coupon(not_active_coupon, MOCK_USER_ID, used_coupons)
        );
    }

    #[test]
    fn test_validate_already_activated_coupon() {
        let test_coupon = create_test_coupon();
        let used_coupons = create_used_coupons();

        let already_activated_coupon = test_coupon;
        assert_eq!(
            CouponValidate::AlreadyActivated,
            validate_coupon(already_activated_coupon, MOCK_USER_ID, used_coupons)
        );
    }

    #[test]
    fn test_validate_has_expired_coupon() {
        let test_coupon = create_test_coupon();
        let used_coupons = create_used_coupons();

        let mut has_expired_coupon = test_coupon;
        has_expired_coupon.expired_at = Some(SystemTime::now() - Duration::from_secs(86400));
        assert_eq!(
            CouponValidate::HasExpired,
            validate_coupon(has_expired_coupon, MOCK_USER_ID_PLUS1, used_coupons)
        );
    }

    #[test]
    fn test_validate_activations_available_coupon() {
        let test_coupon = create_test_coupon();
        let mut used_coupons = create_used_coupons();
        used_coupons.clear();

        let mut activations_available_coupon = test_coupon;
        activations_available_coupon.quantity = 1;
        assert_eq!(
            CouponValidate::Valid,
            validate_coupon(activations_available_coupon, MOCK_USER_ID_PLUS1, used_coupons)
        );
    }

    #[test]
    fn test_validate_no_activations_available_coupon() {
        let test_coupon = create_test_coupon();
        let mut used_coupons = create_used_coupons();

        used_coupons.push(UsedCoupon {
            coupon_id: MOCK_COUPON_ID,
            user_id: MOCK_USER_ID_PLUS2,
        });

        assert!(used_coupons.len() == 2);

        let mut no_activations_available_coupon = test_coupon;
        no_activations_available_coupon.quantity = 1;
        assert_eq!(
            CouponValidate::NoActivationsAvailable,
            validate_coupon(no_activations_available_coupon, MOCK_USER_ID_PLUS1, used_coupons)
        );
    }
}
