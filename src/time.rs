use js_sys::Date;
use time::OffsetDateTime;

pub fn now() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp((Date::now() as u64 / 1000) as _)
        .expect("unable to create time from epoch")
}
