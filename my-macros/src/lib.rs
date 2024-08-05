extern crate proc_macro;

use proc_macro::TokenStream;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::parse_macro_input;
use walkdir::WalkDir;

#[derive(thiserror::Error, Debug, Eq, PartialEq, Clone)]
enum PageGenerationError {
    #[error("Invalid page name '{0}'.")]
    InvalidPageName(String),
    #[error("The file '{0}' can't be read.")]
    CantReadFile(PathBuf),
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Page {
    name: String,
    path: String,
    content: String,
}

/// Get all files in the pages directory recursively
/// Check that the files and directories have valid names
/// Construct a page struct for each file
fn get_pages(pages_dir: &Path) -> Result<Vec<Page>, PageGenerationError> {
    let file_name_regex = regex::Regex::new(r"^[a-zA-Z0-9]+$").unwrap();

    let mut pages = Vec::new();

    let walker = WalkDir::new(pages_dir).into_iter();

    // skip pages_dir itself
    for entry in walker.skip(1) {
        let entry = entry.expect("Couldn't unwrap entry");

        let name = entry
            .file_name()
            .to_str()
            .expect("Couldn't convert file name to string")
            .to_string();
        if !file_name_regex.is_match(&name) {
            return Err(PageGenerationError::InvalidPageName(name));
        }

        // continue if the entry is a directory
        if entry.file_type().is_dir() {
            continue;
        }

        let path = format!(
            "/{}",
            entry
                .path()
                .strip_prefix(pages_dir)
                .expect("Couldn't strip prefix")
                .to_str()
                .expect("Couldn't convert path to string")
        );

        let Ok(content) = fs::read_to_string(entry.path()) else {
            return Err(PageGenerationError::CantReadFile(
                entry.path().to_path_buf(),
            ))
        };

        pages.push(Page {
            name,
            path,
            content,
        });
    }

    println!("Found pages: {pages:?}");

    Ok(pages)
}

struct GeneratePagesInput {
    pages_dir: String,
    predefined_routes: proc_macro2::TokenStream,
}

impl Parse for GeneratePagesInput {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        // Parse the first argument: pages_dir as a string literal
        let pages_dir: syn::LitStr = input.parse()?;

        // Parse the comma separating the arguments
        let _comma: syn::Token![,] = input.parse()?;

        // Parse the second argument: predefined_routes as a token stream inside brackets
        let content;
        syn::bracketed!(content in input);
        let predefined_routes: proc_macro2::TokenStream = content.parse()?;
        
        Ok(GeneratePagesInput {
            pages_dir: pages_dir.value(),
            predefined_routes,
        })
    }
}

/// Generates the Route enum and a Component function for each page
///
///
/// # Panics
///
/// Will panic if any page can't be created or doesn't match the naming scheme
#[proc_macro]
pub fn generate_pages(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GeneratePagesInput);
    let pages_dir = input.pages_dir;
    println!("Pages directory: '{pages_dir}'");
    let predefined_routes = input.predefined_routes;
    println!("Predefined routes: '{predefined_routes}'");

    let pages = get_pages(Path::new(&pages_dir)).unwrap();

    let mut functions = Vec::new();
    let mut routes = Vec::new();

    for page in pages {
        let function_name = syn::Ident::new(&page.name, proc_macro2::Span::call_site());
        let page_name = page.name;
        let page_path = page.path;
        let page_content = page.content;

        // Generate enum variant with route attribute
        let route = quote! {
            #[route(#page_path)]
            #function_name {},
        };
        routes.push(route);

        // Generate component function
        let function = quote! {
            #[component]
            fn #function_name() -> Element {
                rsx! {
                    PageTitle { title: #page_name }
                    PageContent { content: #page_content }
                    Link { to: Route::Home {}, "Go Home" }
                }
            }
        };
        functions.push(function);
    }

    let expanded = quote! {
        #[derive(Clone, Routable, Debug, PartialEq)]
        enum Route {
            #predefined_routes
            #(#routes)*
        }

        #(#functions)*
    };

    println!("Generated code: '{expanded}'");

    TokenStream::from(expanded)
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::{get_pages, Page, PageGenerationError};

    #[rstest]
    #[case("test_correct", Ok(vec ! [
    Page {
    name: "Page0".to_string(),
    path: "/Page0".to_string(),
    content: "Page0Content".to_string(),
    },
    Page {
    name: "SubDir0Page0".to_string(),
    path: "/SubDir0/SubDir0Page0".to_string(),
    content: "SubDir0Page0Content".to_string(),
    },
    Page {
    name: "SubDir0Page1".to_string(),
    path: "/SubDir0/SubDir0Page1".to_string(),
    content: "SubDir0Page1Content".to_string(),
    },
    Page {
    name: "SubDir1Page0".to_string(),
    path: "/SubDir1/SubDir1Page0".to_string(),
    content: "SubDir1Page0Content".to_string(),
    },
    Page {
    name: "SubDir1SubDir1Page0".to_string(),
    path: "/SubDir1/SubDir1SubDir0/SubDir1SubDir1Page0".to_string(),
    content: "SubDir1SubDir1Page0Content".to_string(),
    },
    ]))]
    #[case("test_dir_wrong", Err(PageGenerationError::InvalidPageName("sub_dir_0".to_string())))]
    #[case("test_pages_wrong", Err(
        PageGenerationError::InvalidPageName("SubDir0_page0".to_string())
    ))]
    fn test_get_pages(
        #[case] value: &str,
        #[case] expected_result: Result<Vec<Page>, PageGenerationError>,
    ) {
        assert_eq!(
            expected_result,
            get_pages(Path::new(&format!("./tests/data/{value}")))
        );
    }
}
