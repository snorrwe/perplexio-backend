pub mod game;
pub mod participation;
pub mod puzzle;
pub mod solution;
pub mod user;
pub mod vector;

pub mod datetime_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M";

    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = if let Some(date) = date {
            format!("{}", date.format(FORMAT))
        } else {
            format!("null")
        };
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "null" {
            return Ok(None);
        }
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
            .map(|date| Some(date))
    }

}

