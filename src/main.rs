use dioxus::prelude::*;
use hook::{repos::use_repos, viewport::use_viewport};

mod gh;
mod hook;
mod time;
mod view;

fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        // Try to show the panic in HTML
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.body())
            .and_then(|element| element.set_attribute("data-panicked", "true").ok());

        // Use console error panic hook to send the info to the console
        console_error_panic_hook::hook(panic_info);
    }));
}

fn main() {
    set_panic_hook();

    console_log::init_with_level(if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    })
    .expect("logger already initialized");

    dioxus::web::launch(app);

    // Dioxus unconditionally replaces the panic hook, so reinstate it
    set_panic_hook();
}

fn app(cx: Scope) -> Element {
    let repos = use_repos(&cx, vec!["dusterthefirst", "thedustyard"]);

    let repos = match repos {
        None => rsx! {
            div { "loading github information" }
        },
        Some((Ok(repos), _)) => LazyNodes::new(|cx| {
            cx.fragment_from_iter(repos.iter().map(|(user, repos)| {
                rsx! {
                    section {
                        key: "{user}",

                        h2 {
                            style: "position: sticky; top: 0; background: #000;",
                            "{user}"
                        }
                        repos.iter().map(|repo| rsx!{
                            view::repos::repository {
                                key: "{repo.repo.node_id}",
                                repo: repo
                                saturate: false,
                            }
                        })
                    }
                }
            }))
        }),
        Some((Err(error), refetch)) => rsx! {
            view::error::github_api_error {
                error: error,
                refetch: refetch,
            }
        },
    };

    let viewport = use_viewport(&cx);

    cx.render(rsx! {
        div {
            header {
                class: "title-card",
                onclick: |_| panic!("test"),

                div {
                    class: "extra",

                    "site licensed under "
                    a {
                        href: "http://mozilla.org/MPL/2.0/",
                        title: "the mozilla public license version 2.0",
                        target: "_blank",
                        rel: "license",
                        "MPL-2.0"
                    }
                    " on "
                    a {
                        href: "https://github.com/dusterthefirst/gh.dusterthefirst.com",
                        target: "_blank",
                        rel: "external",
                        title: "this website's source code",
                        "github"
                    }
                }

                div {
                    class: "title",

                    div {
                        class: "main",
                        "Zachary Kohnen"
                    }
                    div {
                        a {
                            href: "mailto:me@dusterthefirst.com",
                            title: "Zachary Kohnen's email",
                            target: "_blank",
                            rel: "author",
                            "me@dusterthefirst.com"
                        }
                    }
                    div {
                        a {
                            href: "https://dusterthefirst.com",
                            title: "Zachary Kohnen's website",
                            target: "_blank",
                            rel: "author",
                            "dusterthefirst.com"
                        }
                    }
                    div {
                        a {
                            href: "https://gh.dusterthefirst.com",
                            title: "Zachary Kohnen's github portfolio",
                            target: "_self",
                            rel: "canonical",
                            "gh.dusterthefirst.com"
                        }
                    }
                    div {
                        a {
                            href: "https://github.com/dusterthefirst",
                            title: "Zachary Kohnen's github",
                            target: "_blank",
                            rel: "external",
                            "github.com/dusterthefirst"
                        }
                    }
                }
            }

            div {
                style: "top: {viewport.scroll_y}px; position: absolute;",

                h1 { "{viewport.scroll_y}px" }
                h1 { "{viewport.client_height}px by {viewport.client_width}px" }
                hr {}
            }

            repos
        }
    })
}
