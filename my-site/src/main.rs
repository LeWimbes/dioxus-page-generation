#![allow(non_snake_case)]

mod components;

use dioxus::prelude::*;
use my_macros::generate_pages;
use crate::components::*;

generate_pages!(
    "./my-site/pages/",
    [
        #[route("/")]
        Home {},
        #[route("/:..segments")]
        NotFound {segments: Vec<String>},
    ]
);

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
        Link { to: Route::SubPage1 {}, "Go to SubPage1" }
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
