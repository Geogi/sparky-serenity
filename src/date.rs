use chrono::{Date, Datelike, Utc, Weekday};

pub fn fr_month(date: Date<Utc>) -> &'static str {
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

pub fn fr_weekday(date: Date<Utc>) -> &'static str {
    match date.weekday() {
        Weekday::Mon => "lundi",
        Weekday::Tue => "mardi",
        Weekday::Wed => "mercredi",
        Weekday::Thu => "jeudi",
        Weekday::Fri => "vendredi",
        Weekday::Sat => "samedi",
        Weekday::Sun => "dimanche",
    }
}

pub fn fr_day(date: Date<Utc>) -> &'static str {
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

pub fn weekday_emote(day: Weekday) -> &'static str {
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

pub fn weekday_from_fr(input: &str) -> Option<Weekday> {
    match input.to_lowercase().as_str() {
        "lundi" => Some(Weekday::Mon),
        "mardi" => Some(Weekday::Tue),
        "mercredi" => Some(Weekday::Wed),
        "jeudi" => Some(Weekday::Thu),
        "vendredi" => Some(Weekday::Fri),
        "samedi" => Some(Weekday::Sat),
        "dimanche" => Some(Weekday::Sun),
        _ => None,
    }
}
