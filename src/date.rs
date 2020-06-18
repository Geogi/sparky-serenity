use anyhow::{bail, Error};
use chrono::{Date, Datelike, NaiveTime, TimeZone, Timelike, Weekday};
use fehler::throws;

pub const TZ_DEFAULT: chrono_tz::Tz = chrono_tz::Europe::Paris;

pub fn fr_month_to_str<T: TimeZone>(date: Date<T>) -> &'static str {
    match date.month() {
        1 => "janvier",
        2 => "février",
        3 => "mars",
        4 => "avril",
        5 => "mai",
        6 => "juin",
        7 => "juillet",
        8 => "août",
        9 => "septembre",
        10 => "octobre",
        11 => "novembre",
        12 => "décembre",
        _ => unreachable!("months range from 1 to 12"),
    }
}

pub fn fr_weekday_to_str(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "lundi",
        Weekday::Tue => "mardi",
        Weekday::Wed => "mercredi",
        Weekday::Thu => "jeudi",
        Weekday::Fri => "vendredi",
        Weekday::Sat => "samedi",
        Weekday::Sun => "dimanche",
    }
}

pub fn fr_day_to_str<T: TimeZone>(date: Date<T>) -> &'static str {
    match date.day() {
        1 => "1er",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        10 => "10",
        11 => "11",
        12 => "12",
        13 => "13",
        14 => "14",
        15 => "15",
        16 => "16",
        17 => "17",
        18 => "18",
        19 => "19",
        20 => "20",
        21 => "21",
        22 => "22",
        23 => "23",
        24 => "24",
        25 => "25",
        26 => "26",
        27 => "27",
        28 => "28",
        29 => "29",
        30 => "30",
        31 => "31",
        _ => unreachable!("days range from 1 to 31"),
    }
}

pub fn fr_weekday_to_emote(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "🇱",
        Weekday::Tue => "🇦",
        Weekday::Wed => "🇪",
        Weekday::Thu => "🇯",
        Weekday::Fri => "🇻",
        Weekday::Sat => "🇸",
        Weekday::Sun => "🇩",
    }
}

#[throws]
pub fn fr_weekday_from_shorthand(input: &str) -> Weekday {
    match input.to_lowercase().as_str() {
        "l" => Weekday::Mon,
        "a" => Weekday::Tue,
        "e" => Weekday::Wed,
        "j" => Weekday::Thu,
        "v" => Weekday::Fri,
        "s" => Weekday::Sat,
        "d" => Weekday::Sun,
        _ => bail!("invalid weekday identifier"),
    }
}

#[throws]
pub fn parse_time_emote_like(input: &str) -> NaiveTime {
    match input {
        "0" => NaiveTime::from_hms(0, 0, 0),
        "030" => NaiveTime::from_hms(0, 30, 0),
        "1" => NaiveTime::from_hms(1, 0, 0),
        "130" => NaiveTime::from_hms(1, 30, 0),
        "2" => NaiveTime::from_hms(2, 0, 0),
        "230" => NaiveTime::from_hms(2, 30, 0),
        "3" => NaiveTime::from_hms(3, 0, 0),
        "330" => NaiveTime::from_hms(3, 30, 0),
        "4" => NaiveTime::from_hms(4, 0, 0),
        "430" => NaiveTime::from_hms(4, 30, 0),
        "5" => NaiveTime::from_hms(5, 0, 0),
        "530" => NaiveTime::from_hms(5, 30, 0),
        "6" => NaiveTime::from_hms(6, 0, 0),
        "630" => NaiveTime::from_hms(6, 30, 0),
        "7" => NaiveTime::from_hms(7, 0, 0),
        "730" => NaiveTime::from_hms(7, 30, 0),
        "8" => NaiveTime::from_hms(8, 0, 0),
        "830" => NaiveTime::from_hms(8, 30, 0),
        "9" => NaiveTime::from_hms(9, 0, 0),
        "930" => NaiveTime::from_hms(9, 30, 0),
        "10" => NaiveTime::from_hms(10, 0, 0),
        "1030" => NaiveTime::from_hms(10, 30, 0),
        "11" => NaiveTime::from_hms(11, 0, 0),
        "1130" => NaiveTime::from_hms(11, 30, 0),
        "12" => NaiveTime::from_hms(12, 0, 0),
        "1230" => NaiveTime::from_hms(12, 30, 0),
        "13" => NaiveTime::from_hms(13, 0, 0),
        "1330" => NaiveTime::from_hms(13, 30, 0),
        "14" => NaiveTime::from_hms(14, 0, 0),
        "1430" => NaiveTime::from_hms(14, 30, 0),
        "15" => NaiveTime::from_hms(15, 0, 0),
        "1530" => NaiveTime::from_hms(15, 30, 0),
        "16" => NaiveTime::from_hms(16, 0, 0),
        "1630" => NaiveTime::from_hms(16, 30, 0),
        "17" => NaiveTime::from_hms(17, 0, 0),
        "1730" => NaiveTime::from_hms(17, 30, 0),
        "18" => NaiveTime::from_hms(18, 0, 0),
        "1830" => NaiveTime::from_hms(18, 30, 0),
        "19" => NaiveTime::from_hms(19, 0, 0),
        "1930" => NaiveTime::from_hms(19, 30, 0),
        "20" => NaiveTime::from_hms(20, 0, 0),
        "2030" => NaiveTime::from_hms(20, 30, 0),
        "21" => NaiveTime::from_hms(21, 0, 0),
        "2130" => NaiveTime::from_hms(21, 30, 0),
        "22" => NaiveTime::from_hms(22, 0, 0),
        "2230" => NaiveTime::from_hms(22, 30, 0),
        "23" => NaiveTime::from_hms(23, 0, 0),
        "2330" => NaiveTime::from_hms(23, 30, 0),
        _ => bail!("invalid emote-like time format"),
    }
}

#[throws]
pub fn time_emote(time: NaiveTime) -> &'static str {
    let hour12 = time.hour12().1;
    match (hour12, time.minute()) {
        (1, 0) => "🕐",
        (1, 30) => "🕜",
        (2, 0) => "🕑",
        (2, 30) => "🕝",
        (3, 0) => "🕒",
        (3, 30) => "🕞",
        (4, 0) => "🕓",
        (4, 30) => "🕟",
        (5, 0) => "🕔",
        (5, 30) => "🕠",
        (6, 0) => "🕕",
        (6, 30) => "🕡",
        (7, 0) => "🕖",
        (7, 30) => "🕢",
        (8, 0) => "🕗",
        (8, 30) => "🕣",
        (9, 0) => "🕘",
        (9, 30) => "🕤",
        (10, 0) => "🕙",
        (10, 30) => "🕥",
        (11, 0) => "🕚",
        (11, 30) => "🕦",
        (12, 0) => "🕛",
        (12, 30) => "🕧",
        _ => bail!("no emote available for this time"),
    }
}
