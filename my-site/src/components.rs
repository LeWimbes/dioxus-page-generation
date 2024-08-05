use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct PageTitleProps {
    title: String,
}
pub fn PageTitle(props: PageTitleProps) -> Element {
    rsx! {
        h1 {"{props.title}"}
    }
}

#[derive(PartialEq, Props, Clone)]
pub struct PageContentProps {
    content: String,
}

pub fn PageContent(props: PageContentProps) -> Element {
    rsx! {
        "{props.content}"
    }
}
