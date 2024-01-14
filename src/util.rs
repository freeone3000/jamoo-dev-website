use std::path::Path;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;

pub(crate) fn path_time_modified(path: &Path) -> SystemTime {
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .unwrap_or(UNIX_EPOCH)
}

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

const URL_UNRESERVED_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789-_.~";
const HEX: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'A', 'B', 'C', 'D', 'E', 'F'];

/// I didn't want to pull in a dependency for this
pub(crate) fn my_urlencode(input: &[u8]) -> String {
    let mut accum = Vec::with_capacity(input.len());
    for b in input {
        if URL_UNRESERVED_CHARS.contains(&b) {
            accum.push(*b as char)
        } else {
            accum.push('%');
            accum.push(HEX[((*b as usize) >> 4) & 0x0F]);
            accum.push(HEX[(*b as usize) & 0x0F]);
        }
    }
    accum.into_iter().collect()
}

pub(crate) fn my_urldecode<S: IntoIterator<Item=char>>(input: S) -> Result<Vec<u8>> {
    fn decode_half_byte(c: Option<char>) -> Result<u8> {
        match c {
            None => Err(anyhow::anyhow!("Invalid hex digit: EOF")),
            Some(c) => match c {
                '0'..='9' => Ok(c as u8 - b'0'),
                'A'..='F' => Ok(c as u8 - b'A' + 10),
                'a'..='f' => Ok(c as u8 - b'a' + 10),
                _ => Err(anyhow::anyhow!("Invalid hex digit: {}", c)),
            }
        }
    }

    let mut iter = input.into_iter();
    let mut output = Vec::new();
    while let Some(c) = iter.next() {
        if c == '%' {
            let byte: u8 = (decode_half_byte(iter.next())? << 4) | (decode_half_byte(iter.next())?);
            output.push(byte);
        } else {
            output.push(c as u8);
        }
    }
    Ok(output)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_my_urlencode() {
        assert_eq!(my_urlencode(b"hello world"), "hello%20world");
        assert_eq!(my_urlencode(b"hello world!"), "hello%20world%21");
    }

    #[test]
    fn test_my_urldecode() {
        assert_eq!(my_urldecode("hello%20world".chars()).unwrap(), b"hello world");
        assert_eq!(my_urldecode("hello%20world%21".chars()).unwrap(), b"hello world!");
    }
}