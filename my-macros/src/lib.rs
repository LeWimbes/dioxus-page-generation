extern crate proc_macro;

use proc_macro::TokenStream;
use std::fs;
use std::path::Path;

use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::parse_macro_input;
use walkdir::WalkDir;

#[derive(Debug, Eq, PartialEq, Clone)]
struct Page {
    name: String,
    path: String,
    content: String,
}

/// Get all files in the pages directory recursively
/// Construct a page struct for each file
fn get_pages(pages_dir: &Path) -> Vec<Page> {
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

        let content = fs::read_to_string(entry.path()).unwrap();

        pages.push(Page {
            name,
            path,
            content,
        });
    }

    println!("Found pages: {pages:?}");

    pages
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

    let pages = get_pages(Path::new(&pages_dir));

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

    use crate::{get_pages, Page};

    #[test]
    fn test_get_pages() {
        let expected = vec![
            Page {
                name: "SubSubPage0".to_string(),
                path: "/SubDir0/SubSubDir0/SubSubPage0".to_string(),
                content: "SubSubPage0Content".to_string(),
            },
            Page {
                name: "SubPage0".to_string(),
                path: "/SubDir0/SubPage0".to_string(),
                content: "SubPage0Content".to_string(),
            },
            Page {
                name: "SubPage1".to_string(),
                path: "/SubDir0/SubPage1".to_string(),
                content: "SubPage1Content".to_string(),
            },
            Page {
                name: "Page0".to_string(),
                path: "/Page0".to_string(),
                content: "Page0Content".to_string(),
            },
        ];

        assert_eq!(
            expected,
            get_pages(Path::new("./tests/pages"))
        );
    }
}
