use gloo_net::http::{Request, RequestCache, Response};
use log::{error, warn};
use once_cell::sync::Lazy;
use time::{Date, OffsetDateTime};

pub enum GithubApiError {
    RateLimited { until: OffsetDateTime },
    Net(gloo_net::Error),
}

#[cfg(debug_assertions)]
pub const AUTH_LOCAL_STORAGE_KEY: Option<&str> =
    Some(concat!(env!("CARGO_PKG_NAME"), "-gh-personal-token"));

#[cfg(debug_assertions)]
static PERSONAL_ACCESS_TOKEN: Lazy<Option<String>> = Lazy::new(|| {
    use web_sys::window;

    let key = AUTH_LOCAL_STORAGE_KEY.expect("AUTH_LOCAL_STORAGE_KEY must exist");

    if let Some(local_storage) = window()
        .expect("failed to get window object")
        .local_storage()
        .expect("failed to get local storage")
    {
        local_storage
            .get_item(key)
            .expect("failed to get local storage item")
    } else {
        warn!("Browser does not support local storage, bruh how are you running wasm in IE6?");

        None
    }
});

#[cfg(not(debug_assertions))]
pub const AUTH_LOCAL_STORAGE_KEY: Option<&str> = None;

#[cfg(not(debug_assertions))]
static PERSONAL_ACCESS_TOKEN: Lazy<Option<String>> = Lazy::new(|| None);

pub async fn fetch(url: &str) -> Result<Response, GithubApiError> {
    let request = Request::get(url)
        .header("accept", "application/vnd.github.v3+json")
        .cache(RequestCache::Default);

    // Attach personal access token if one provided
    let request = if let Some(token) = PERSONAL_ACCESS_TOKEN.as_ref() {
        request.header("Authorization", &format!("token {token}"))
    } else {
        request
    };

    let response = request.send().await.map_err(GithubApiError::Net)?;

    let headers = response.headers();

    match response.status() {
        200 => {}
        403 => {
            let until = OffsetDateTime::from_unix_timestamp(
                headers
                    .get("x-ratelimit-reset")
                    .expect("x-ratelimit-reset header missing")
                    .as_str()
                    .parse()
                    .expect("x-ratelimit-reset provided as a non-integer"),
            )
            .expect("x-ratelimit-reset provided an invalid unix timestamp");

            error!("Ratelimit encountered. Retry after {until}");

            return Err(GithubApiError::RateLimited { until });
        }
        401 => {
            error!("Provided personal access token was rejected by github");

            // TODO: bother with a new error type? this should only ever happen in development.
            return Err(GithubApiError::RateLimited {
                until: Date::MAX.midnight().assume_utc(),
            });
        }
        code => unimplemented!("Encountered unknown http status code: {code}"),
    }

    let remaining: u32 = headers
        .get("x-ratelimit-remaining")
        .expect("x-ratelimit-remaining header missing")
        .as_str()
        .parse()
        .expect("x-ratelimit-remaining provided as a non-integer");

    if remaining < 10 {
        let limit: u32 = headers
            .get("x-ratelimit-limit")
            .expect("x-ratelimit-limit header missing")
            .as_str()
            .parse()
            .expect("x-ratelimit-limit provided as a non-integer");

        warn!("Getting close to the rate limit {remaining}/{limit} requests left");
    }

    Ok(response)
}
