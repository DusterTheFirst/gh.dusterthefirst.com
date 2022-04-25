use std::{collections::HashMap, iter};

use dioxus::prelude::*;
use log::{debug, trace};
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
pub struct RepoAndColor {
    pub repo: Repo,
    pub color: Option<String>,
}

pub type RefetchFn<'f> = Box<dyn Fn() + 'f>;

type Repos = HashMap<&'static str, Vec<RepoAndColor>>;

pub fn use_repos<'state>(
    cx: &'state ScopeState,
    users: Vec<&'static str>,
) -> Option<(&'state Result<Repos, GithubApiError>, RefetchFn<'state>)> {
    let future = use_future(cx, (), move |()| async move {
        let result = futures::try_join!(
            fetch_colors(),
            futures::future::try_join_all(users.iter().map(|user| fetch_all_user_repos(user)))
        );

        result.map(|(colors, repos)| {
            iter::zip(
                users.iter().copied(),
                repos.into_iter().map(|repos| {
                    repos
                        .into_iter()
                        .map(|repo| RepoAndColor {
                            color: repo
                                .language
                                .as_ref()
                                .and_then(|language| colors.get(language).cloned()),
                            repo,
                        })
                        .collect()
                }),
            )
            .collect()
        })
    });

    future.value().map(|res| {
        (
            res,
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

async fn fetch_all_user_repos(user: &str) -> Result<Vec<Repo>, GithubApiError> {
    let mut repos = Vec::new();

    let mut url = format!(
        "https://api.github.com/users/{user}/repos?per_page=100&sort=created&direction=asc"
    );

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

            if let Some(next) = captures.get("next") {
                trace!("paginating to next {next}");

                url = next.to_string();
            } else {
                debug!("Reached end of pagination for {user}");
                break;
            }
        } else {
            debug!("No pagination for {user}");
            break;
        }
    }

    Ok(repos)
}
