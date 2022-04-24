use std::{future, ops::Sub};

use dioxus::{
    core::to_owned,
    prelude::{Scope, *},
};
use futures::StreamExt;
use gloo_timers::future::IntervalStream;
use instant::SystemTime;
use time::{macros::format_description, Duration, OffsetDateTime};

use crate::{data::RefetchFn, gh::GithubApiError};

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

    let now = time::OffsetDateTime::from_unix_timestamp(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("current time before unix epoch")
            .as_secs() as _,
    )
    .expect("unable to convert time from epoch");

    let duration = until.sub(now).max(Duration::ZERO);

    if duration.is_negative() {
        refresh.cancel(&cx);
    }

    let until = until
        .format(format_description!("[hour]:[minute]:[second] UTC"))
        .expect("failed to format date");

    cx.render(rsx! {
        div { "encountered a ratelimit, try again after {duration} (@ {until}) "}
    })
}
