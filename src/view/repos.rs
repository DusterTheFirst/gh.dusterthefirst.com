use css_colors::{percent, Color};
use dioxus::prelude::*;

use crate::hook::repos::RepoAndColor;

#[inline_props]
pub fn repository<'a>(cx: Scope, repo: &'a RepoAndColor, saturate: bool) -> Element {
    let RepoAndColor { repo, color } = repo;

    let color = color
        .map(|color| {
            if *saturate {
                color
            } else {
                color.desaturate(percent(50))
            }
        })
        .map(|color| color.to_string())
        .unwrap_or_else(|| "default".to_string());
    let language = repo.language.as_deref().unwrap_or("Unknown");

    cx.render(rsx! {
        div {
            class: "repo",
            style: "background-color: {color}",

            div {
                class: "description",

                span {
                    "Owner: "
                    a {
                        href: "{repo.owner.html_url}",
                        "{repo.owner.login}"
                    }
                }
                span {
                    "Name: "
                    a {
                        href: "{repo.html_url}",
                        "{repo.name}"
                    }
                }
                repo.description.as_ref().map(|description| rsx!{ span { "Description: {description}" } })
                span { "Lang: {language}" }
                span { "Created: {repo.created_at}" }
                span { "Updated: {repo.updated_at}" }
            }

            details {
                summary { "raw..." }
                pre { "{repo:#?}" }
            }
        }
    })
}
