use dioxus::prelude::*;

use crate::hook::repos::RepoAndColor;

#[inline_props]
pub fn all_repositories<'a>(cx: Scope, repos: &'a [RepoAndColor]) -> Element {
    cx.render(LazyNodes::new(|factory: NodeFactory| -> VNode {
        factory.fragment_from_iter(repos.iter().map(|RepoAndColor { repo, color }| {
            let color = color.as_deref().unwrap_or("default");
            let language = repo.language.as_deref().unwrap_or("Unknown");

            rsx! {
                div {
                    key: "{repo.node_id}",
                    style: "background-color: {color}",

                    div {
                        style: "display: flex; justify-content: space-between; text-align: center",

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
            }
        }))
    }))
}
