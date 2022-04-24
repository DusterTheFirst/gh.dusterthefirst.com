use gloo_net::http::{Request, RequestCache, Response};
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

    Ok(response)
}
