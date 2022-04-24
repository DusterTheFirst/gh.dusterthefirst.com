use data::use_repos;
use dioxus::prelude::*;
use gh::GithubApiError;
use instant::SystemTime;
use time::macros::format_description;

mod data;
mod gh;

fn main() {
    dioxus::web::launch(app);
}

fn app(cx: Scope) -> Element {
    let repos = use_repos(&cx);

    let repos = match repos {
        None => rsx! {
            div { "loading github information" }
        },
        Some((Ok(repos), _refresh)) => rsx! {
            repos.map(|repo| {
                let color = repo.color.unwrap_or("default");
                let repo = repo.repo;

                rsx! {
                    div {
                        key: "{repo.node_id}",
                        style: "background-color: {color}",

                        div {
                            style: "display: flex; justify-content: space-between; text-align: center",

                            span { "Name: {repo.name}" }
                            span { "Lang: {repo.language:?}" }
                            span { "Created: {repo.created_at}" }
                        }

                        details {
                            summary { "raw..." }
                            pre { "{repo:#?}" }
                        }
                    }

                }
            })
        },
        Some((Err(error), refresh)) => {
            let error = match error {
                GithubApiError::Net(e) => rsx! { div { "{e}" } },
                GithubApiError::RateLimited { until } => {
                    let duration = *until
                        - time::OffsetDateTime::from_unix_timestamp(
                            SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .expect("current time before unix epoch")
                                .as_secs() as _,
                        )
                        .expect("unable to convert time from epoch");

                    let until = until
                        .format(format_description!("[hour]:[minute]:[second] UTC"))
                        .expect("failed to format date");

                    rsx! {
                        div { "encountered a ratelimit, try again after {duration} (@ {until}) "}

                    }
                }
            };

            rsx! {
                div { style: "font-weight: bold", "Encountered error fetching github information"}
                error

                button {
                    onclick: move |_e| { refresh() },
                    "retry"
                }
            }
        }
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
