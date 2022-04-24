use std::collections::HashMap;

use dioxus::prelude::*;
use gloo_net::http::{Request, RequestCache};
use serde::Deserialize;
use time::OffsetDateTime;

use crate::gh::{self, GithubApiError};

#[derive(Deserialize, Debug)]
pub struct Repo {
    pub name: String,
    pub node_id: String,
    pub html_url: String,
    pub description: Option<String>,
    pub language: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct RepoAndColor<'r, 'c> {
    pub repo: &'r Repo,
    pub color: Option<&'c str>,
}

type RefetchFn<'f> = Box<dyn Fn() + 'f>;

pub fn use_repos(
    cx: &ScopeState,
) -> Option<(
    Result<impl Iterator<Item = RepoAndColor>, &GithubApiError>,
    RefetchFn<'_>,
)> {
    let future = use_future(cx, (), |()| async {
        futures::try_join!(fetch_colors(), fetch_repos())
    });

    future.value().map(|res| {
        (
            res.as_ref().map(|(colors, repos)| {
                repos.iter().map(|repo| RepoAndColor {
                    color: repo.language.as_ref().and_then(|language| {
                        colors
                            .get(language)
                            .and_then(|ghc| ghc.as_ref().map(|c| c.as_str()))
                    }),
                    repo,
                })
            }),
            Box::new(|| {
                future.clear();
                future.restart();
            }) as Box<_>,
        )
    })
}

async fn fetch_colors() -> Result<HashMap<String, Option<String>>, GithubApiError> {
    let response =
        gh::fetch("https://api.github.com/repos/ozh/github-colors/contents/colors.json").await?;

    let json = response
        .json::<serde_json::Value>()
        .await
        .expect("encountered non-json response body");

    assert_eq!(json["encoding"], "base64", "non base64 encoding used");

    let json = base64::decode(
        json["content"]
            .as_str()
            .expect("content should be a string")
            .bytes()
            .filter(|&byte| byte != b'\n')
            .collect::<Vec<_>>(),
    )
    .expect("content was encoded as invalid base64");

    let colors: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&json).expect("failed to parse color information");

    Ok(colors
        .into_iter()
        .map(|(key, value)| (key, value["color"].as_str().map(String::from)))
        .collect::<HashMap<String, Option<String>>>())
}

async fn fetch_repos() -> Result<Vec<Repo>, GithubApiError> {
    // reqwest::get("https://api.github.com/users/DusterTheFirst/repos")

    // TODO: pagination
    let response = Request::get("https://api.github.com/orgs/thedustyard/repos?per_page=10") // TODO: 100
        .header("accept", "application/vnd.github.v3+json")
        .cache(RequestCache::Default)
        .send()
        .await
        .map_err(GithubApiError::Net)?;

    // let used = response.header("x-ratelimit-used");
    // let remaining = response.header("x-ratelimit-remaining");
    // let resource = response.header("x-ratelimit-resource");
    // let limit = response.header("x-ratelimit-limit");

    if response.status() == 403 {
        let reset = response
            .headers()
            .get("x-ratelimit-reset")
            .expect("x-ratelimit-reset header missing");

        let until = OffsetDateTime::from_unix_timestamp(
            reset
                .as_str()
                .parse()
                .expect("x-ratelimit-reset provided as a non-integer"),
        )
        .expect("x-ratelimit-reset provided an invalid unix timestamp");

        return Err(GithubApiError::RateLimited { until });
    }
    
    let link = response.headers().get("link");

    Ok(response
        .json::<Vec<Repo>>()
        .await
        .expect("received unexpected json content"))
}
