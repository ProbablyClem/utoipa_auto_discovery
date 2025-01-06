use utoipa::OpenApi;

use utoipauto::utoipauto;

use crate::default_features::controllers;

// Discover from multiple controllers
#[utoipauto(
    paths = "( crate::controllers::controller1 => ./tests/default_features/controllers/controller1.rs) ; ( crate::controllers::controller2 => ./tests/default_features/controllers/controller2.rs )"
)]
#[derive(OpenApi)]
#[openapi(info(title = "Percentage API", version = "1.0.0"))]
pub struct MultiControllerApiDocs {}

#[test]
fn test_path_import() {
    assert_eq!(MultiControllerApiDocs::openapi().paths.paths.len(), 2)
}

/// Discover from a single controller
#[utoipauto(paths = "( crate::controllers::controller1 => ./tests/default_features/controllers/controller1.rs)")]
#[derive(OpenApi)]
#[openapi(info(title = "Percentage API", version = "1.0.0"))]
pub struct SingleControllerApiDocs {}

#[test]
fn test_ignored_path() {
    assert_eq!(SingleControllerApiDocs::openapi().paths.paths.len(), 1)
}

/// Discover with manual path
#[utoipauto(paths = "./tests/default_features/controllers/controller1.rs")]
#[derive(OpenApi)]
#[openapi(
    info(title = "Percentage API", version = "1.0.0"),
    paths(controllers::controller2::route3)
)]
pub struct SingleControllerManualPathApiDocs {}

#[test]
fn test_manual_path() {
    assert_eq!(SingleControllerManualPathApiDocs::openapi().paths.paths.len(), 2)
}

/// Discover from a module root
#[utoipauto(paths = "( crate::controllers => ./tests/default_features/controllers)")]
#[derive(OpenApi)]
#[openapi(info(title = "Percentage API", version = "1.0.0"))]
pub struct ModuleApiDocs {}

#[test]
fn test_module_import_path() {
    assert_eq!(ModuleApiDocs::openapi().paths.paths.len(), 2)
}

/// Discover from the crate root
#[utoipauto(paths = "./tests/default_features")]
#[derive(OpenApi)]
#[openapi(info(title = "Percentage API", version = "1.0.0"))]
pub struct CrateApiDocs {}

#[test]
fn test_crate_import_path() {
    assert_eq!(CrateApiDocs::openapi().paths.paths.len(), 2)
}

// Discover from multiple controllers new syntax
#[utoipauto(
    paths = "./tests/default_features/controllers/controller1.rs, ./tests/default_features/controllers/controller2.rs"
)]
#[derive(OpenApi)]
#[openapi(info(title = "Percentage API", version = "1.0.0"))]
pub struct MultiControllerNoModuleApiDocs {}

#[test]
fn test_path_import_no_module() {
    assert_eq!(MultiControllerNoModuleApiDocs::openapi().paths.paths.len(), 2)
}

// Discover from multiple controllers new syntax
#[utoipauto(paths = "./tests/default_features/models.rs")]
#[derive(OpenApi)]
#[openapi(info(title = "Percentage API", version = "1.0.0"))]
pub struct ModelsImportApiDocs {}

#[test]
fn test_path_import_schema() {
    assert_eq!(
        ModelsImportApiDocs::openapi()
            .components
            .expect("no components")
            .schemas
            .len(),
        3, // 1 derive, 1 manual, 1 manual with utoipa::ToSchema
    )
}

// Discover from multiple controllers new syntax
#[utoipauto(paths = "./tests/default_features/models.rs")]
#[derive(OpenApi)]
#[openapi(info(title = "Percentage API", version = "1.0.0"))]
pub struct ResponsesImportApiDocs {}

#[test]
fn test_path_import_responses() {
    assert_eq!(
        ResponsesImportApiDocs::openapi()
            .components
            .expect("no components")
            .responses
            .len(),
        3, // 1 derive, 1 manual, 1 manual with utoipa::ToResponse
    )
}

/// Discover custom handler
#[utoipauto(
    paths = "./tests/default_features/controllers/controller3.rs",
    function_attribute_name = "test_handler"
)]
#[derive(OpenApi)]
#[openapi(info(title = "Percentage API", version = "1.0.0"))]
pub struct CustomHandlerApiDocs {}

#[test]
fn test_custom_handler() {
    assert_eq!(CustomHandlerApiDocs::openapi().paths.paths.len(), 1)
}
