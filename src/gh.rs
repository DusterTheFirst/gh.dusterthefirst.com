use gloo_net::http::{Request, RequestCache, Response};
use log::{error, warn};
use time::OffsetDateTime;

pub enum GithubApiError {
    RateLimited { until: OffsetDateTime },
    Net(gloo_net::Error),
}

pub async fn fetch(url: &str) -> Result<Response, GithubApiError> {
    let response = Request::get(url)
        .header("accept", "application/vnd.github.v3+json")
        .cache(RequestCache::Default)
        .send()
        .await
        .map_err(GithubApiError::Net)?;

    let headers = response.headers();

    if response.status() == 403 {
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
