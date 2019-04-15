#![allow(clippy::redundant_closure)]

use ansi_term::Colour;
use chrono::prelude::*;
use handlebars::handlebars_helper;

handlebars_helper!(date: |ts: i64| {
    let naive_datetime = NaiveDateTime::from_timestamp(ts / 1000, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
    format!("{}", datetime)
});

handlebars_helper!(colored_status: |status: str| {
    match status {
        "blue" => Colour::Blue.paint("blue").to_string(),
        "blue_anime" => Colour::Blue.bold().paint("blue").to_string(),
        "yellow" => Colour::Yellow.paint("yellow").to_string(),
        "yellow_anime" => Colour::Yellow.bold().paint("yellow").to_string(),
        "red" => Colour::Red.paint("red").to_string(),
        "red_anime" => Colour::Red.bold().paint("red").to_string(),
        "aborted" => Colour::White.dimmed().paint("disabled").to_string(),
        "disabled" => Colour::White.dimmed().paint("disabled").to_string(),
        "notbuilt" => Colour::White.dimmed().paint("notbuilt").to_string(),
        "SUCCESS" => Colour::Blue.paint("SUCCESS").to_string(),
        "UNSTABLE" => Colour::Yellow.paint("UNSTABLE").to_string(),
        "FAILURE" => Colour::Red.paint("FAILURE").to_string(),
        "ABORTED" => Colour::White.dimmed().paint("ABORTED").to_string(),
        "NOT_BUILT" => Colour::White.dimmed().paint("NOT_BUILT").to_string(),
        x => x.to_string(),
    }
});
