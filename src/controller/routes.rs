use stq_router::RouteParser;
use stq_types::*;

/// List of all routes with params for the app
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Healthcheck,
    Attributes,
    Attribute(AttributeId),
    BaseProducts,
    BaseProductsCount,
    BaseProductWithVariants,
    BaseProductsSearch,
    BaseProductsAutoComplete,
    BaseProductsMostViewed,
    BaseProductsMostDiscount,
    BaseProductsSearchFiltersPrice,
    BaseProductsSearchFiltersCategory,
    BaseProductsSearchFiltersAttributes,
    BaseProductsSearchFiltersCount,
    BaseProduct(BaseProductId),
    BaseProductWithViewsUpdate(BaseProductId),
    BaseProductByProduct(ProductId),
    BaseProductWithVariant(BaseProductId),
    BaseProductCustomAttributes(BaseProductId),
    BaseProductPublish,
    BaseProductDraft,
    Categories,
    Category(CategoryId),
    CategoryAttrs,
    CategoryAttr(CategoryId),
    CurrencyExchange,
    CustomAttributes,
    CustomAttribute(CustomAttributeId),
    ModeratorProductComments,
    ModeratorBaseProductComment(BaseProductId),
    ModeratorBaseProductSearch,
    ModeratorStoreComments,
    ModeratorStoreComment(StoreId),
    ModeratorStoreSearch,
    Products,
    ProductStoreId,
    Product(ProductId),
    ProductAttributes(ProductId),
    ProductsByBaseProduct(BaseProductId),
    SellerProductPrice(ProductId),
    Stores,
    StoresSearch,
    StoresAutoComplete,
    StoresSearchFiltersCount,
    StoresSearchFiltersCountry,
    StoresSearchFiltersCategory,
    StoresCart,
    StoresSlugExists,
    Store(StoreId),
    StoreCount,
    StoreByUser(UserId),
    StoreProducts(StoreId),
    StoreProductsCount(StoreId),
    StorePublish(StoreId),
    StoreDraft(StoreId),
    Roles,
    RoleById { id: RoleId },
    RolesByUserId { user_id: UserId },
    WizardStores,
}

pub fn create_route_parser() -> RouteParser<Route> {
    let mut router = RouteParser::default();

    // Healthcheck
    router.add_route(r"^/healthcheck$", || Route::Healthcheck);

    // Stores Routes
    router.add_route(r"^/stores$", || Route::Stores);

    // Stores/:id route
    router.add_route_with_params(r"^/stores/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(StoreId)
            .map(Route::Store)
    });

    // Stores/by_user_id/:id route
    router.add_route_with_params(r"^/stores/by_user_id/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(UserId)
            .map(Route::StoreByUser)
    });

    // Stores/:id/products route
    router.add_route_with_params(r"^/stores/(\d+)/products$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(StoreId)
            .map(Route::StoreProducts)
    });

    // Stores/:id/products/count route
    router.add_route_with_params(r"^/stores/(\d+)/products/count$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(StoreId)
            .map(Route::StoreProductsCount)
    });

    // Stores count route
    router.add_route(r"^/stores/count$", || Route::StoreCount);

    // Stores Cart route
    router.add_route(r"^/stores/cart$", || Route::StoresCart);

    // Stores Slug exists
    router.add_route(r"^/stores/slug_exists$", || Route::StoresSlugExists);

    // Stores Search route
    router.add_route(r"^/stores/search$", || Route::StoresSearch);

    // Stores Search filter count route
    router.add_route(r"^/stores/search/filters/count$", || Route::StoresSearchFiltersCount);

    // Stores Search filter country route
    router.add_route(r"^/stores/search/filters/country$", || Route::StoresSearchFiltersCountry);

    // Stores Search filter  route
    router.add_route(r"^/stores/search/filters/category$", || Route::StoresSearchFiltersCategory);

    // Stores auto complete route
    router.add_route(r"^/stores/auto_complete$", || Route::StoresAutoComplete);

    // Products Routes
    router.add_route(r"^/products$", || Route::Products);

    // Product Store Id Routes
    router.add_route(r"^/products/store_id$", || Route::ProductStoreId);

    // Products/:id route
    router.add_route_with_params(r"^/products/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(ProductId)
            .map(Route::Product)
    });

    // Products/:id route
    router.add_route_with_params(r"^/products/(\d+)/seller_price$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(ProductId)
            .map(Route::SellerProductPrice)
    });

    // Products/by_base_product/:id route
    router.add_route_with_params(r"^/products/by_base_product/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(BaseProductId)
            .map(Route::ProductsByBaseProduct)
    });

    // Products/:id/attributes route
    router.add_route_with_params(r"^/products/(\d+)/attributes$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(ProductId)
            .map(Route::ProductAttributes)
    });

    // Base products routes
    router.add_route(r"^/base_products$", || Route::BaseProducts);

    // Base products/:id route
    router.add_route_with_params(r"^/base_products/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(BaseProductId)
            .map(Route::BaseProduct)
    });

    // Base products/:id/custom_attributes route
    router.add_route_with_params(r"^/base_products/(\d+)/custom_attributes$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(BaseProductId)
            .map(Route::BaseProductCustomAttributes)
    });

    // Base products/:id/update_view route
    router.add_route_with_params(r"^/base_products/(\d+)/update_view$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(BaseProductId)
            .map(Route::BaseProductWithViewsUpdate)
    });

    // Base products count route
    router.add_route(r"^/base_products/count$", || Route::BaseProductsCount);

    // Base products with variants routes
    router.add_route(r"^/base_products/with_variants$", || Route::BaseProductWithVariants);

    // Base products/:id/with_variants route
    router.add_route_with_params(r"^/base_products/(\d+)/with_variants$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(BaseProductId)
            .map(Route::BaseProductWithVariant)
    });

    // base_products/by_product/:id route
    router.add_route_with_params(r"^/base_products/by_product/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(ProductId)
            .map(Route::BaseProductByProduct)
    });

    // BaseProducts Search route
    router.add_route(r"^/base_products/search$", || Route::BaseProductsSearch);

    // BaseProducts auto complete route
    router.add_route(r"^/base_products/auto_complete$", || Route::BaseProductsAutoComplete);

    // BaseProducts with most discount
    router.add_route(r"^/base_products/most_discount$", || Route::BaseProductsMostDiscount);

    // BaseProducts with most viewed
    router.add_route(r"^/base_products/most_viewed$", || Route::BaseProductsMostViewed);

    // BaseProducts search filters price route
    router.add_route(r"^/base_products/search/filters/price$", || Route::BaseProductsSearchFiltersPrice);

    // BaseProducts search filters category route
    router.add_route(r"^/base_products/search/filters/category$", || {
        Route::BaseProductsSearchFiltersCategory
    });

    // BaseProducts search filters attribute route
    router.add_route(r"^/base_products/search/filters/attributes$", || {
        Route::BaseProductsSearchFiltersAttributes
    });

    // BaseProducts search filters count route
    router.add_route(r"^/base_products/search/filters/count$", || Route::BaseProductsSearchFiltersCount);

    // Attributes Routes
    router.add_route(r"^/attributes$", || Route::Attributes);

    // CustomAttributes Routes
    router.add_route(r"^/custom_attributes$", || Route::CustomAttributes);

    // Attributes/:id route
    router.add_route_with_params(r"^/custom_attributes/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(CustomAttributeId)
            .map(Route::CustomAttribute)
    });

    // Attributes/:id route
    router.add_route_with_params(r"^/attributes/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(AttributeId)
            .map(Route::Attribute)
    });

    // Categories Routes
    router.add_route(r"^/categories$", || Route::Categories);

    // Categories/:id route
    router.add_route_with_params(r"^/categories/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<CategoryId>().ok())
            .map(Route::Category)
    });

    // Categories Attributes Routes
    router.add_route(r"^/categories/attributes$", || Route::CategoryAttrs);

    // Categories Attributes/:id route
    router.add_route_with_params(r"^/categories/(\d+)/attributes$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<CategoryId>().ok())
            .map(Route::CategoryAttr)
    });

    // Currency exchange Routes
    router.add_route(r"^/currency_exchange$", || Route::CurrencyExchange);

    // Wizard store Routes
    router.add_route(r"^/wizard_stores$", || Route::WizardStores);

    // Moderator Product Comments Routes
    router.add_route(r"^/moderator_product_comments$", || Route::ModeratorProductComments);

    // Moderator Product Comment/:base_product_id Route
    router.add_route_with_params(r"^/moderator_product_comments/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(BaseProductId)
            .map(Route::ModeratorBaseProductComment)
    });

    // Moderator Store Comments Routes
    router.add_route(r"^/moderator_store_comments$", || Route::ModeratorStoreComments);

    // Moderator Store Comment/:store_id Route
    router.add_route_with_params(r"^/moderator_store_comments/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(StoreId)
            .map(Route::ModeratorStoreComment)
    });

    // Moderator Store search
    router.add_route(r"^/stores/moderator_search$", || Route::ModeratorStoreSearch);

    // Stores/:id/publish route
    router.add_route_with_params(r"^/stores/(\d+)/publish$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(StoreId)
            .map(Route::StorePublish)
    });

    // Stores/:id/draft route
    router.add_route_with_params(r"^/stores/(\d+)/draft$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(StoreId)
            .map(Route::StoreDraft)
    });

    // Moderator Base Product search
    router.add_route(r"^/base_products/moderator_search$", || Route::ModeratorBaseProductSearch);

    // BaseProducts/publish route
    router.add_route(r"^/base_products/publish$", || Route::BaseProductPublish);

    // BaseProducts/draft route
    router.add_route(r"^/base_products/draft$", || Route::BaseProductDraft);

    router.add_route(r"^/roles$", || Route::Roles);
    router.add_route_with_params(r"^/roles/by-user-id/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|user_id| Route::RolesByUserId { user_id })
    });
    router.add_route_with_params(r"^/roles/by-id/([a-zA-Z0-9-]+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|id| Route::RoleById { id })
    });

    router
}
