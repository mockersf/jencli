use ansi_term::Colour;
use chrono::prelude::*;
use handlebars::{Handlebars, Helper, RenderContext, RenderError};

pub fn date(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let param = h.param(0).unwrap().value();
    if let Some(ts) = param.as_i64() {
        let naive_datetime = NaiveDateTime::from_timestamp(ts / 1000, 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
        let rendered = format!("{}", datetime);
        try!(rc.writer.write_all(rendered.into_bytes().as_ref()));
    }
    Ok(())
}

pub fn colored_status(
    h: &Helper,
    _: &Handlebars,
    rc: &mut RenderContext,
) -> Result<(), RenderError> {
    let param = h.param(0).unwrap().value();
    let to_render = match param.as_str() {
        Some("blue") => Colour::Blue.paint("blue").to_string(),
        Some("blue_anime") => Colour::Blue.bold().paint("blue").to_string(),
        Some("yellow") => Colour::Yellow.paint("yellow").to_string(),
        Some("yellow_anime") => Colour::Yellow.bold().paint("yellow").to_string(),
        Some("red") => Colour::Red.paint("red").to_string(),
        Some("red_anime") => Colour::Red.bold().paint("red").to_string(),
        Some("aborted") => Colour::White.dimmed().paint("disabled").to_string(),
        Some("disabled") => Colour::White.dimmed().paint("disabled").to_string(),
        Some("notbuilt") => Colour::White.dimmed().paint("notbuilt").to_string(),
        Some("SUCCESS") => Colour::Blue.paint("SUCCESS").to_string(),
        Some("UNSTABLE") => Colour::Yellow.paint("UNSTABLE").to_string(),
        Some("FAILURE") => Colour::Red.paint("FAILURE").to_string(),
        Some("ABORTED") => Colour::White.dimmed().paint("ABORTED").to_string(),
        Some("NOT_BUILT") => Colour::White.dimmed().paint("NOT_BUILT").to_string(),
        Some(x) => x.to_string(),
        None => "".to_string(),
    };
    let rendered = to_render.to_string();
    try!(rc.writer.write_all(rendered.into_bytes().as_ref()));
    Ok(())
}
