use chrono::NaiveDate;
use serde::{de::IntoDeserializer, Deserialize};

pub fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    let opt = opt.as_deref();
    match opt {
        None | Some("") => Ok(None),
        Some(s) => T::deserialize(s.into_deserializer()).map(Some),
    }
}

pub fn date_time_as_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let Ok(t) = String::deserialize(deserializer) else {
        return Ok(None);
    };

    match NaiveDate::parse_from_str(&t, "%Y-%m-%dT%H:%M:%S%.fZ") {
        Ok(d) => Ok(Some(d)),
        _ => Ok(None),
    }
}
