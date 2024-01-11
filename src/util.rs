use chrono::{DateTime, NaiveDateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};
pub(crate) fn system_time_to_date_time(t: SystemTime) -> Option<DateTime<Utc>> {
    // secs is already relative to utc after this match
    let (sec, nsec) = match t.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
        Err(e) => { // unlikely but should be handled
            let dur = e.duration();
            let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
            if nsec == 0 {
                (-sec, 0)
            } else {
                (-sec - 1, 1_000_000_000 - nsec)
            }
        },
    };
    NaiveDateTime::from_timestamp_opt(sec, nsec)
        .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
}