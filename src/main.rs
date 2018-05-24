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
extern crate jenkins_api;

use chrono::Utc;
use handlebars::Handlebars;
use serde::Serialize;
use std::iter;

use std::{thread, time};

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

    let output = command_to_iter(jenkins, render, opt.command)?;

    output.for_each(|string| println!("{}", string));
    Ok(())
}

fn command_to_iter(
    jenkins: jencli::JenkinsInformation,
    mut render: Handlebars,
    command: cli_config::CommandOpt,
) -> Result<Box<Iterator<Item = String>>, failure::Error> {
    match command {
        cli_config::CommandOpt::Search { pattern, template } => {
            render.register_template_string(HANDLEBARS_TEMPLATE, template)?;
            Ok(Box::new(item_to_template(
                render,
                jencli::search_job(&jenkins, &pattern)?,
            )))
        }
        cli_config::CommandOpt::Job { name, template } => {
            render.register_template_string(HANDLEBARS_TEMPLATE, template)?;
            Ok(Box::new(item_to_template(
                render,
                iter::once(jencli::get_job(&jenkins, &name)?),
            )))
        }
        cli_config::CommandOpt::Build {
            name,
            number,
            template,
        } => {
            render.register_template_string(HANDLEBARS_TEMPLATE, template)?;
            Ok(Box::new(item_to_template(
                render,
                iter::once(jencli::get_build(&jenkins, &name, number)?),
            )))
        }
        cli_config::CommandOpt::Views { pattern, template } => {
            render.register_template_string(HANDLEBARS_TEMPLATE, template)?;
            Ok(Box::new(item_to_template(
                render,
                jencli::list_views(&jenkins, pattern)?,
            )))
        }
        cli_config::CommandOpt::View { name, template } => {
            render.register_template_string(HANDLEBARS_TEMPLATE, template)?;
            Ok(Box::new(item_to_template(
                render,
                jencli::list_jobs_of_view(&jenkins, &name)?,
            )))
        }
        cli_config::CommandOpt::Trigger {
            name,
            wait_start,
            wait_finish,
            polling,
            template,
        } => {
            render.register_template_string(HANDLEBARS_TEMPLATE, template)?;
            let item = jencli::trigger_job(&jenkins, &name)?;

            Ok(Box::new(item_to_template(
                render,
                command_trigger(jenkins, name, item, wait_start, wait_finish, polling),
            )))
        }
        _ => unimplemented!(),
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BuildAndQueue {
    build: Option<EnrichedBuild>,
    queue_item: jenkins_api::queue::QueueItem,
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct EnrichedBuild {
    #[serde(flatten)]
    build: jenkins_api::build::CommonBuild,
    elapsed: i64,
}
impl BuildAndQueue {
    fn from(
        jenkins: &jencli::JenkinsInformation,
        item: &jenkins_api::queue::ShortQueueItem,
        name: &str,
    ) -> Self {
        let queue = jencli::get_queue_item(&jenkins, &item).unwrap();
        let build = queue.executable.as_ref().map(|build| {
            let full_build = jencli::get_build(&jenkins, &name, Some(build.number)).unwrap();
            EnrichedBuild {
                elapsed: Utc::now().timestamp() - full_build.timestamp as i64 / 1000,
                build: full_build,
            }
        });
        BuildAndQueue {
            build,
            queue_item: queue,
        }
    }
}
fn command_trigger(
    jenkins: jencli::JenkinsInformation,
    name: String,
    item: jenkins_api::queue::ShortQueueItem,
    wait_start: bool,
    wait_finish: bool,
    polling: u64,
) -> impl Iterator<Item = impl Serialize> {
    let moved_jenkins = jenkins.clone();
    let moved_name = name.clone();
    let moved_item = item.clone();

    iter::once(BuildAndQueue::from(&jenkins, &item, &name))
        .chain(iter::once(BuildAndQueue::from(&jenkins, &item, &name)))
        .chain(iter::repeat(moved_item).map(move |item| {
            thread::sleep(time::Duration::from_millis(polling * 1000));
            BuildAndQueue::from(&moved_jenkins, &item, &moved_name)
        }))
        .enumerate()
        .take_while(move |(i, item)| {
            *i == 0 || (wait_start && !item.queue_item.executable.is_some())
                || (wait_finish
                    && !item.build
                        .as_ref()
                        .map(|build| build.build.result.is_some())
                        .unwrap_or(false))
        })
        .filter(|(i, _)| *i != 1)
        .map(|(_, item)| item)
        .chain(
            iter::once(())
                .map(move |_| BuildAndQueue::from(&jenkins, &item, &name))
                .take_while(move |_| wait_finish || wait_start),
        )
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
