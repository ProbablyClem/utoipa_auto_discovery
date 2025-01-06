use utoipa::OpenApi;
use utoipauto::utoipauto;

#[utoipauto(paths = "./lib_of_openapi/src from lib_of_openapi")]
#[derive(Debug, OpenApi)]
#[openapi(info(title = "Import form generics test."))]
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
