use std::collections::HashMap;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

fn main() {
    dioxus::web::launch(app);
}

#[derive(Serialize, Deserialize, Debug)]
struct GithubColor {
    color: Option<String>,
}

type GithubColors = HashMap<String, GithubColor>;

#[derive(Serialize, Deserialize, Debug)]
struct Repo {
    name: String,
    html_url: String,
    description: Option<String>,
    language: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    updated_at: OffsetDateTime,
}

fn app(cx: Scope) -> Element {
    let colors = use_future(&cx, (), |_| async move {
        reqwest::get("https://raw.githubusercontent.com/ozh/github-colors/master/colors.json")
            .await?
            .json::<GithubColors>()
            .await
    })
    .value();

    let colors = match colors {
        Some(Ok(colors)) => rsx! {
            pre { "{colors:#?}" }
        },
        Some(Err(e)) => rsx! {
            div { style: "font-weight: bold", "Encountered error fetching github color information"}
            div { "{e}" }
        },
        None => rsx! {
            div { "loading color information" }
        },
    };

    let repos = use_future(&cx, (), |_| async move {
        reqwest::get("https://api.github.com/orgs/thedustyard/repos")
        // reqwest::get("https://api.github.com/users/DusterTheFirst/repos")
            .await?
            .json::<Vec<Repo>>()
            .await
    })
    .value();

    let repos = match repos {
        Some(Ok(colors)) => rsx! {
            pre { "{colors:#?}" }
        },
        Some(Err(e)) => rsx! {
            div { style: "font-weight: bold", "Encountered error fetching github repo information"}
            div { "{e}" }
        },
        None => rsx! {
            div { "loading repo information" }
        },
    };

    cx.render(rsx! {
        div {
            details {
                open: "true",

                summary { "dustyard repos:" }
                repos
            }
            details {
                open: "true",

                summary { "colors:" }
                colors
            }
        }
    })
}
