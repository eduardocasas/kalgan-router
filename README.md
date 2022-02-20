# kalgan-router

HTTP routing tool based on routes stored in yaml files used by Kalgan Framework.

## Examples

This is the yaml file to be used in the following tests:
```yaml
## tests/routes.yaml

routes:
  - home:
      path: /
      controller: home_controller::index
      methods: get
  - user:
      path: /user/{id}
      controller: user_controller::crud
      middleware: user_middleware::test
      methods: get, post, delete, put
      requirements:
        id: "^[0-9]+"
```
```rust
use kalgan_router::Router;

let router = Router::new("tests/routes.yaml");
```
```rust
assert_eq!(router.get_uri("home", HashMap::new()), "/".to_string())
```
```rust
let mut parameters = HashMap::new();
parameters.insert("id", "101".to_string());
assert_eq!(router.get_uri("user", parameters), "/user/101".to_string())
```
```rust
let route = router.get_route("/", "get").unwrap();
```
```rust
assert_eq!(route.get_name(), &"home".to_string());
```
```rust
assert_eq!(route.get_path(), &"/".to_string());
```
```rust
assert_eq!(route.get_methods(), &vec!["get".to_string()]);
```
```rust
assert_eq!(route.get_controller(), &"home_controller::index".to_string());
```
```rust
assert_eq!(route.get_middleware(), &"".to_string());
```
## Documentation

For further information please visit:

* [Official Kalgan Site](https://kalgan.eduardocasas.com)
* [API Documentation on docs.rs](https://docs.rs/crate/kalgan-router/latest)


## License

This crate is licensed under either of the following licenses:

* [MIT License](https://choosealicense.com/licenses/mit/)
* [Apache License 2.0](https://choosealicense.com/licenses/apache-2.0/)
