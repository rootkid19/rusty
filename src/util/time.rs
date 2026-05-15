use anyhow::Result;
use serde::{Deserialize, Deserializer, Serializer};
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

const NOTE_ID_TIMESTAMP: &[time::format_description::FormatItem<'static>] =
    format_description!("[year][month][day]T[hour][minute][second]Z");
const HUMAN_TIME_TODAY: &[time::format_description::FormatItem<'static>] =
    format_description!("today [hour]:[minute]");
const HUMAN_TIME_THIS_YEAR: &[time::format_description::FormatItem<'static>] =
    format_description!("[month repr:short] [day] [hour]:[minute]");
const HUMAN_TIME_FULL: &[time::format_description::FormatItem<'static>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]");

pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

pub fn format_note_id_timestamp(value: OffsetDateTime) -> String {
    value
        .format(NOTE_ID_TIMESTAMP)
        .expect("note id timestamp format is valid")
}

pub fn format_rfc3339(value: OffsetDateTime) -> String {
    value.format(&Rfc3339).expect("rfc3339 format is valid")
}

pub fn format_human_time(value: OffsetDateTime) -> String {
    let local = to_local(value);
    let now = to_local(now_utc());

    if local.date() == now.date() {
        return local
            .format(HUMAN_TIME_TODAY)
            .expect("today format is valid");
    }

    if local.year() == now.year() {
        return local
            .format(HUMAN_TIME_THIS_YEAR)
            .expect("year format is valid");
    }

    local
        .format(HUMAN_TIME_FULL)
        .expect("full human format is valid")
}

pub fn local_date(value: OffsetDateTime) -> Date {
    to_local(value).date()
}

pub fn today_local() -> Date {
    local_date(now_utc())
}

pub fn start_of_day_utc(date: Date) -> Result<OffsetDateTime> {
    let midnight = Time::from_hms(0, 0, 0)?;
    Ok(PrimitiveDateTime::new(date, midnight).assume_utc())
}

pub fn start_of_day_local(date: Date) -> Result<OffsetDateTime> {
    let midnight = Time::from_hms(0, 0, 0)?;
    let local = PrimitiveDateTime::new(date, midnight);
    match UtcOffset::current_local_offset() {
        Ok(offset) => Ok(local.assume_offset(offset).to_offset(UtcOffset::UTC)),
        Err(_) => Ok(local.assume_utc()),
    }
}

pub fn month_to_number(month: Month) -> u8 {
    match month {
        Month::January => 1,
        Month::February => 2,
        Month::March => 3,
        Month::April => 4,
        Month::May => 5,
        Month::June => 6,
        Month::July => 7,
        Month::August => 8,
        Month::September => 9,
        Month::October => 10,
        Month::November => 11,
        Month::December => 12,
    }
}

fn to_local(value: OffsetDateTime) -> OffsetDateTime {
    match UtcOffset::current_local_offset() {
        Ok(offset) => value.to_offset(offset),
        Err(_) => value,
    }
}

pub mod serde_rfc3339 {
    use super::*;

    pub fn serialize<S>(
        value: &OffsetDateTime,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format_rfc3339(*value))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        OffsetDateTime::parse(&value, &Rfc3339).map_err(serde::de::Error::custom)
    }
}
