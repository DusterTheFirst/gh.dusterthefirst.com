use dioxus::prelude::*;
use gloo_events::EventListener;
use hook::repos::use_repos;
use web_sys::window;

mod gh;
mod hook;
mod time;
mod view;

fn main() {
    console_error_panic_hook::set_once();

    console_log::init_with_level(if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    })
    .expect("logger already initialized");

    dioxus::web::launch(app);
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

                        h2 { "{user}" }
                        view::repos::all_repositories {
                            repos: repos
                        }
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

    let input = use_state(&cx, String::new);
    let pad_top = use_state(&cx, || 0.0);

    cx.use_hook(|_| {
        EventListener::new(&window().unwrap().document().unwrap(), "scroll", {
            let pad_top = pad_top.to_owned();

            move |_e| {
                pad_top.set(
                    window()
                        .expect("window should always exist")
                        .scroll_y()
                        .expect("scroll_y should not error"),
                );
            }
        })
        .forget();
    });

    cx.render(rsx! {
        div {
            div {
                style: "top: {pad_top}px; position: absolute;",

                input { value: "{input}", oninput: |event| input.set(event.value.clone()) }
                pre { "{input}" }

                hr {}
            }

            repos
        }
    })
}
