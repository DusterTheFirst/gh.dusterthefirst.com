use std::{future, ops::Sub};

use dioxus::{
    core::to_owned,
    prelude::{Scope, *},
};
use futures::StreamExt;
use gloo_timers::future::IntervalStream;
use time::{macros::format_description, Duration, OffsetDateTime};

use crate::{gh::GithubApiError, hook::use_repos::RefetchFn, time::now};

#[inline_props]
pub fn github_api_error<'a>(
    cx: Scope,
    error: &'a GithubApiError,
    refetch: RefetchFn<'a>,
) -> Element {
    let error = match error {
        GithubApiError::Net(e) => rsx! {
            div { "{e}" }
        },
        GithubApiError::RateLimited { until } => rsx! {
            self::rate_limited {
                until: *until
            }
        },
    };

    cx.render(rsx! {
        div { style: "font-weight: bold", "Encountered error fetching github information"}
        error

        button {
            onclick: move |_e| { refetch() },
            "retry"
        }
    })
}

#[inline_props]
fn rate_limited(cx: Scope, until: OffsetDateTime) -> Element {
    let elapsed = use_state(&cx, || 0);
    let refresh = use_future(&cx, (), |()| {
        to_owned![elapsed];

        IntervalStream::new(1_000).for_each(move |()| {
            elapsed.modify(|e| e + 1);

            future::ready(())
        })
    });

    let duration = {
        let duration = until.sub(now());

        if duration <= Duration::ZERO {
            refresh.cancel(&cx);
            Duration::ZERO
        } else {
            duration
        }
    };

    let until = until
        .format(format_description!("[hour]:[minute]:[second] UTC"))
        .expect("failed to format date");

    let key = crate::gh::AUTH_LOCAL_STORAGE_KEY;

    cx.render(rsx! {
        div { "encountered a ratelimit, try again after {duration} (@ {until}) "}

        key.map(|key| {
            rsx! {
                hr {}
                div { "during development you can increase the api limit with a personal access token" }
                details {
                    summary { "tell me how" }

                    p { "place your personal access token in local storage with the following key" }
                    code { pre { "{key}" } }
                    p { "this key will be ignored when the application is not compiled with debug_assertions enabled" }
                }
            }
        })
    })
}
