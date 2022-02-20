//! An http routing tool based on routes stored in yaml files.

use log::{debug, warn};
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::{collections::HashMap, vec::Vec};
mod routes;

#[derive(Debug, Clone)]
struct Parameter {
    value: String,
    requirement: Option<regex::Regex>,
}

#[derive(Debug)]
/// The object that keeps the routes collection.
///
/// This is the yaml file to be used in the following tests:
/// ```yaml
/// ## tests/routes.yaml
///
/// routes:
///   - home:
///       path: /
///       controller: home_controller::index
///       methods: get
///   - user:
///       path: /user/{id}
///       controller: user_controller::crud
///       middleware: user_middleware::test
///       methods: get, post, delete, put
///       requirements:
///         id: "^[0-9]+"
/// ```
pub struct Router {
    pub collection: Vec<Route>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
/// The object that stores the route information.
///
/// This is the yaml file to be used in the following tests:
/// ```yaml
/// ## tests/routes.yaml
///
/// routes:
///   - home:
///       path: /
///       controller: home_controller::index
///       methods: get
/// ```
pub struct Route {
    name: String,
    path: String,
    #[serde(skip_serializing, skip_deserializing)]
    path_split: Vec<Parameter>,
    methods: Vec<String>,
    controller: String,
    middleware: String,
    pub parameters: HashMap<String, String>,
    pub language: String,
}
impl Route {
    /// Creates and returns the `Route` instance given the route parameters.
    fn new(route_name: String, route_keys: &Mapping) -> Route {
        Route {
            name: route_name,
            path: Route::parse_path(route_keys),
            path_split: Route::get_path_split(route_keys),
            methods: Route::parse_methods(route_keys),
            controller: Route::parse_controller(route_keys),
            middleware: Route::parse_middleware(route_keys),
            parameters: HashMap::new(),
            language: Route::parse_language(route_keys),
        }
    }
    /// Returns the name of the route.
    ///
    /// # Examples
    /// ```
    /// # use kalgan_router::Router;
    /// # let router = Router::new("tests/routes.yaml");
    /// # let route = router.get_route("/", "get").unwrap();
    /// assert_eq!(route.get_name(), &"home".to_string());
    /// ```
    pub fn get_name(&self) -> &String {
        &self.name
    }
    /// Returns the uri of the route.
    ///
    /// # Examples
    /// ```
    /// # use kalgan_router::Router;
    /// # let router = Router::new("tests/routes.yaml");
    /// # let route = router.get_route("/", "get").unwrap();
    /// assert_eq!(route.get_path(), &"/".to_string());
    /// ```
    pub fn get_path(&self) -> &String {
        &self.path
    }
    /// Returns the collection of route's methods.
    ///
    /// # Examples
    /// ```
    /// # use kalgan_router::Router;
    /// # let router = Router::new("tests/routes.yaml");
    /// # let route = router.get_route("/", "get").unwrap();
    /// assert_eq!(route.get_methods(), &vec!["get".to_string()]);
    /// ```
    pub fn get_methods(&self) -> &Vec<String> {
        &self.methods
    }
    /// Returns the path of the routes's controller.
    ///
    /// # Examples
    /// ```
    /// # use kalgan_router::Router;
    /// # let router = Router::new("tests/routes.yaml");
    /// # let route = router.get_route("/", "get").unwrap();
    /// assert_eq!(route.get_controller(), &"home_controller::index".to_string());
    /// ```
    pub fn get_controller(&self) -> &String {
        &self.controller
    }
    /// Returns the path of the route's middleware.
    ///
    /// # Examples
    /// ```
    /// # use kalgan_router::Router;
    /// # let router = Router::new("tests/routes.yaml");
    /// # let route = router.get_route("/", "get").unwrap();
    /// assert_eq!(route.get_middleware(), &"".to_string());
    /// ```
    pub fn get_middleware(&self) -> &String {
        &self.middleware
    }
    fn get_path_split(route_keys: &Mapping) -> Vec<Parameter> {
        let uri = Route::parse_path(route_keys);
        let mut start_path = 0;
        let mut collection = Vec::new();
        if get_regex_for_parameters().find_iter(&uri).count() == 0 {
            collection.push(Parameter {
                value: uri.to_string(),
                requirement: None,
            });
        } else {
            for mat in get_regex_for_parameters().find_iter(&uri) {
                if start_path < mat.start() {
                    collection.push(Parameter {
                        value: uri[start_path..mat.start()].to_string(),
                        requirement: None,
                    });
                }
                let value = kalgan_string::strip_both(&uri[mat.start()..mat.end()], '{', '}');
                collection.push(Parameter {
                    value: value.clone(),
                    requirement: Some({
                        if route_keys.contains_key(&Value::String("requirements".to_string())) {
                            let requirement = &route_keys
                                [&Value::String("requirements".to_string())]
                                .as_mapping()
                                .unwrap();
                            if requirement.contains_key(&Value::String(value.clone())) {
                                regex::Regex::new(
                                    format!(
                                        r"{}",
                                        &requirement[&Value::String(value)].as_str().unwrap()
                                    )
                                    .as_str(),
                                )
                                .unwrap()
                            } else {
                                get_regex_default_requirement()
                            }
                        } else {
                            get_regex_default_requirement()
                        }
                    }),
                });
                start_path = mat.end();
            }
            if start_path != uri.len() {
                collection.push(Parameter {
                    value: uri[start_path..].to_string(),
                    requirement: None,
                });
            }
        }
        collection
    }
    fn uri_matches_path(&mut self, uri: &str) -> bool {
        let mut start = 0;
        let mut end;
        let collection = &self.path_split.clone();
        for (index, parameter) in collection.iter().enumerate() {
            if parameter.requirement.is_none() {
                end = start + parameter.value.len();
                if uri.len() < end || uri[start..end] != parameter.value {
                    return false;
                }
            } else {
                let mut partial_uri = &uri[start..];
                if self.path_split.len() >= (index + 2)
                    && self.path_split[index + 1].requirement.is_none()
                {
                    match partial_uri.find(&self.path_split[index + 1].value) {
                        Some(position) => partial_uri = &partial_uri[..position],
                        None => return false,
                    }
                }
                match parameter.requirement.as_ref().unwrap().find(&partial_uri) {
                    Some(value) => {
                        if value.start() == 0 {
                            let result = &partial_uri[value.start()..value.end()];
                            end = start + result.len();
                            if !parameter.requirement.is_none() {
                                self.parameters
                                    .insert(parameter.value.clone(), result.to_string());
                            }
                        } else {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
            start = end;
        }
        if uri.len() == start {
            debug!("Route \"{}\" matches \"{}\".", self.name, uri);
            true
        } else {
            false
        }
    }
    fn parse_methods(route_keys: &Mapping) -> Vec<String> {
        if route_keys.contains_key(&Value::String("methods".to_string())) {
            let mut collection: Vec<String> = Vec::new();
            let col: Vec<&str> = kalgan_string::strip(
                &route_keys[&Value::String("methods".to_string())]
                    .as_str()
                    .unwrap(),
                ',',
            )
            .split(",")
            .collect();
            for method in col {
                collection.push(method.trim().to_string().to_lowercase());
            }
            collection
        } else {
            Vec::new()
        }
    }
    fn parse_path(route_keys: &Mapping) -> String {
        route_keys[&Value::String("path".to_string())]
            .as_str()
            .unwrap()
            .to_string()
    }
    fn parse_controller(route_keys: &Mapping) -> String {
        route_keys[&Value::String("controller".to_string())]
            .as_str()
            .unwrap()
            .replace("/", "::")
            .to_string()
    }
    fn parse_middleware(route_keys: &Mapping) -> String {
        if route_keys.contains_key(&Value::String("middleware".to_string())) {
            route_keys[&Value::String("middleware".to_string())]
                .as_str()
                .unwrap()
                .to_string()
        } else {
            "".to_string()
        }
    }
    fn parse_language(route_keys: &Mapping) -> String {
        if route_keys.contains_key(&Value::String("language".to_string())) {
            route_keys[&Value::String("language".to_string())]
                .as_str()
                .unwrap()
                .to_string()
        } else {
            "".to_string()
        }
    }
    fn set_language(&mut self) {
        if !self.language.is_empty() {
            if self.parameters.contains_key(&self.language) {
                self.language = self.parameters[&self.language].clone();
            }
        }
    }
}
impl Router {
    /// Creates and return the `Router` instance given the routes source path (can be a file or a folder).
    /// # Examples
    /// ```
    /// use kalgan_router::Router;
    /// let router = Router::new("tests/routes.yaml");
    /// ```
    pub fn new(source: &str) -> Router {
        routes::generate(source)
    }
    /// Returns the `Route` instance for the given uri and method.
    /// # Examples
    /// ```
    /// # use kalgan_router::Router;
    /// # let router = Router::new("tests/routes.yaml");
    /// let route = router.get_route("/", "get").unwrap();
    /// ```
    pub fn get_route(&self, uri: &str, method: &str) -> Result<Route, String> {
        debug!("Finding a Route for \"{}\"...", uri);
        for item in &self.collection {
            let mut route = item.clone();
            debug!("Checking Route \"{}\"...", route.name);
            if (route.methods.is_empty()
                || route.methods.contains(&method.to_string().to_lowercase()))
                && route.uri_matches_path(&uri)
            {
                route.set_language();
                return Ok(route);
            }
        }
        Err(format!(
            "No route found for uri '{}' and method '{}'",
            uri, method
        ))
    }
    /// Returns the `uri` for the given route name.
    /// # Examples
    /// ```
    /// # use std::collections::HashMap;
    /// # use kalgan_router::Router;
    /// # let router = Router::new("tests/routes.yaml");
    /// assert_eq!(router.get_uri("home", HashMap::new()), "/".to_string())
    /// ```
    /// ```
    /// # use std::collections::HashMap;
    /// # use kalgan_router::Router;
    /// # let router = Router::new("tests/routes.yaml");
    /// let mut parameters = HashMap::new();
    /// parameters.insert("id", "101".to_string());
    /// assert_eq!(router.get_uri("user", parameters), "/user/101".to_string())
    /// ```
    pub fn get_uri(&self, route_name: &str, parameters: HashMap<&str, String>) -> String {
        for route in &self.collection {
            if route.name == route_name {
                let mut uri = route.path.clone();
                for (key, value) in parameters {
                    uri = uri.replace(&format!("{{{}}}", key), &value);
                }
                return uri;
            }
        }
        warn!("Route \"{}\" not found.", route_name);
        format!("Route \"{}\" not found.", route_name)
    }
}
fn get_regex_for_parameters() -> regex::Regex {
    regex::Regex::new(r"\{.+?\}").unwrap()
}
fn get_regex_default_requirement() -> regex::Regex {
    regex::Regex::new(r"[^/]+").unwrap()
}
