//! Products Services, presents CRUD operations with product
use std::collections::{HashMap, HashSet};

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use r2d2::ManageConnection;

use stq_static_resources::currency_type::CurrencyType;
use stq_static_resources::Currency;
use stq_types::{AttributeId, AttributeValueCode, BaseProductId, ExchangeRate, ProductId, ProductPrice, ProductSellerPrice, StoreId};

use super::types::ServiceFuture;
use errors::Error;
use models::*;
use repos::{
    AttributeValuesRepo, AttributesRepo, BaseProductsSearchTerms, CurrencyExchangeRepo, CustomAttributesRepo, ProductAttrsRepo,
    ProductFilters, ProductsRepo, RepoResult, ReposFactory, StoresRepo,
};
use services::check_can_update_by_status;
use services::Service;

pub trait ProductsService {
    /// Returns product by ID
    fn get_product(&self, product_id: ProductId) -> ServiceFuture<Option<Product>>;
    /// Returns products by IDs
    fn get_products(&self, product_ids: Vec<ProductId>) -> ServiceFuture<Vec<Product>>;
    /// Return product by ID
    fn get_product_without_filters(&self, product_id: ProductId) -> ServiceFuture<Option<Product>>;
    /// Returns product seller price by ID
    fn get_product_seller_price(&self, product_id: ProductId) -> ServiceFuture<Option<ProductSellerPrice>>;
    /// Returns store_id by ID
    fn get_product_store_id(&self, product_id: ProductId, visibility: Option<Visibility>) -> ServiceFuture<Option<StoreId>>;
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
    /// Get by store id
    fn find_products_with_store_id(&self, store_id: StoreId) -> ServiceFuture<Vec<Product>>;
    /// Get by base product id
    fn find_products_attributes(&self, product_id: ProductId) -> ServiceFuture<Vec<AttrValue>>;
    /// Check that you can update product
    fn validate_update_product(&self, product_id: ProductId) -> ServiceFuture<bool>;
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
        let fiat_currency = self.dynamic_context.fiat_currency;

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let raw_product = products_repo.find(product_id)?;
                if let Some(raw_product) = raw_product {
                    let customer_price = calculate_product_customer_price(&*currency_exchange, &raw_product, currency, fiat_currency)?;
                    let result_product = Product::new(raw_product, customer_price);

                    Ok(Some(result_product))
                } else {
                    Ok(None)
                }
            }
            .map_err(|e: FailureError| e.context("Service Product, get_product endpoint error occurred.").into())
        })
    }

    /// Returns products by IDs
    fn get_products(&self, product_ids: Vec<ProductId>) -> ServiceFuture<Vec<Product>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let currency = self.dynamic_context.currency;
        let fiat_currency = self.dynamic_context.fiat_currency;

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let raw_products = products_repo.find_many(product_ids)?;
                let products: Result<Vec<_>, _> = raw_products
                    .into_iter()
                    .map(|raw_product| {
                        calculate_product_customer_price(&*currency_exchange, &raw_product, currency, fiat_currency)
                            .map(|customer_price| Product::new(raw_product, customer_price))
                    })
                    .collect();
                products
            }
            .map_err(|e: FailureError| e.context("Service Product, get_products endpoint error occurred.").into())
        })
    }

    /// Return product
    fn get_product_without_filters(&self, product_id: ProductId) -> ServiceFuture<Option<Product>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let currency = self.dynamic_context.currency;
        let fiat_currency = self.dynamic_context.fiat_currency;

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let product_filters = ProductFilters::default();

                let raw_product = products_repo.find_by_filters(product_id, product_filters)?;
                if let Some(raw_product) = raw_product {
                    let customer_price = calculate_product_customer_price(&*currency_exchange, &raw_product, currency, fiat_currency)?;
                    let result_product = Product::new(raw_product, customer_price);

                    Ok(Some(result_product))
                } else {
                    Ok(None)
                }
            }
            .map_err(|e: FailureError| e.context("Service Product, get_order_product endpoint error occurred.").into())
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
                        discount: product.discount,
                    }))
                } else {
                    Ok(None)
                }
            }
            .map_err(|e: FailureError| e.context("Service Product, get endpoint error occurred.").into())
        })
    }

    /// Returns store_id by ID
    fn get_product_store_id(&self, product_id: ProductId, visibility: Option<Visibility>) -> ServiceFuture<Option<StoreId>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let visibility = visibility.unwrap_or(Visibility::Published);

        debug!(
            "Get product store id by product id = {:?} with visibility = {:?}",
            product_id, visibility
        );

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                let product = products_repo.find(product_id)?;
                if let Some(product) = product {
                    let base_product = base_products_repo.find(product.base_product_id, visibility)?;
                    if let Some(base_product) = base_product {
                        Ok(Some(base_product.store_id))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            .map_err(|e: FailureError| e.context("Service Product, get_store_id endpoint error occurred.").into())
        })
    }

    /// Deactivates specific product
    fn deactivate_product(&self, product_id: ProductId) -> ServiceFuture<Product> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
            conn.transaction::<Product, FailureError, _>(move || {
                let result_product = products_repo.deactivate(product_id)?;
                prod_attr_repo.delete_all_attributes(result_product.id)?;

                Ok(result_product.into())
            })
            .map_err(|e| e.context("Service Product, deactivate endpoint error occurred.").into())
        })
    }

    /// Lists users limited by `from` and `count` parameters
    fn list_products(&self, from: i32, count: i32) -> ServiceFuture<Vec<Product>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let currency = self.dynamic_context.currency;
        let fiat_currency = self.dynamic_context.fiat_currency;

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let raw_products = products_repo.list(from, count)?;

                let products = raw_products
                    .into_iter()
                    .map(|raw_product| {
                        calculate_product_customer_price(&*currency_exchange, &raw_product, currency, fiat_currency)
                            .and_then(|customer_price| Ok(Product::new(raw_product, customer_price)))
                    })
                    .collect::<RepoResult<Vec<Product>>>();

                products
            }
            .map_err(|e: FailureError| e.context("Service Product, list endpoint error occurred.").into())
        })
    }

    /// Creates new product
    fn create_product(&self, payload: NewProductWithAttributes) -> ServiceFuture<Product> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
            let attr_repo = repo_factory.create_attributes_repo(&*conn, user_id);
            let attribute_values_repo = repo_factory.create_attribute_values_repo(&*conn, user_id);
            let custom_attributes_repo = repo_factory.create_custom_attributes_repo(&*conn, user_id);
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);

            let NewProductWithAttributes { mut product, attributes } = payload;

            conn.transaction::<Product, FailureError, _>(move || {
                // fill currency id taken from base_product first
                let base_product_id = product
                    .base_product_id
                    .ok_or(format_err!("Base product id not set.").context(Error::NotFound))?;

                let base_product = base_products_repo.find(base_product_id, Visibility::Active)?;
                let base_product =
                    base_product.ok_or(format_err!("Base product with id {} not found.", base_product_id).context(Error::NotFound))?;

                product.base_product_id = Some(base_product_id);

                check_vendor_code(&*stores_repo, base_product.store_id, &product.vendor_code)?;

                let result_product: Product = products_repo.create((product, base_product.currency).into())?.into();

                create_product_attributes_values(
                    &*products_repo,
                    &*prod_attr_repo,
                    &*attr_repo,
                    &*custom_attributes_repo,
                    &*attribute_values_repo,
                    &result_product.product,
                    base_product.id,
                    attributes,
                )?;

                Ok(result_product)
            })
            .map_err(|e| e.context("Service Product, create endpoint error occurred.").into())
        })
    }

    /// Updates specific product
    fn update_product(&self, product_id: ProductId, payload: UpdateProductWithAttributes) -> ServiceFuture<Product> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
            let attr_repo = repo_factory.create_attributes_repo(&*conn, user_id);
            let attribute_values_repo = repo_factory.create_attribute_values_repo(&*conn, user_id);
            let custom_attributes_repo = repo_factory.create_custom_attributes_repo(&*conn, user_id);
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);

            conn.transaction::<Product, FailureError, _>(move || {
                let original_product = products_repo
                    .find(product_id)?
                    .ok_or(format_err!("Not found such product id: {}", product_id).context(Error::NotFound))?;

                let product = if let Some(product) = payload.product {
                    if let Some(vendor_code) = &product.vendor_code {
                        let BaseProduct { store_id, .. } = base_products_repo
                            .find(original_product.base_product_id, Visibility::Active)?
                            .ok_or(
                            format_err!("Base product with id {} not found.", original_product.base_product_id).context(Error::NotFound),
                        )?;

                        if *original_product.vendor_code.as_str() != *vendor_code {
                            check_vendor_code(&*stores_repo, store_id, &vendor_code)?;
                        }
                    };

                    products_repo.update(product_id, product)?
                } else {
                    original_product
                };

                let result_product: Product = product.into();

                if let Some(attributes) = payload.attributes {
                    create_product_attributes_values(
                        &*products_repo,
                        &*prod_attr_repo,
                        &*attr_repo,
                        &*custom_attributes_repo,
                        &*attribute_values_repo,
                        &result_product.product,
                        result_product.product.base_product_id,
                        attributes,
                    )?;
                }

                Ok(result_product)
            })
            .map_err(|e| e.context("Service Product, update endpoint error occurred.").into())
        })
    }

    /// Get by base product id
    fn find_products_with_base_id(&self, base_product_id: BaseProductId) -> ServiceFuture<Vec<Product>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let currency = self.dynamic_context.currency;
        let fiat_currency = self.dynamic_context.fiat_currency;

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let raw_products = products_repo.find_with_base_id(base_product_id)?;

                let result_products = raw_products
                    .into_iter()
                    .map(|raw_product| {
                        calculate_product_customer_price(&*currency_exchange, &raw_product, currency, fiat_currency)
                            .and_then(|customer_price| Ok(Product::new(raw_product, customer_price)))
                    })
                    .collect::<RepoResult<Vec<Product>>>();

                result_products
            }
            .map_err(|e: FailureError| e.context("Service Product, find_with_base_id endpoint error occurred.").into())
        })
    }

    fn find_products_with_store_id(&self, store_id: StoreId) -> ServiceFuture<Vec<Product>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let currency = self.dynamic_context.currency;
        let fiat_currency = self.dynamic_context.fiat_currency;

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);

                let base_product_ids = base_products_repo
                    .search(BaseProductsSearchTerms {
                        store_id: Some(store_id),
                        ..Default::default()
                    })?
                    .into_iter()
                    .map(|p| p.id)
                    .collect();

                let raw_products = products_repo.find_with_base_ids(base_product_ids)?;

                let result_products = raw_products
                    .into_iter()
                    .map(|raw_product| {
                        calculate_product_customer_price(&*currency_exchange, &raw_product, currency, fiat_currency)
                            .and_then(|customer_price| Ok(Product::new(raw_product, customer_price)))
                    })
                    .collect::<RepoResult<Vec<Product>>>();

                result_products
            }
            .map_err(|e: FailureError| {
                e.context("Service Product, find_products_with_store_id endpoint error occurred.")
                    .into()
            })
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
                .and_then(|pr_attrs| {
                    let attr_values = pr_attrs
                        .into_iter()
                        .map(|pr_attr| AttrValue {
                            attr_id: pr_attr.attr_id,
                            attr_value_id: pr_attr.attr_value_id,
                            value: pr_attr.value,
                            meta_field: pr_attr.meta_field,
                        })
                        .collect();

                    Ok(attr_values)
                })
                .map_err(|e| e.context("Service Product, find_attributes endpoint error occurred.").into())
        })
    }

    /// Check that you can update product
    fn validate_update_product(&self, product_id: ProductId) -> ServiceFuture<bool> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        info!("Check update product: {}", product_id);

        self.spawn_on_pool(move |conn| {
            let products_repo = repo_factory.create_product_repo(&conn, user_id);
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);

            let product = products_repo
                .find(product_id)?
                .ok_or(format_err!("Not found such product id: {}", product_id).context(Error::NotFound))?;

            let base_product = base_products_repo.find(product.base_product_id, Visibility::Active)?;

            let current_status = match base_product {
                Some(value) => value.status,
                None => return Err(Error::NotFound.into()),
            };

            Ok(check_can_update_by_status(current_status))
        })
    }
}

pub fn calculate_product_customer_price(
    currency_exchange: &CurrencyExchangeRepo,
    product: &RawProduct,
    crypto_currency: Currency,
    fiat_currency: Currency,
) -> RepoResult<CustomerPrice> {
    Ok(calculate_customer_price(
        product,
        &currency_exchange.get_exchange_for_currency(product.currency)?,
        crypto_currency,
        fiat_currency,
    ))
}

pub fn calculate_customer_price(
    product: &RawProduct,
    product_currency_map: &Option<HashMap<Currency, ExchangeRate>>,
    crypto_currency: Currency,
    fiat_currency: Currency,
) -> CustomerPrice {
    let header_currency = match product.currency.currency_type() {
        CurrencyType::Crypto => crypto_currency,
        CurrencyType::Fiat => fiat_currency,
    };

    if let Some(currency_map) = product_currency_map {
        let price = ProductPrice(product.price.0 / currency_map.get(&header_currency).map(|c| c.0).unwrap_or(1.0));
        CustomerPrice {
            price,
            currency: header_currency,
        }
    } else {
        // When no currency convert how seller price
        CustomerPrice {
            price: product.price,
            currency: header_currency,
        }
    }
}

pub fn create_product_attributes_values(
    products_repo: &ProductsRepo,
    prod_attr_repo: &ProductAttrsRepo,
    attr_repo: &AttributesRepo,
    custom_attributes_repo: &CustomAttributesRepo,
    attribute_values_repo: &AttributeValuesRepo,
    product_arg: &RawProduct,
    base_product_arg: BaseProductId,
    attribute_values: Vec<AttrValue>,
) -> Result<(), FailureError> {
    // deleting old attributes for this product
    prod_attr_repo.delete_all_attributes(product_arg.id)?;
    let attribute_values = fill_attr_value(attribute_values_repo, attribute_values)?;
    check_products_attribute_values_are_unique(prod_attr_repo, custom_attributes_repo, base_product_arg, attribute_values.clone())?;
    update_custom_attributes(products_repo, custom_attributes_repo, base_product_arg, attribute_values.clone())?;

    for attr_value in attribute_values {
        let attr = attr_repo.find(attr_value.attr_id)?;
        let attr = attr.ok_or(format_err!("Not found such attribute id : {}", attr_value.attr_id).context(Error::NotFound))?;
        let new_prod_attr = NewProdAttr::new(
            product_arg.id,
            base_product_arg,
            attr_value.attr_id,
            attr_value.value,
            attr.value_type,
            attr_value.meta_field,
            attr_value.attr_value_id,
        );
        prod_attr_repo.create(new_prod_attr)?;
    }

    Ok(())
}

fn fill_attr_value(attribute_values_repo: &AttributeValuesRepo, attribute_values: Vec<AttrValue>) -> Result<Vec<AttrValue>, FailureError> {
    attribute_values
        .into_iter()
        .map(|attr_value| {
            let attribute_value = attribute_values_repo
                .find(attr_value.attr_id.clone(), attr_value.value.clone())?
                .ok_or(format_err!(
                    "Attribute value for {} with code {} not found",
                    attr_value.attr_id,
                    attr_value.value
                ))?;
            Ok(AttrValue {
                attr_value_id: Some(attribute_value.id),
                ..attr_value
            })
        })
        .collect()
}

fn check_products_attribute_values_are_unique(
    prod_attr_repo: &ProductAttrsRepo,
    custom_attributes_repo: &CustomAttributesRepo,
    base_product_arg: BaseProductId,
    new_product_attributes: Vec<AttrValue>,
) -> Result<(), FailureError> {
    // searching for existed product with such attribute values
    let base_attrs = prod_attr_repo.find_all_attributes_by_base(base_product_arg)?;
    // get available attributes
    let available_attributes = custom_attributes_repo
        .find_all_attributes(base_product_arg)?
        .into_iter()
        .map(|v| (v.attribute_id, String::default().into()))
        .collect::<HashMap<AttributeId, AttributeValueCode>>();

    let mut hash = HashMap::<ProductId, HashMap<AttributeId, AttributeValueCode>>::default();
    for attr in base_attrs {
        let mut prod_attrs = hash.entry(attr.prod_id).or_insert_with(|| available_attributes.clone());
        prod_attrs.insert(attr.attr_id, attr.value);
    }

    let result = hash.into_iter().any(|(_, prod_attrs)| {
        new_product_attributes.iter().all(|attr| {
            if let Some(value) = prod_attrs.get(&attr.attr_id) {
                value == &attr.value
            } else {
                false
            }
        })
    });

    if result {
        Err(format_err!("Product with attributes {:?} already exists", new_product_attributes)
            .context(Error::Validate(
                validation_errors!({"attributes": ["attributes" => "Product with this attributes already exists"]}),
            ))
            .into())
    } else {
        Ok(())
    }
}

fn update_custom_attributes(
    products_repo: &ProductsRepo,
    custom_attributes_repo: &CustomAttributesRepo,
    base_product_id: BaseProductId,
    new_product_attributes: Vec<AttrValue>,
) -> Result<(), FailureError> {
    let all_custom_attributes = custom_attributes_repo.find_all_attributes(base_product_id)?;
    let old_custom_attributes: HashSet<AttributeId> = all_custom_attributes.iter().map(|ca| ca.attribute_id).collect();
    let new_custom_attributes: HashSet<AttributeId> = new_product_attributes.iter().map(|ca| ca.attr_id).collect();
    if old_custom_attributes != new_custom_attributes {
        let variants_count = products_repo.find_with_base_id(base_product_id)?.len();
        if variants_count > 1 {
            return Err(
                format_err!("Can not update attributes: base product has {} active variants", variants_count)
                    .context(Error::Validate(
                        validation_errors!({"attributes": ["attributes" => "Too many variants."]}),
                    ))
                    .into(),
            );
        } else {
            custom_attributes_repo.delete_all(base_product_id)?;
            for new_product_attribute in new_product_attributes {
                custom_attributes_repo.create(NewCustomAttribute {
                    base_product_id,
                    attribute_id: new_product_attribute.attr_id,
                })?;
            }
        }
    }
    Ok(())
}

pub fn check_vendor_code(stores_repo: &StoresRepo, store_id: StoreId, vendor_code: &str) -> Result<(), FailureError> {
    let vendor_code_exists = stores_repo
        .vendor_code_exists(store_id, vendor_code)?
        .ok_or(format_err!("Store with id {} not found.", store_id).context(Error::NotFound))?;

    if vendor_code_exists {
        Err(
            format_err!("Vendor code '{}' already exists for store with id {}.", vendor_code, store_id)
                .context(Error::Validate(
                    validation_errors!({"vendor_code": ["vendor_code" => "Vendor code already exists."]}),
                ))
                .into(),
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
    use uuid::Uuid;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

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
            uuid: Uuid::new_v4(),
        }
    }

    pub fn create_new_product_with_attributes(base_product_id: BaseProductId) -> NewProductWithAttributes {
        NewProductWithAttributes {
            product: create_new_product(base_product_id),
            attributes: vec![AttrValue {
                attr_id: AttributeId(1),
                value: AttributeValueCode("String".to_string()),
                meta_field: None,
                attr_value_id: None,
            }],
        }
    }

    pub fn create_new_product(base_product_id: BaseProductId) -> NewProductWithoutCurrency {
        NewProductWithoutCurrency {
            base_product_id: Some(base_product_id),
            discount: None,
            photo_main: None,
            vendor_code: "vendor_code".to_string(),
            cashback: None,
            additional_photos: None,
            price: ProductPrice(0f64),
            pre_order: Some(false),
            pre_order_days: Some(0),
            uuid: Uuid::new_v4(),
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
        }
    }

    #[test]
    fn test_get_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_product(ProductId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().product.id, ProductId(1));
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
        assert_eq!(result.product.base_product_id, MOCK_BASE_PRODUCT_ID);
    }

    #[test]
    fn test_update_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_product = create_update_product_with_attributes();
        let work = service.update_product(ProductId(1), new_product);
        let result = core.run(work).unwrap();
        assert_eq!(result.product.id, ProductId(1));
        assert_eq!(result.product.base_product_id, MOCK_BASE_PRODUCT_ID);
    }

    #[test]
    fn test_deactivate_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.deactivate_product(ProductId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.product.id, ProductId(1));
        assert_eq!(result.product.is_active, false);
    }

}
