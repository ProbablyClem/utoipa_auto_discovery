use std::vec;

use crate::file_utils::{extract_module_name_from_path, parse_files};
use crate::token_utils::Parameters;
use quote::ToTokens;
use syn::meta::ParseNestedMeta;
use syn::token::Comma;
use syn::{punctuated::Punctuated, Attribute, GenericParam, Item, ItemFn, Meta, Token, Type};

/// Discover everything from a file, will explore folder recursively
pub fn discover_from_file(
    src_path: String,
    crate_name: String,
    params: &Parameters,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let files = parse_files(&src_path).unwrap_or_else(|_| panic!("Failed to parse file {}", src_path));

    files
        .into_iter()
        .map(|e| {
            #[cfg(feature = "generic_full_path")]
            let imports = extract_use_statements(&e.0, &crate_name);
            #[cfg(not(feature = "generic_full_path"))]
            let imports = vec![];
            parse_module_items(
                &extract_module_name_from_path(&e.0, &crate_name),
                e.1.items,
                imports,
                params,
            )
        })
        .fold(Vec::<DiscoverType>::new(), |mut acc, mut v| {
            acc.append(&mut v);
            acc
        })
        .into_iter()
        .fold(
            (Vec::<String>::new(), Vec::<String>::new(), Vec::<String>::new()),
            |mut acc, v| {
                match v {
                    DiscoverType::Fn(n) => acc.0.push(n),
                    DiscoverType::Model(n) => acc.1.push(n),
                    DiscoverType::Response(n) => acc.2.push(n),
                    DiscoverType::CustomModelImpl(n) => acc.1.push(n),
                    DiscoverType::CustomResponseImpl(n) => acc.2.push(n),
                };

                acc
            },
        )
}

enum DiscoverType {
    Fn(String),
    Model(String),
    Response(String),
    CustomModelImpl(String),
    CustomResponseImpl(String),
}

fn parse_module_items(
    module_path: &str,
    items: Vec<Item>,
    imports: Vec<String>,
    params: &Parameters,
) -> Vec<DiscoverType> {
    items
        .into_iter()
        .filter(|e| {
            matches!(
                e,
                syn::Item::Mod(_) | syn::Item::Fn(_) | syn::Item::Struct(_) | syn::Item::Enum(_) | syn::Item::Impl(_)
            )
        })
        .map(|v| match v {
            syn::Item::Mod(m) => m.content.map_or(Vec::<DiscoverType>::new(), |cs| {
                parse_module_items(
                    &build_path(module_path, &m.ident.to_string()),
                    cs.1,
                    imports.clone(),
                    params,
                )
            }),
            syn::Item::Fn(f) => parse_function(&f, &params.fn_attribute_name)
                .into_iter()
                .map(|item| DiscoverType::Fn(build_path(module_path, &item)))
                .collect(),
            syn::Item::Struct(s) => parse_from_attr(
                &s.attrs,
                &build_path(module_path, &s.ident.to_string()),
                s.generics.params,
                imports.clone(),
                params,
            ),
            syn::Item::Enum(e) => parse_from_attr(
                &e.attrs,
                &build_path(module_path, &e.ident.to_string()),
                e.generics.params,
                imports.clone(),
                params,
            ),
            syn::Item::Impl(im) => parse_from_impl(&im, module_path, params),
            _ => vec![],
        })
        .fold(Vec::<DiscoverType>::new(), |mut acc, mut v| {
            acc.append(&mut v);
            acc
        })
}

/// Search for ToSchema and ToResponse implementations in attr
fn parse_from_attr(
    a: &Vec<Attribute>,
    name: &str,
    generic_params: Punctuated<GenericParam, Comma>,
    imports: Vec<String>,
    params: &Parameters,
) -> Vec<DiscoverType> {
    let mut out: Vec<DiscoverType> = vec![];
    let is_non_lifetime_generic = !generic_params.iter().all(|p| matches!(p, GenericParam::Lifetime(_)));

    for attr in a {
        let meta = &attr.meta;
        if meta.path().is_ident("utoipa_ignore") {
            return vec![];
        }
        if meta.path().is_ident("derive") {
            let nested = attr
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .expect("Failed to parse derive attribute");
            for nested_meta in nested {
                if nested_meta.path().segments.len() == 2 {
                    if nested_meta.path().segments[0].ident == "utoipa" {
                        if nested_meta.path().segments[1].ident == "ToSchema" && !is_non_lifetime_generic {
                            out.push(DiscoverType::Model(name.to_string()));
                        } else if nested_meta.path().segments[1].ident == "ToResponse" && !is_non_lifetime_generic {
                            out.push(DiscoverType::Response(name.to_string()));
                        }
                    }
                } else if nested_meta.path().is_ident(&params.schema_attribute_name) && !is_non_lifetime_generic {
                    out.push(DiscoverType::Model(name.to_string()));
                }
                if nested_meta.path().is_ident(&params.response_attribute_name) {
                    out.push(DiscoverType::Response(name.to_string()));
                }
            }
        }
        if is_non_lifetime_generic && attr.path().is_ident("aliases") {
            let _ = attr.parse_nested_meta(|meta| {
                out.push(DiscoverType::Model(parse_generic_schema(
                    meta,
                    name,
                    imports.clone(),
                    &generic_params,
                )));

                Ok(())
            });
        }
    }

    out
}

#[cfg(not(feature = "generic_full_path"))]
fn parse_generic_schema(
    meta: ParseNestedMeta,
    name: &str,
    _imports: Vec<String>,
    non_lifetime_generic_params: &Punctuated<GenericParam, Comma>,
) -> String {
    let splited_types = split_type(meta);
    let mut nested_generics = Vec::new();

    for spl_type in splited_types {
        let parts: Vec<&str> = spl_type.split(',').collect();
        let mut generics = Vec::new();

        for (index, part) in parts.iter().enumerate() {
            match non_lifetime_generic_params
                .get(index)
                .expect("Too few parameters provided to generic")
            {
                GenericParam::Lifetime(_) => (),
                _ => generics.push(part.to_string()),
            };
        }

        nested_generics.push(generics.join(", "));
    }

    let generics = merge_nested_generics(nested_generics);

    name.to_string() + &generics
}

#[cfg(feature = "generic_full_path")]
fn parse_generic_schema(
    meta: ParseNestedMeta,
    name: &str,
    imports: Vec<String>,
    non_lifetime_generic_params: &Punctuated<GenericParam, Comma>,
) -> String {
    let splitted_types = split_type(meta);
    let mut nested_generics = Vec::new();

    for spl_type in splitted_types {
        let parts: Vec<&str> = spl_type.split(",").collect();
        let mut generics = Vec::new();

        for (index, part) in parts.iter().enumerate() {
            match non_lifetime_generic_params
                .get(index)
                .expect("Too few parameters provided to generic")
            {
                GenericParam::Lifetime(_) => (),
                GenericParam::Type(_) => generics.push(process_one_generic(part, name, imports.clone())),
                GenericParam::Const(_) => generics.push(part.to_string()),
            };
        }

        nested_generics.push(generics.join(", "));
    }

    let generics = merge_nested_generics(nested_generics);
    let generic_type_with_module_path = name.to_string() + &generics;

    generic_type_with_module_path
}

#[cfg(feature = "generic_full_path")]
fn process_one_generic(part: &str, name: &str, imports: Vec<String>) -> String {
    let mut processed_parts = Vec::new();

    if part.contains("<") {
        // Handle nested generics
        let nested_parts: Vec<&str> = part.splitn(2, "<").collect();
        let nested_generic = find_import(
            imports.clone(),
            get_current_module_from_name(name).as_str(),
            nested_parts[0].trim(),
        ) + "<"
            + &process_one_generic(nested_parts[1].trim(), name, imports.clone());
        processed_parts.push(nested_generic);
    } else {
        // Normal type, find the full path
        let full_path = find_import(
            imports.clone(),
            get_current_module_from_name(name).as_str(),
            part.trim(),
        );
        processed_parts.push(full_path);
    }

    processed_parts.join("::")
}

pub fn split_type(meta: ParseNestedMeta) -> Vec<String> {
    let value = meta.value().unwrap(); // this parses the `=`
    let generic_type: Type = value.parse().unwrap();
    let type_as_string = generic_type.into_token_stream().to_string();
    // get generic type
    let start = type_as_string.find('<').unwrap_or(0) + 1;
    let end = type_as_string.rfind('>').unwrap_or(type_as_string.len());
    let splited_type = type_as_string[start..end].to_string();

    let types: Vec<String> = splited_type
        .split('<')
        .map(|s| s.split('>').next().unwrap_or("").to_string())
        .collect();

    types
}

fn merge_nested_generics(nested_generics: Vec<String>) -> String {
    let mut generics = String::from("<");
    if nested_generics.len() == 1 {
        generics = generics + nested_generics.first().unwrap() + ">";
    } else {
        for (i, gen) in nested_generics.iter().enumerate() {
            generics.push_str(gen.trim());
            if i != nested_generics.len() - 1 {
                generics.push('<');
            }
        }
        for _ in 0..nested_generics.len() {
            generics.push('>');
        }
    }

    generics
}

fn parse_from_impl(im: &syn::ItemImpl, module_base_path: &str, params: &Parameters) -> Vec<DiscoverType> {
    im.trait_
        .as_ref()
        .and_then(|trt| trt.1.segments.last().map(|p| p.ident.to_string()))
        .and_then(|impl_name| {
            if impl_name.eq(params.schema_attribute_name.as_str()) {
                Some(vec![DiscoverType::CustomModelImpl(build_path(
                    module_base_path,
                    &im.self_ty.to_token_stream().to_string(),
                ))])
            } else if impl_name.eq(params.response_attribute_name.as_str()) {
                Some(vec![DiscoverType::CustomResponseImpl(build_path(
                    module_base_path,
                    &im.self_ty.to_token_stream().to_string(),
                ))])
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn parse_function(f: &ItemFn, fn_attributes_name: &str) -> Vec<String> {
    let mut fns_name: Vec<String> = vec![];
    if should_parse_fn(f) {
        for i in 0..f.attrs.len() {
            if f.attrs[i]
                .meta
                .path()
                .segments
                .iter()
                .any(|item| item.ident.eq(fn_attributes_name))
            {
                fns_name.push(f.sig.ident.to_string());
            }
        }
    }
    fns_name
}

fn should_parse_fn(f: &ItemFn) -> bool {
    !f.attrs.is_empty() && !is_ignored(f)
}

fn is_ignored(f: &ItemFn) -> bool {
    f.attrs.iter().any(|attr| {
        if let Some(name) = attr.path().get_ident() {
            name.eq("utoipa_ignore")
        } else {
            false
        }
    })
}

fn build_path(file_name: &str, fn_name: &str) -> String {
    format!("{}::{}", file_name, fn_name)
}

#[cfg(feature = "generic_full_path")]
fn extract_use_statements(file_path: &str, crate_name: &str) -> Vec<String> {
    let file = std::fs::read_to_string(file_path).unwrap();
    let mut out: Vec<String> = vec![];
    let mut multiline_import = String::new();
    let mut is_multiline = false;

    for line in file.lines() {
        let mut line = line.trim().to_string();

        if is_multiline {
            multiline_import.push_str(&line);
            if line.ends_with("}") {
                is_multiline = false;
                line = multiline_import.clone();
                multiline_import.clear();
            } else {
                continue;
            }
        }

        if line.starts_with("use") {
            line = line.replace("use ", "").replace(";", "").replace(crate_name, "");

            if line.ends_with("{") {
                is_multiline = true;
                multiline_import = line;
                continue;
            }

            let parts: Vec<&str> = line.split('{').collect();
            if parts.len() > 1 {
                let module_path = parts[0];
                let imports: Vec<&str> = parts[1].trim_end_matches('}').split(',').collect();
                for import in imports {
                    let import = import.trim();
                    if import.starts_with("::") {
                        out.push(format!("{}{}", crate_name, import));
                    } else {
                        out.push(format!("{}{}", module_path, import));
                    }
                }
            } else {
                if line.starts_with("::") {
                    line = format!("{}{}", crate_name, line);
                }
                out.push(line);
            }
        }
    }
    out
}

#[cfg(feature = "generic_full_path")]
fn find_import(imports: Vec<String>, current_module: &str, name: &str) -> String {
    let name = name.trim();
    let current_module = current_module.trim();
    for import in imports.iter() {
        if import.contains(name) {
            let full_path = import_to_full_path(import);
            return full_path;
        }
    }

    // If the name contains `::` it means that it's a partial import or a full path
    if name.contains("::") {
        return handle_partial_import(imports, name).unwrap_or_else(|| name.to_string());
    }

    // Only append the module path if the name does not already contain it
    if !name.starts_with(current_module) {
        return current_module.to_string() + "::" + name;
    }

    name.to_string()
}

#[cfg(feature = "generic_full_path")]
fn handle_partial_import(imports: Vec<String>, name: &str) -> Option<String> {
    name.split("::").next().and_then(|first| {
        let first = first.trim();

        for import in imports {
            let import = import.trim();

            if import.ends_with(first) {
                let full_path = import_to_full_path(&import);

                let usable_import = format!("{}{}", full_path.trim(), name[first.len()..].trim());
                return Some(usable_import);
            }
        }
        None
    })
}

#[cfg(feature = "generic_full_path")]
fn import_to_full_path(import: &str) -> String {
    import.split(" as ").next().unwrap_or(&import).trim().to_string()
}

#[cfg(feature = "generic_full_path")]
fn get_current_module_from_name(name: &str) -> String {
    let parts: Vec<&str> = name.split("::").collect();
    parts[..parts.len() - 1].join("::")
}

#[cfg(test)]
mod test {
    #[cfg(feature = "generic_full_path")]
    use crate::discover::{find_import, get_current_module_from_name, process_one_generic};
    use quote::quote;

    #[test]
    fn test_parse_function() {
        let quoted = quote! {
            #[utoipa]
            pub fn route_custom() {}
        };

        let item_fn: syn::ItemFn = syn::parse2(quoted).unwrap();
        let fn_name = super::parse_function(&item_fn, "utoipa");
        assert_eq!(fn_name, vec!["route_custom"]);

        let quoted = quote! {
            #[handler]
            pub fn route_custom() {}
        };

        let item_fn: syn::ItemFn = syn::parse2(quoted).unwrap();
        let fn_name = super::parse_function(&item_fn, "handler");
        assert_eq!(fn_name, vec!["route_custom"]);
    }

    #[test]
    #[cfg(feature = "generic_full_path")]
    fn test_process_one_generic_nested_generics() {
        let part = "Generic<Inner>";
        let name = "module::name";
        let imports = vec!["module::Generic".to_string(), "module::Inner".to_string()];
        let expected = "module::Generic<module::Inner>";
        let result = process_one_generic(part, name, imports);
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg(feature = "generic_full_path")]
    fn test_process_one_generic_partial_import() {
        let part = "PartialImportGenericSchema<more_schemas::MoreSchema>";
        let name = "crate";
        let imports = vec!["crate::generic_full_path::more_schemas".to_string()];
        let expected = "PartialImportGenericSchema<crate::generic_full_path::more_schemas::MoreSchema>";
        let result = process_one_generic(part, name, imports);
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg(feature = "generic_full_path")]
    fn test_process_one_generic_no_nested_generics() {
        let part = "Generic";
        let name = "module::name";
        let imports = vec!["module::Generic".to_string()];
        let expected = "module::Generic";
        let result = process_one_generic(part, name, imports);
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg(feature = "generic_full_path")]
    fn test_process_one_generic_no_generics() {
        let part = "NonGeneric";
        let name = "module::name";
        let imports = vec!["module::NonGeneric".to_string()];
        let expected = "module::NonGeneric";
        let result = process_one_generic(part, name, imports);
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg(feature = "generic_full_path")]
    fn test_find_import_multiple_modules() {
        let imports = vec![
            "module1::module2::name".to_string(),
            "module1::module2::other_name".to_string(),
        ];
        let current_module = "module1::module2";
        let name = "name";
        let expected = "module1::module2::name";
        let result = find_import(imports, current_module, name);
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg(feature = "generic_full_path")]
    fn test_find_import_single_module() {
        let imports = vec!["module1::name".to_string(), "module1::other_name".to_string()];
        let current_module = "module1";
        let name = "name";
        let expected = "module1::name";
        let result = find_import(imports, current_module, name);
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg(feature = "generic_full_path")]
    fn test_find_import_no_module() {
        let imports = vec!["name".to_string(), "other_name".to_string()];
        let current_module = "";
        let name = "name";
        let expected = "name";
        let result = find_import(imports, current_module, name);
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg(feature = "generic_full_path")]
    fn test_get_current_module_from_name_multiple_modules() {
        let name = "module1::module2::name";
        let expected = "module1::module2";
        let result = get_current_module_from_name(name);
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg(feature = "generic_full_path")]
    fn test_get_current_module_from_name_single_module() {
        let name = "module1::name";
        let expected = "module1";
        let result = get_current_module_from_name(name);
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg(feature = "generic_full_path")]
    fn test_get_current_module_from_name_no_module() {
        let name = "name";
        let expected = "";
        let result = get_current_module_from_name(name);
        assert_eq!(result, expected);
    }
}
