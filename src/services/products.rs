//! Products Services, presents CRUD operations with product
use std::collections::HashMap;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use r2d2::ManageConnection;

use stq_static_resources::{Currency, ModerationStatus};
use stq_types::{BaseProductId, ExchangeRate, ProductId, ProductSellerPrice, StoreId};

use super::types::ServiceFuture;
use errors::Error;
use models::*;
use repos::{AttributesRepo, CustomAttributesValuesRepo, ProductAttrsRepo, ReposFactory, StoresRepo};
use services::Service;

pub trait ProductsService {
    /// Returns product by ID
    fn get_product(&self, product_id: ProductId) -> ServiceFuture<Option<Product>>;
    /// Returns product seller price by ID
    fn get_product_seller_price(&self, product_id: ProductId) -> ServiceFuture<Option<ProductSellerPrice>>;
    /// Returns store_id by ID
    fn get_product_store_id(&self, product_id: ProductId) -> ServiceFuture<Option<StoreId>>;
    /// Deactivates specific product
    fn deactivate_product(&self, product_id: ProductId) -> ServiceFuture<Product>;
    /// Creates base product
    fn create_product(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product>;
    /// Lists product variants limited by `from` and `count` parameters
    fn list_products(&self, from: i32, count: i32) -> ServiceFuture<Vec<Product>>;
    /// Updates  product
    fn update_product(&self, product_id: ProductId, payload: UpdateProductWithAttributes) -> ServiceFuture<Product>;
    /// Get by base product id
    fn find_products_with_base_id(&self, base_product_id: BaseProductId) -> ServiceFuture<Vec<Product>>;
    /// Get by base product id
    fn find_products_attributes(&self, product_id: ProductId) -> ServiceFuture<Vec<AttrValue>>;
    /// Get by product id
    fn find_products_custom_attributes(&self, product_id: ProductId) -> ServiceFuture<Vec<CustomAttributeValue>>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ProductsService for Service<T, M, F>
{
    /// Returns product by ID
    fn get_product(&self, product_id: ProductId) -> ServiceFuture<Option<Product>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let currency = self.dynamic_context.currency;

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let product = products_repo.find(product_id)?;
                if let Some(mut product) = product {
                    let currencies_map = currency_exchange.get_exchange_for_currency(currency)?;
                    recalc_currencies(&mut product, &currencies_map, currency);
                    Ok(Some(product))
                } else {
                    Ok(None)
                }
            }.map_err(|e: FailureError| e.context("Service Product, get endpoint error occured.").into())
        })
    }

    /// Returns product seller price by ID
    fn get_product_seller_price(&self, product_id: ProductId) -> ServiceFuture<Option<ProductSellerPrice>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let product = products_repo.find(product_id)?;
                if let Some(product) = product {
                    Ok(Some(ProductSellerPrice {
                        price: product.price,
                        currency: product.currency,
                    }))
                } else {
                    Ok(None)
                }
            }.map_err(|e: FailureError| e.context("Service Product, get endpoint error occured.").into())
        })
    }

    /// Returns store_id by ID
    fn get_product_store_id(&self, product_id: ProductId) -> ServiceFuture<Option<StoreId>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                let product = products_repo.find(product_id)?;
                if let Some(product) = product {
                    let base_product = base_products_repo.find(product.base_product_id)?;
                    if let Some(base_product) = base_product {
                        Ok(Some(base_product.store_id))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }.map_err(|e: FailureError| e.context("Service Product, get_store_id endpoint error occured.").into())
        })
    }

    /// Deactivates specific product
    fn deactivate_product(&self, product_id: ProductId) -> ServiceFuture<Product> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
            conn.transaction::<(Product), FailureError, _>(move || {
                let product = products_repo.deactivate(product_id)?;
                prod_attr_repo.delete_all_attributes(product.id)?;
                Ok(product)
            }).map_err(|e| e.context("Service Product, deactivate endpoint error occured.").into())
        })
    }

    /// Lists users limited by `from` and `count` parameters
    fn list_products(&self, from: i32, count: i32) -> ServiceFuture<Vec<Product>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let currency = self.dynamic_context.currency;

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let mut products = products_repo.list(from, count)?;
                let currencies_map = currency_exchange.get_exchange_for_currency(currency)?;
                products
                    .iter_mut()
                    .for_each(|mut product| recalc_currencies(&mut product, &currencies_map, currency));
                Ok(products)
            }.map_err(|e: FailureError| e.context("Service Product, list endpoint error occured.").into())
        })
    }

    /// Creates new product
    fn create_product(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let attr_repo = repo_factory.create_attributes_repo(&*conn, user_id);
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            conn.transaction::<(Product), FailureError, _>(move || {
                let NewProductWithAttributes { product, attributes } = payload;

                // fill currency id taken from base_product first
                let base_product = base_products_repo.find(product.base_product_id)?;
                let base_product = base_product
                    .ok_or(format_err!("Base product with id {} not found.", product.base_product_id).context(Error::NotFound))?;

                check_vendor_code(&*stores_repo, base_product.store_id, &product.vendor_code)?;

                let product = products_repo.create((product, base_product.currency).into())?;
                create_attributes(&*prod_attr_repo, &*attr_repo, &product, base_product.id, Some(attributes))?;

                Ok(product)
            }).map_err(|e| e.context("Service Product, create endpoint error occured.").into())
        })
    }

    /// Updates specific product
    fn update_product(&self, product_id: ProductId, payload: UpdateProductWithAttributes) -> ServiceFuture<Product> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let attr_repo = repo_factory.create_attributes_repo(&*conn, user_id);
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let custom_attributes_values_repo = repo_factory.create_custom_attributes_values_repo(&*conn, user_id);
            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);

            conn.transaction::<(Product), FailureError, _>(move || {
                let original_product = products_repo
                    .find(product_id)?
                    .ok_or(format_err!("Not found such product id: {}", product_id).context(Error::NotFound))?;

                let product = if let Some(product) = payload.product {
                    if let Some(vendor_code) = &product.vendor_code {
                        let BaseProduct { store_id, .. } = base_products_repo.find(original_product.base_product_id)?.ok_or(
                            format_err!("Base product with id {} not found.", original_product.base_product_id).context(Error::NotFound),
                        )?;

                        check_vendor_code(&*stores_repo, store_id, &vendor_code)?;
                    };

                    let reset_moderation = product.reset_moderation_status_needed();
                    let updated_product = products_repo.update(product_id, product)?;
                    // reset moderation if needed
                    if reset_moderation {
                        let update_base_product = UpdateBaseProduct::update_status(ModerationStatus::Draft);
                        base_products_repo.update(updated_product.base_product_id, update_base_product)?;
                    }
                    updated_product
                } else {
                    original_product
                };

                let UpdateProductWithAttributes {
                    attributes,
                    custom_attributes,
                    ..
                } = payload;

                create_attributes(&*prod_attr_repo, &*attr_repo, &product, product.base_product_id, attributes)?;
                create_custom_attributes(&*custom_attributes_values_repo, custom_attributes, &product)?;

                Ok(product)
            }).map_err(|e| e.context("Service Product, update endpoint error occured.").into())
        })
    }

    /// Get by base product id
    fn find_products_with_base_id(&self, base_product_id: BaseProductId) -> ServiceFuture<Vec<Product>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let currency = self.dynamic_context.currency;

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let mut products = products_repo.find_with_base_id(base_product_id)?;
                let currencies_map = currency_exchange.get_exchange_for_currency(currency)?;
                products
                    .iter_mut()
                    .for_each(|mut product| recalc_currencies(&mut product, &currencies_map, currency));
                Ok(products)
            }.map_err(|e: FailureError| e.context("Service Product, find_with_base_id endpoint error occured.").into())
        })
    }

    /// Get by base product id
    fn find_products_attributes(&self, product_id: ProductId) -> ServiceFuture<Vec<AttrValue>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
            prod_attr_repo
                .find_all_attributes(product_id)
                .map(|pr_attrs| pr_attrs.into_iter().map(|pr_attr| pr_attr.into()).collect())
                .map_err(|e| e.context("Service Product, find_attributes endpoint error occured.").into())
        })
    }

    fn find_products_custom_attributes(&self, product_id: ProductId) -> ServiceFuture<Vec<CustomAttributeValue>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let custom_attributes_values_repo = repo_factory.create_custom_attributes_values_repo(&*conn, user_id);
            custom_attributes_values_repo
                .find_all_attributes(product_id)
                .map_err(|e| e.context("Service Product, find_custom_attributes endpoint error occured.").into())
        })
    }
}

fn recalc_currencies(product: &mut Product, currencies_map: &Option<HashMap<Currency, ExchangeRate>>, currency: Currency) {
    if let Some(currency_map) = currencies_map {
        product.price.0 *= currency_map[&product.currency].0;
        product.currency = currency;
    }
}

fn create_attributes(
    prod_attr_repo: &ProductAttrsRepo,
    attr_repo: &AttributesRepo,
    product_arg: &Product,
    base_product_arg: BaseProductId,
    attributes: Option<Vec<AttrValue>>,
) -> Result<(), FailureError> {
    if let Some(attributes) = attributes {
        // deleting old attributes for this product
        prod_attr_repo.delete_all_attributes(product_arg.id)?;
        // searching for existed product with such attribute values
        let base_attrs = prod_attr_repo.find_all_attributes_by_base(base_product_arg)?;
        check_attributes_values_exist(base_attrs, attributes.clone())?;

        for attr_value in attributes {
            let attr = attr_repo.find(attr_value.attr_id)?;
            let attr = attr.ok_or(format_err!("Not found such attribute id : {}", attr_value.attr_id).context(Error::NotFound))?;
            let new_prod_attr = NewProdAttr::new(
                product_arg.id,
                base_product_arg,
                attr_value.attr_id,
                attr_value.value,
                attr.value_type,
                attr_value.meta_field,
            );
            prod_attr_repo.create(new_prod_attr)?;
        }
    }

    Ok(())
}

fn create_custom_attributes(
    values_repo: &CustomAttributesValuesRepo,
    custom_attributes: Option<Vec<NewCustomAttributeValuePayload>>,
    product_arg: &Product,
) -> Result<(), FailureError> {
    if let Some(custom_attributes) = custom_attributes {
        values_repo.delete_by_product(product_arg.id)?;

        let values = NewCustomAttributeValue::into_vec(product_arg.id, custom_attributes);
        values_repo.create(values)?;
    }

    Ok(())
}

fn check_attributes_values_exist(base_attrs: Vec<ProdAttr>, attributes: Vec<AttrValue>) -> Result<(), FailureError> {
    let mut hash = HashMap::<ProductId, HashMap<i32, String>>::default();
    for attr in base_attrs {
        let mut prod_attrs = hash.entry(attr.prod_id).or_insert_with(HashMap::<i32, String>::default);
        prod_attrs.insert(attr.attr_id, attr.value);
    }

    let result = hash.into_iter().any(|(_, prod_attrs)| {
        attributes.iter().all(|attr| {
            if let Some(value) = prod_attrs.get(&attr.attr_id) {
                value == &attr.value
            } else {
                false
            }
        })
    });

    if result {
        Err(format_err!("Product with attributes {:?} already exists", attributes)
            .context(Error::Validate(
                validation_errors!({"attributes": ["attributes" => "Product with this attributes already exists"]}),
            )).into())
    } else {
        Ok(())
    }
}

fn check_vendor_code(stores_repo: &StoresRepo, store_id: StoreId, vendor_code: &str) -> Result<(), FailureError> {
    let vendor_code_exists = stores_repo
        .vendor_code_exists(store_id, vendor_code)?
        .ok_or(format_err!("Store with id {} not found.", store_id).context(Error::NotFound))?;

    if vendor_code_exists {
        Err(
            format_err!("Vendor code '{}' already exists for store with id {}.", vendor_code, store_id)
                .context(Error::Validate(
                    validation_errors!({"vendor_code": ["vendor_code" => "Vendor code already exists."]}),
                )).into(),
        )
    } else {
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;
    use std::time::SystemTime;

    use stq_static_resources::Currency;
    use stq_types::*;

    use tokio_core::reactor::Core;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    pub fn create_product(id: ProductId, base_product_id: BaseProductId) -> Product {
        Product {
            id: id,
            base_product_id: base_product_id,
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

    pub fn create_new_product_with_attributes(base_product_id: BaseProductId) -> NewProductWithAttributes {
        NewProductWithAttributes {
            product: create_new_product(base_product_id),
            attributes: vec![AttrValue {
                attr_id: 1,
                value: "String".to_string(),
                meta_field: None,
            }],
        }
    }

    pub fn create_new_product(base_product_id: BaseProductId) -> NewProductWithoutCurrency {
        NewProductWithoutCurrency {
            base_product_id: base_product_id,
            discount: None,
            photo_main: None,
            vendor_code: "vendor_code".to_string(),
            cashback: None,
            additional_photos: None,
            price: ProductPrice(0f64),
            pre_order: Some(false),
            pre_order_days: Some(0),
        }
    }

    pub fn create_update_product() -> UpdateProduct {
        UpdateProduct {
            discount: None,
            photo_main: None,
            vendor_code: None,
            cashback: None,
            additional_photos: None,
            price: None,
            currency: None,
            pre_order: None,
            pre_order_days: None,
        }
    }

    pub fn create_update_product_with_attributes() -> UpdateProductWithAttributes {
        UpdateProductWithAttributes {
            product: Some(create_update_product()),
            attributes: None,
            custom_attributes: None,
        }
    }

    #[test]
    fn test_get_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_product(ProductId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, ProductId(1));
    }

    #[test]
    fn test_list() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.list_products(1, 5);
        let result = core.run(work).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_create_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_product = create_new_product_with_attributes(MOCK_BASE_PRODUCT_ID);
        let work = service.create_product(new_product);
        let result = core.run(work).unwrap();
        assert_eq!(result.base_product_id, MOCK_BASE_PRODUCT_ID);
    }

    #[test]
    fn test_update_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_product = create_update_product_with_attributes();
        let work = service.update_product(ProductId(1), new_product);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, ProductId(1));
        assert_eq!(result.base_product_id, MOCK_BASE_PRODUCT_ID);
    }

    #[test]
    fn test_deactivate_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.deactivate_product(ProductId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.id, ProductId(1));
        assert_eq!(result.is_active, false);
    }

}
