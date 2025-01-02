use chrono::{Duration, TimeDelta};

pub fn parse_time_delta_variable(s: &str) -> Option<TimeDelta> {
    let delta: Option<i64> = s[..s.len() - 1].parse().ok();
    let span: Option<char> = s.chars().last();

    let Some(delta) = delta else { return None };
    let Some(span) = span else { return None };

    match span {
        'w' => Some(Duration::weeks(delta)),
        'd' => Some(Duration::days(delta)),
        'h' => Some(Duration::hours(delta)),
        _ => None,
    }
}
