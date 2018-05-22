extern crate config;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate structopt;
extern crate failure;

extern crate ansi_term;
extern crate chrono;
extern crate handlebars;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate jencli;

use handlebars::Handlebars;
use serde::Serialize;
use std::iter;

mod cli_config;
mod handlebars_helpers;

static HANDLEBARS_TEMPLATE: &'static str = "item_template";

fn main() -> Result<(), failure::Error> {
    env_logger::init();

    let opt = cli_config::load()?;

    let jenkins = jencli::JenkinsInformation {
        url: opt.url,
        user: opt.user,
        password: opt.password,
        depth: opt.depth,
    };

    let mut render = Handlebars::new();
    render.register_escape_fn(handlebars::no_escape);
    render.register_helper("colored", Box::new(handlebars_helpers::colored_status));
    render.register_helper("date", Box::new(handlebars_helpers::date));

    let output: Vec<String> = match opt.command {
        cli_config::CommandOpt::Search { pattern, template } => {
            render.register_template_string(HANDLEBARS_TEMPLATE, template)?;
            item_to_template(render, jencli::search_job(jenkins, &pattern)?).collect()
        }
        cli_config::CommandOpt::Job { name, template } => {
            render.register_template_string(HANDLEBARS_TEMPLATE, template)?;
            item_to_template(render, iter::once(jencli::get_job(jenkins, &name)?)).collect()
        }
        _ => unimplemented!(),
    };

    output.iter().for_each(|string| println!("{}", string));
    Ok(())
}

fn item_to_template<T, IT>(render: Handlebars, items: T) -> impl Iterator<Item = String>
where
    T: Iterator<Item = IT>,
    IT: Serialize,
{
    items
        .map(move |item| {
            debug!("{}", serde_json::to_string(&item).unwrap());
            render
                .render(HANDLEBARS_TEMPLATE, &item)
                .map(|s| s.replace("\\t", "\t"))
                .map(|s| s.replace("\\n", "\n"))
        })
        .filter_map(|result| result.ok())
}
