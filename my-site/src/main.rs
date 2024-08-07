#![allow(non_snake_case)]

use dioxus::prelude::*;

use my_macros::generate_pages;

use crate::components::*;

mod components;

generate_pages!("./my-site/pages");

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
    #[route("/SubPage")]
    SubPage {},
}

fn main() {
    launch(App);
}

fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        PageTitle { title: "Home Sweet Home" }
        Link { to: Route::SubPage {}, "Go to SubPage" }
    }
}

#[component]
fn NotFound(segments: Vec<String>) -> Element {
    rsx! {
        PageTitle { title: "404 Not Found!!!!" }
        PageContent { content: "Oh no, you got lost :/" }
        Link { to: Route::Home {}, "Go Home" }
    }
}
