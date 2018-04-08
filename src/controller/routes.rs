use stq_router::RouteParser;

/// List of all routes with params for the app
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Healthcheck,
    Stores,
    StoresSearch,
    StoresSearchCount,
    StoresAutoComplete,
    Store(i32),
    StoreProducts(i32),
    StoreProductsCount(i32),
    Products,
    ProductsSearch,
    ProductsSearchInCategory,
    ProductsSearchWithoutCategory,
    ProductsAutoComplete,
    ProductsMostViewed,
    ProductsMostDiscount,
    ProductsSearchFilters,
    ProductsSearchInCategoryFilters,
    ProductsSearchWithoutCategoryFilters,
    Product(i32),
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
            .map(|store_id| Route::Store(store_id))
    });

    // Stores/:id/products route
    router.add_route_with_params(r"^/stores/(\d+)/products$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|store_id| Route::StoreProducts(store_id))
    });

    // Stores/:id/products/count route
    router.add_route_with_params(r"^/stores/(\d+)/products/count$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|store_id| Route::StoreProductsCount(store_id))
    });

    // Stores Search route
    router.add_route(r"^/stores/search$", || Route::StoresSearch);

    // Stores Search count route
    router.add_route(r"^/stores/search/count$", || Route::StoresSearchCount);

    // Stores auto complete route
    router.add_route(r"^/stores/auto_complete$", || Route::StoresAutoComplete);

    // Products Routes
    router.add_route(r"^/products$", || Route::Products);

    // Products/:id route
    router.add_route_with_params(r"^/products/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|product_id| Route::Product(product_id))
    });

    // Base products routes
    router.add_route(r"^/base_products$", || Route::BaseProducts);

    // Base products/:id route
    router.add_route_with_params(r"^/base_products/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|product_id| Route::BaseProduct(product_id))
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
            .map(|product_id| Route::BaseProductWithVariant(product_id))
    });

    // Products Search route
    router.add_route(r"^/products/search$", || Route::ProductsSearch);

    // Products Search without category route
    router.add_route(r"^/products/search/without_category$", || {
        Route::ProductsSearchWithoutCategory
    });

    // Products Search in category route
    router.add_route(r"^/products/search/in_category$", || {
        Route::ProductsSearchInCategory
    });

    // Products auto complete route
    router.add_route(r"^/products/auto_complete$", || Route::ProductsAutoComplete);

    // Products with most discount
    router.add_route(r"^/products/most_discount$", || Route::ProductsMostDiscount);

    // Products with most viewed
    router.add_route(r"^/products/most_viewed$", || Route::ProductsMostViewed);

    // Products search filters route
    router.add_route(r"^/products/search/filters", || {
        Route::ProductsSearchFilters
    });

    // Products search filters route
    router.add_route(r"^/products/search/without_category/filters", || {
        Route::ProductsSearchWithoutCategoryFilters
    });

    // Products search filters route
    router.add_route(r"^/products/search/in_category/filters", || {
        Route::ProductsSearchInCategoryFilters
    });

    // User_roles Routes
    router.add_route(r"^/user_roles$", || Route::UserRoles);

    // User_roles/:id route
    router.add_route_with_params(r"^/user_roles/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|user_role_id| Route::UserRole(user_role_id))
    });

    // Attributes Routes
    router.add_route(r"^/attributes$", || Route::Attributes);

    // Attributes/:id route
    router.add_route_with_params(r"^/attributes/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|attribute_id| Route::Attribute(attribute_id))
    });

    // Categories Routes
    router.add_route(r"^/categories$", || Route::Categories);

    // Categories/:id route
    router.add_route_with_params(r"^/categories/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|category_id| Route::Category(category_id))
    });

    // roles/default/:id route
    router.add_route_with_params(r"^/roles/default/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|user_id| Route::DefaultRole(user_id))
    });

    // Categories Attributes Routes
    router.add_route(r"^/categories/attributes$", || Route::CategoryAttrs);

    // Categories Attributes/:id route
    router.add_route_with_params(r"^/categories/(\d+)/attributes$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|category_id| Route::CategoryAttr(category_id))
    });

    router
}
