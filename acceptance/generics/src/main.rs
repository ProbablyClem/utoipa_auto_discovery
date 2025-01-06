pub mod routes;
pub mod schemas;

use routes::*;

use utoipa::OpenApi;
use utoipauto::utoipauto;

#[utoipauto]
#[derive(Debug, OpenApi)]
#[openapi(info(title = "Generic Test Api"))]
pub(crate) struct ApiDoc;

fn main() {
    println!(
        "Our OpenApi documentation {}",
        ApiDoc::openapi().to_pretty_json().unwrap()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use utility::assert_json_eq;

    pub(crate) const EXPECTED_OPEN_API: &str = include_str!("open_api.expected.json");

    #[test]
    fn test_openapi() {
        let open_api = ApiDoc::openapi().to_json().unwrap();

        assert_json_eq(&open_api, EXPECTED_OPEN_API);
    }
}
