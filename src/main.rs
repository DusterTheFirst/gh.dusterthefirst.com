use data::use_repos;
use dioxus::prelude::*;

mod data;
mod gh;
mod view;

fn main() {
    console_log::init_with_level(if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    })
    .expect("logger already initialized");

    dioxus::web::launch(app);
}

fn app(cx: Scope) -> Element {
    let repos = use_repos(&cx);

    let repos = match repos {
        None => rsx! {
            div { "loading github information" }
        },
        Some((Ok(repos), _)) => rsx! {
            view::repos::all_repositories {
                repos: repos
            }
        },
        Some((Err(error), refetch)) => rsx! {
            view::error::github_api_error {
                error: error,
                refetch: refetch,
            }
        },
    };

    let input = use_state(&cx, String::new);

    cx.render(rsx! {
        div {
            h1 { "Repos and Color" }
            repos

            hr {}

            input { value: "{input}", oninput: |event| input.set(event.value.clone()) }
            pre { "{input}" }

        }
    })
}
