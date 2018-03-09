use stq_router::RouteParser;

/// List of all routes with params for the app
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Healthcheck,
    Stores,
    Store(i32),
    StoresSearch,
    StoresAutoComplete,
    Products,
    Product(i32),
    ProductsSearch,
    ProductsAutoComplete,
    UserRoles,
    UserRole(i32),
    Attributes,
    Attribute(i32),
    Categories,
    Category(i32),
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

    // Stores Search route
    router.add_route(r"^/stores/search$", || Route::StoresSearch);

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

    // Products Search route
    router.add_route(r"^/products/search$", || Route::ProductsSearch);

    // Products auto complete route
    router.add_route(r"^/stores/auto_complete$", || Route::ProductsAutoComplete);

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

    // User_roles/:id route
    router.add_route_with_params(r"^/attributes/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|attribute_id| Route::Attribute(attribute_id))
    });

    // Categories Routes
    router.add_route(r"^/categories$", || Route::Categories);

    // User_roles/:id route
    router.add_route_with_params(r"^/categories/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|category_id| Route::Category(category_id))
    });

    router
}
