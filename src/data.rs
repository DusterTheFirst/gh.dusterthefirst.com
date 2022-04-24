use std::{borrow::Cow, collections::HashMap};

use dioxus::prelude::*;
use once_cell::sync::Lazy;
use regex::Regex;
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

    pub owner: Owner,
}

#[derive(Deserialize, Debug)]
pub struct Owner {
    pub login: String,
    pub avatar_url: String,
    pub html_url: String,
}

#[derive(Debug)]
pub struct RepoAndColor<'r> {
    pub repo: &'r Repo,
    pub color: Option<&'r str>,
}

pub type RefetchFn<'f> = Box<dyn Fn() + 'f>;

pub fn use_repos(
    cx: &ScopeState,
) -> Option<(
    Result<Vec<RepoAndColor<'_>>, &GithubApiError>,
    RefetchFn<'_>,
)> {
    let future = use_future(cx, (), |()| async {
        futures::try_join!(fetch_colors(), fetch_repos())
    });

    future.value().map(|res| {
        (
            res.as_ref().map(|(colors, repos)| {
                repos
                    .iter()
                    .map(|repo| RepoAndColor {
                        color: repo
                            .language
                            .as_ref()
                            .and_then(|language| colors.get(language).map(|ghc| ghc.as_str())),
                        repo,
                    })
                    .collect::<Vec<_>>()
            }),
            Box::new(|| {
                future.clear();
                future.restart();
            }) as Box<_>,
        )
    })
}

async fn fetch_colors() -> Result<HashMap<String, String>, GithubApiError> {
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
        .filter_map(|(key, value)| {
            value["color"]
                .as_str()
                .map(|value| (key, String::from(value)))
        })
        .collect::<HashMap<String, String>>())
}

async fn fetch_repos() -> Result<Vec<Repo>, GithubApiError> {
    // reqwest::get("https://api.github.com/users/DusterTheFirst/repos")

    let mut repos = Vec::new();

    let mut url = Cow::Borrowed("https://api.github.com/orgs/thedustyard/repos?per_page=100"); // TODO: 100

    loop {
        let response = gh::fetch(&url).await?;

        repos.extend(
            response
                .json::<Vec<Repo>>()
                .await
                .expect("received unexpected json content"),
        );

        if let Some(link) = response.headers().get("link") {
            static REGEX: Lazy<Regex> = Lazy::new(|| {
                Regex::new("<(?P<url>.+?)>; rel=\"(?P<rel>.+?)\"").expect("invalid regex")
            });

            let captures = REGEX
                .captures_iter(&link)
                .map(|captures| {
                    (
                        captures.name("rel").expect("no `rel` group").as_str(),
                        captures.name("url").expect("no `url` group").as_str(),
                    )
                })
                .collect::<HashMap<_, _>>();

            log::debug!(target: "amogus", "{captures:?}");

            if let Some(next) = captures.get("next") {
                url = Cow::Owned(next.to_string());
            } else {
                log::debug!("Reached end of pagination");
                break;
            }
        } else {
            log::debug!("No pagination");
            break;
        }
    }

    Ok(repos)
}
