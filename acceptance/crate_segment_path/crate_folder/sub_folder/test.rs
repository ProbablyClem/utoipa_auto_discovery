use utoipa::OpenApi;
use utoipauto::utoipauto;

#[utoipauto(paths = "( ./crate_segment_path/crate_folder/sub_folder/paths.rs from crate::sub_folder )")]
#[derive(OpenApi)]
#[openapi(info(title = "Percentage API", version = "1.0.0"))]
#[allow(dead_code)]
pub struct CrateInAnotherPath {}

#[utoipauto(
    paths = "( ./folder_in_src/crate_folder/new_sub_folder/paths.rs from folder-in-src::new_sub_folder )",
    convert_to_full_path = false
)]
#[derive(OpenApi)]
#[openapi(info(title = "Percentage API", version = "1.0.0"))]
#[allow(dead_code)]
pub struct CrateInAnotherCrate {}

#[test]
fn test_crate_in_another_path() {
    assert_eq!(CrateInAnotherPath::openapi().paths.paths.len(), 2)
}

#[test]
fn test_crate_in_another_crate() {
    assert_eq!(CrateInAnotherCrate::openapi().paths.paths.len(), 2)
}
