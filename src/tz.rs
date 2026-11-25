//! The local UTC offset, without a time-zone database.
//!
//! The standard library can't tell you the local offset, and a whole time
//! crate felt heavy for one number. `date +%z` exists on every Unix this tool
//! targets and already knows the user's zone, so ask it.

/// Local offset from UTC in seconds, or `None` if it can't be determined.
pub fn local_offset() -> Option<i64> {
    let out = std::process::Command::new("date")
        .arg("+%z")
        .output()
        .ok()?;
    parse_offset(std::str::from_utf8(&out.stdout).ok()?.trim())
}

/// Parse `+0530` / `-0800` style offsets.
fn parse_offset(s: &str) -> Option<i64> {
    let (sign, digits) = match s.as_bytes().first()? {
        b'+' => (1, &s[1..]),
        b'-' => (-1, &s[1..]),
        _ => return None,
    };
    if digits.len() != 4 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let hours: i64 = digits[..2].parse().ok()?;
    let minutes: i64 = digits[2..].parse().ok()?;
    (minutes < 60).then_some(sign * (hours * 3600 + minutes * 60))
}

#[cfg(test)]
mod tests {
    use super::parse_offset;

    #[test]
    fn parses_common_offsets() {
        assert_eq!(parse_offset("+0530"), Some(19_800));
        assert_eq!(parse_offset("-0800"), Some(-28_800));
        assert_eq!(parse_offset("+0000"), Some(0));
        assert_eq!(parse_offset("+0545"), Some(20_700));
    }

    #[test]
    fn rejects_garbage() {
        for s in ["", "0530", "+530", "+05:30", "+0575", "IST"] {
            assert_eq!(parse_offset(s), None, "{s:?}");
        }
    }
}
