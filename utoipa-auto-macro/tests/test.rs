mod controllers;
use utoipa::OpenApi;
use utoipa_auto_macro::utoipa_auto_discovery;
// // Discover from multiple controllers
// #[utoipa_auto_discovery(
//     paths = "( crate::controllers::controller1 => ./utoipa-auto-macro/tests/controllers/controller1.rs) ; ( crate::controllers::controller2 => ./utoipa-auto-macro/tests/controllers/controller2.rs )"
// )]
// #[derive(OpenApi)]
// #[openapi(info(title = "Percentage API", version = "1.0.0"))]
// pub struct MultiControllerApiDocs {}

// #[test]
// fn test_path_import() {
//     assert_eq!(MultiControllerApiDocs::openapi().paths.paths.len(), 2)
// }

// /// Discover from a single controller
// #[utoipa_auto_discovery(
//     paths = "( crate::controllers::controller1 => ./utoipa-auto-macro/tests/controllers/controller1.rs)"
// )]
// #[derive(OpenApi)]
// #[openapi(info(title = "Percentage API", version = "1.0.0"))]
// pub struct SingleControllerApiDocs {}

// #[test]
// fn test_ignored_path() {
//     assert_eq!(SingleControllerApiDocs::openapi().paths.paths.len(), 1)
// }

/// Discover from a module root
#[utoipa_auto_discovery(paths = "( crate::controllers => ./utoipa-auto-macro/tests/controllers)")]
#[derive(OpenApi)]
#[openapi(info(title = "Percentage API", version = "1.0.0"))]
pub struct ModuleApiDocs {}
#[test]
fn test_module_import_path() {
    println!("{:#?}", ModuleApiDocs::openapi().to_json());
    assert_eq!(ModuleApiDocs::openapi().paths.paths.len(), 2)
}

// /// Discover from the crate root
// #[utoipa_auto_discovery(paths = "( crate => ./utoipa-auto-macro/tests/test.rs)")]
// #[derive(OpenApi)]
// #[openapi(info(title = "Percentage API", version = "1.0.0"))]
// pub struct CrateApiDocs {}

// #[test]
// fn test_crate_import_path() {
//     assert_eq!(CrateApiDocs::openapi().paths.paths.len(), 2)
// }
