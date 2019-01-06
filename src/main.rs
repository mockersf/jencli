extern crate config;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate structopt;
extern crate failure;

extern crate ansi_term;
extern crate chrono;
#[macro_use]
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
                &mut render,
                jencli::search_job(&jenkins, &pattern)?,
            )))
        }
        cli_config::CommandOpt::Job { name, template } => {
            render.register_template_string(HANDLEBARS_TEMPLATE, template)?;
            Ok(Box::new(item_to_template(
                &mut render,
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
                &mut render,
                iter::once(jencli::get_build(&jenkins, &name, number)?),
            )))
        }
        cli_config::CommandOpt::Views { pattern, template } => {
            render.register_template_string(HANDLEBARS_TEMPLATE, template)?;
            Ok(Box::new(item_to_template(
                &mut render,
                jencli::list_views(&jenkins, pattern)?,
            )))
        }
        cli_config::CommandOpt::View { name, template } => {
            render.register_template_string(HANDLEBARS_TEMPLATE, template)?;
            Ok(Box::new(item_to_template(
                &mut render,
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
                &mut render,
                command_trigger(jenkins, name, item, wait_start, wait_finish, polling),
            )))
        }
        cli_config::CommandOpt::Running {
            no_queued,
            template,
        } => {
            render.register_template_string(HANDLEBARS_TEMPLATE, template)?;

            let jenkins2 = jenkins.clone();

            let iter = item_to_template(
                &mut render,
                jencli::get_executors(&jenkins)?
                    .map(move |build| BuildAndQueue::from_build(&jenkins, build)),
            );

            if !no_queued {
                Ok(Box::new(iter.chain(item_to_template(
                    &mut render,
                    jencli::get_queue(&jenkins2)?.map(BuildAndQueue::from_queue_item),
                ))))
            } else {
                Ok(Box::new(iter))
            }
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BuildAndQueue {
    build: Option<EnrichedBuild>,
    queue_item: Option<jenkins_api::queue::QueueItem>,
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct EnrichedBuild {
    #[serde(flatten)]
    build: Option<jenkins_api::build::CommonBuild>,
    elapsed: Option<i64>,
    progress: Option<u32>,
    node: Option<String>,
}
impl BuildAndQueue {
    fn from_short_queue_item(
        jenkins: &jencli::JenkinsInformation,
        item: &jenkins_api::queue::ShortQueueItem,
        name: &str,
    ) -> Self {
        let queue = jencli::get_queue_item(&jenkins, &item).unwrap();
        let build = queue.executable.as_ref().map(|build| {
            let full_build = jencli::get_build(&jenkins, &name, Some(build.number)).unwrap();
            EnrichedBuild {
                elapsed: Some(Utc::now().timestamp() - full_build.timestamp as i64 / 1000),
                build: Some(full_build),
                node: None,
                progress: None,
            }
        });
        BuildAndQueue {
            build,
            queue_item: Some(queue),
        }
    }

    fn from_queue_item(item: jenkins_api::queue::QueueItem) -> Self {
        BuildAndQueue {
            build: None,
            queue_item: Some(item),
        }
    }

    fn from_build(_jenkins: &jencli::JenkinsInformation, build: jencli::BuildingOn) -> Self {
        // let queue = build
        //     .build
        //     .clone()
        //     .and_then(|build| jencli::get_queue_item_from_id(&jenkins, build.queue_id).ok());
        // println!("{:#?}", queue);
        let enriched_build = EnrichedBuild {
            elapsed: build
                .build
                .clone()
                .map(|build| Utc::now().timestamp() - build.timestamp as i64 / 1000),
            build: build.build,
            node: Some(build.node),
            progress: Some(build.progress),
        };
        BuildAndQueue {
            build: Some(enriched_build),
            queue_item: None,
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

    iter::once(BuildAndQueue::from_short_queue_item(&jenkins, &item, &name))
        .chain(iter::once(BuildAndQueue::from_short_queue_item(
            &jenkins, &item, &name,
        )))
        .chain(iter::repeat(moved_item).map(move |item| {
            thread::sleep(time::Duration::from_millis(polling * 1000));
            BuildAndQueue::from_short_queue_item(&moved_jenkins, &item, &moved_name)
        }))
        .enumerate()
        .take_while(move |(i, item)| {
            *i == 0
                || (wait_start
                    && !if let Some(ref qi) = item.queue_item {
                        qi.executable.is_some()
                    } else {
                        false
                    })
                || (wait_finish
                    && !item
                        .build
                        .as_ref()
                        .map(|build| {
                            if let Some(ref b) = build.build {
                                b.result.is_some()
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false))
        })
        .filter(|(i, _)| *i != 1)
        .map(|(_, item)| item)
        .chain(
            iter::once(())
                .map(move |_| BuildAndQueue::from_short_queue_item(&jenkins, &item, &name))
                .take_while(move |_| wait_finish || wait_start),
        )
}

fn item_to_template<T, IT>(render: &mut Handlebars, items: T) -> impl Iterator<Item = String>
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
                .ok()
        })
        .filter_map(|result| result)
        .collect::<Vec<_>>()
        .into_iter()
}
