use stq_router::RouteParser;

/// List of all routes with params for the app
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Healthcheck,
    Stores,
    StoresSearch,
    StoresSearchFiltersCount,
    StoresSearchFiltersCountry,
    StoresSearchFiltersCategory,
    StoresAutoComplete,
    Store(i32),
    StoreProducts(i32),
    StoreProductsCount(i32),
    Products,
    BaseProductsSearch,
    BaseProductsAutoComplete,
    BaseProductsMostViewed,
    BaseProductsMostDiscount,
    BaseProductsSearchFiltersPrice,
    BaseProductsSearchFiltersCategory,
    BaseProductsSearchFiltersAttributes,
    Product(i32),
    ProductAttributes(i32),
    ProductsByBaseProduct(i32),
    BaseProducts,
    BaseProductWithVariants,
    BaseProduct(i32),
    BaseProductWithVariant(i32),
    UserRoles,
    UserRole(i32),
    DefaultRole(i32),
    Attributes,
    Attribute(i32),
    Categories,
    Category(i32),
    CategoryAttrs,
    CategoryAttr(i32),
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
            .map(Route::Store)
    });

    // Stores/:id/products route
    router.add_route_with_params(r"^/stores/(\d+)/products$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(Route::StoreProducts)
    });

    // Stores/:id/products/count route
    router.add_route_with_params(r"^/stores/(\d+)/products/count$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(Route::StoreProductsCount)
    });

    // Stores Search route
    router.add_route(r"^/stores/search$", || Route::StoresSearch);

    // Stores Search filter count route
    router.add_route(r"^/stores/search/filters/count$", || {
        Route::StoresSearchFiltersCount
    });
    
    // Stores Search filter country route
    router.add_route(r"^/stores/search/filters/country$", || {
        Route::StoresSearchFiltersCountry
    });
    
    // Stores Search filter  route
    router.add_route(r"^/stores/search/filters/category$", || {
        Route::StoresSearchFiltersCategory
    });

    // Stores auto complete route
    router.add_route(r"^/stores/auto_complete$", || Route::StoresAutoComplete);

    // Products Routes
    router.add_route(r"^/products$", || Route::Products);

    // Products/:id route
    router.add_route_with_params(r"^/products/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(Route::Product)
    });

    // Products/by_base_product/:id route
    router.add_route_with_params(r"^/products/by_base_product/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|base_product_id| Route::ProductsByBaseProduct(base_product_id))
    });

    // Products/:id/attributes route
    router.add_route_with_params(r"^/products/(\d+)/attributes$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|product_id| Route::ProductAttributes(product_id))
    });

    // Base products routes
    router.add_route(r"^/base_products$", || Route::BaseProducts);

    // Base products/:id route
    router.add_route_with_params(r"^/base_products/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(Route::BaseProduct)
    });

    // Base products with variants routes
    router.add_route(r"^/base_products/with_variants$", || {
        Route::BaseProductWithVariants
    });

    // Base products/:id/with_variants route
    router.add_route_with_params(r"^/base_products/(\d+)/with_variants$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(Route::BaseProductWithVariant)
    });

    // BaseProducts Search route
    router.add_route(r"^/base_products/search$", || Route::BaseProductsSearch);

    // BaseProducts auto complete route
    router.add_route(r"^/base_products/auto_complete$", || {
        Route::BaseProductsAutoComplete
    });

    // BaseProducts with most discount
    router.add_route(r"^/base_products/most_discount$", || {
        Route::BaseProductsMostDiscount
    });

    // BaseProducts with most viewed
    router.add_route(r"^/base_products/most_viewed$", || {
        Route::BaseProductsMostViewed
    });

    // BaseProducts search filters price route
    router.add_route(r"^/base_products/search/filters/price$", || {
        Route::BaseProductsSearchFiltersPrice
    });

    // BaseProducts search filters category route
    router.add_route(r"^/base_products/search/filters/category", || {
        Route::BaseProductsSearchFiltersCategory
    });

    // BaseProducts search filters attribute route
    router.add_route(r"^/base_products/search/filters/attributes", || {
        Route::BaseProductsSearchFiltersAttributes
    });

    // User_roles Routes
    router.add_route(r"^/user_roles$", || Route::UserRoles);

    // User_roles/:id route
    router.add_route_with_params(r"^/user_roles/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(Route::UserRole)
    });

    // Attributes Routes
    router.add_route(r"^/attributes$", || Route::Attributes);

    // Attributes/:id route
    router.add_route_with_params(r"^/attributes/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(Route::Attribute)
    });

    // Categories Routes
    router.add_route(r"^/categories$", || Route::Categories);

    // Categories/:id route
    router.add_route_with_params(r"^/categories/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(Route::Category)
    });

    // roles/default/:id route
    router.add_route_with_params(r"^/roles/default/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(Route::DefaultRole)
    });

    // Categories Attributes Routes
    router.add_route(r"^/categories/attributes$", || Route::CategoryAttrs);

    // Categories Attributes/:id route
    router.add_route_with_params(r"^/categories/(\d+)/attributes$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(Route::CategoryAttr)
    });

    router
}
