use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::quote;
use syn::Attribute;

pub struct Parameters {
    pub paths: String,
    pub fn_attribute_name: String,
    pub schema_attribute_name: String,
    pub response_attribute_name: String,
    pub convert_to_full_path: bool,
}

/// Extract the paths string attribute from the proc_macro::TokenStream
///
/// If none is specified, we use the default path "./src"
pub fn extract_attributes(stream: proc_macro2::TokenStream) -> Parameters {
    let paths = extract_attribute("paths", stream.clone());
    let fn_attribute_name = extract_attribute("function_attribute_name", stream.clone());
    let schema_attribute_name = extract_attribute("schema_attribute_name", stream.clone());
    let response_attribute_name = extract_attribute("response_attribute_name", stream.clone());
    let convert_to_full_path = extract_bool_attribute("convert_to_full_path", stream);
    // if no paths specified, we use the default path "./src"
    Parameters {
        paths: paths.unwrap_or("./src".to_string()),
        fn_attribute_name: fn_attribute_name.unwrap_or("utoipa".to_string()),
        schema_attribute_name: schema_attribute_name.unwrap_or("ToSchema".to_string()),
        response_attribute_name: response_attribute_name.unwrap_or("ToResponse".to_string()),
        convert_to_full_path: convert_to_full_path.unwrap_or(true),
    }
}

// extract the name = "" attributes from the proc_macro::TokenStream
fn extract_attribute(name: &str, stream: proc_macro2::TokenStream) -> Option<String> {
    let mut has_value = false;

    for token in stream {
        if has_value {
            if let proc_macro2::TokenTree::Literal(lit) = token {
                return Some(get_content(lit));
            }
        }
        if let proc_macro2::TokenTree::Ident(ident) = token {
            if ident.to_string().eq(name) {
                has_value = true;
            }
        }
    }
    None
}

fn extract_bool_attribute(name: &str, stream: proc_macro2::TokenStream) -> Option<bool> {
    let mut has_value = false;

    for token in stream {
        if has_value {
            if let proc_macro2::TokenTree::Ident(ident) = token {
                let value = ident.to_string();
                return Some(value.parse::<bool>().unwrap());
            }
        }
        if let proc_macro2::TokenTree::Ident(ident) = token {
            if ident.to_string().eq(name) {
                has_value = true;
            }
        }
    }
    None
}

fn get_content(lit: Literal) -> String {
    let content = lit.to_string();
    content[1..content.len() - 1].to_string()
}

/// Check if the macro is placed before the #[derive] and #[openapi] attributes
/// Otherwise, panic!
pub fn check_macro_placement(attrs: Vec<Attribute>) {
    if !attrs.iter().any(|elm| elm.path().is_ident("derive")) {
        panic!("Please put utoipauto before #[derive] and #[openapi]");
    }

    if !attrs.iter().any(|elm| elm.path().is_ident("openapi")) {
        panic!("Please put utoipauto before #[derive] and #[openapi]");
    }
}

// Output the macro back to the compiler
pub fn output_macro(openapi_macro: syn::ItemStruct) -> proc_macro::TokenStream {
    let code = quote!(
          #openapi_macro
    );

    TokenStream::from(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_content() {
        let lit = Literal::string("p1");
        let content = get_content(lit);
        assert_eq!(content, "p1");
    }

    #[test]
    fn test_extract_attributes() {
        let tokens = quote! {
            paths = "p1"
        };

        let attributes = extract_attributes(tokens);
        assert_eq!(attributes.paths, "p1")
    }

    #[test]
    fn test_extract_attribute() {
        let quote = quote! {
            paths = "p1", thing = "thing", other = "other"
        };

        let attributes = extract_attribute("thing", quote).unwrap();
        assert_eq!(attributes, "thing");
    }

    #[test]
    fn test_extract_attribute_none() {
        let quote = quote! {
            paths = "p1", thing = "thing", other = "other"
        };

        let attributes = extract_attribute("not_found", quote);
        assert_eq!(attributes, None);
    }

    #[test]
    fn test_extract_attribute_empty() {
        let quote = quote! {};

        let attributes = extract_attribute("thing", quote);
        assert_eq!(attributes, None);
    }

    #[test]
    fn test_extract_attributes_empty() {
        let tokens = quote! {};

        let attributes = extract_attributes(tokens);
        assert_eq!(attributes.paths, "./src");
        assert_eq!(attributes.fn_attribute_name, "utoipa");
        assert_eq!(attributes.schema_attribute_name, "ToSchema");
        assert_eq!(attributes.response_attribute_name, "ToResponse");
    }

    #[test]
    fn test_extract_attributes_custom_name() {
        let tokens = quote! {
            paths = "p1", function_attribute_name = "handler", schema_attribute_name = "Schema", response_attribute_name = "Response"
        };

        let attributes = extract_attributes(tokens);
        assert_eq!(attributes.paths, "p1");
        assert_eq!(attributes.fn_attribute_name, "handler");
        assert_eq!(attributes.schema_attribute_name, "Schema");
        assert_eq!(attributes.response_attribute_name, "Response");
    }
}
