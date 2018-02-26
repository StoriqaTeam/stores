use regex::Regex;

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
    UserRoles,
    UserRole(i32),
    Languages,
    Currencies
}

/// RouteParser class maps regex to type-safe list of routes, defined by `enum Route`
pub struct RouteParser {
    regex_and_converters: Vec<(Regex, Box<ParamsConverter>)>,
}

type ParamsConverter = Fn(Vec<&str>) -> Option<Route>;

impl RouteParser {
    /// Creates new Router
    /// #Examples
    ///
    /// ```
    /// use stores_lib::controller::routes::RouteParser;
    ///
    /// let router = RouteParser::new();
    /// ```
    pub fn new() -> Self {
        Self {
            regex_and_converters: Vec::new(),
        }
    }

    /// Adds mapping between regex and route
    /// #Examples
    ///
    /// ```
    /// use stores_lib::controller::routes::{RouteParser, Route};
    ///
    /// let mut router = RouteParser::new();
    /// router.add_route(r"^/stores$", Route::Stores);
    /// ```
    pub fn add_route(&mut self, regex_pattern: &str, route: Route) -> &Self {
        self.add_route_with_params(regex_pattern, move |_| Some(route.clone()));
        self
    }

    /// Adds mapping between regex and route with params
    /// converter is a function with argument being a set of regex matches (strings) for route params in regex
    /// this is needed if you want to convert params from strings to int or some other types
    ///
    /// #Examples
    ///
    /// ```
    /// use stores_lib::controller::routes::{RouteParser, Route};
    ///
    /// let mut router = RouteParser::new();
    /// router.add_route_with_params(r"^/stores/(\d+)$", |params| {
    ///     params.get(0)
    ///        .and_then(|string_id| string_id.parse::<i32>().ok())
    ///        .map(|user_id| Route::Store(user_id))
    /// });
    /// ```
    pub fn add_route_with_params<F>(&mut self, regex_pattern: &str, converter: F) -> &Self
    where
        F: Fn(Vec<&str>) -> Option<Route> + 'static,
    {
        let regex = Regex::new(regex_pattern).unwrap();
        self.regex_and_converters.push((regex, Box::new(converter)));
        self
    }

    /// Tests string router for matches
    /// Returns Some(route) if there's a match
    /// #Examples
    ///
    /// ```
    /// use stores_lib::controller::routes::*;
    ///
    /// let mut router = RouteParser::new();
    /// router.add_route(r"^/stores$", Route::Stores);
    /// let route = router.test("/stores").unwrap();
    /// assert_eq!(route, Route::Stores);
    /// ```
    pub fn test(&self, route: &str) -> Option<Route> {
        self.regex_and_converters
            .iter()
            .fold(None, |acc, ref regex_and_converter| {
                if acc.is_some() {
                    return acc;
                }
                RouteParser::get_matches(&regex_and_converter.0, route).and_then(|params| regex_and_converter.1(params))
            })
    }

    fn get_matches<'a>(regex: &Regex, string: &'a str) -> Option<Vec<&'a str>> {
        regex.captures(string).and_then(|captures| {
            captures
                .iter()
                .skip(1)
                .fold(Some(Vec::<&str>::new()), |mut maybe_acc, maybe_match| {
                    if let Some(ref mut acc) = maybe_acc {
                        if let Some(mtch) = maybe_match {
                            acc.push(mtch.as_str());
                        }
                    }
                    maybe_acc
                })
        })
    }
}

pub fn create_route_parser() -> RouteParser {
    let mut router = RouteParser::new();

    // Healthcheck
    router.add_route(r"^/healthcheck$", Route::Healthcheck);

    // Stores Routes
    router.add_route(r"^/stores$", Route::Stores);

    // Stores/:id route
    router.add_route_with_params(r"^/stores/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|store_id| Route::Store(store_id))
    });

    // Stores Search route
    router.add_route(r"^/stores/search$", Route::StoresSearch);

    // Stores Search route
    router.add_route(r"^/stores/auto_complete$", Route::StoresAutoComplete);

    // Products Routes
    router.add_route(r"^/products$", Route::Products);

    // Products/:id route
    router.add_route_with_params(r"^/products/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|store_id| Route::Product(store_id))
    });

    // User_roles Routes
    router.add_route(r"^/user_roles$", Route::UserRoles);

    // Languages Routes
    router.add_route(r"^/languages$", Route::Languages);

    // Currencies Routes
    router.add_route(r"^/currencies$", Route::Currencies);

    // User_roles/:id route
    router.add_route_with_params(r"^/user_roles/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|user_id| Route::UserRole(user_id))
    });

    router
}
