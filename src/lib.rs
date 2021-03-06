use serde::Serialize;

use regex::Regex;

#[derive(Clone)]
pub struct JenkinsInformation {
    pub url: String,
    pub user: Option<String>,
    pub password: Option<String>,
    pub depth: u8,
}

fn build_jenkins_client(
    jenkins_info: &JenkinsInformation,
) -> Result<jenkins_api::Jenkins, failure::Error> {
    let mut jenkins_builder = jenkins_api::JenkinsBuilder::new(&jenkins_info.url);
    jenkins_builder = match (&jenkins_info.user, &jenkins_info.password) {
        (Some(ref user), None) => jenkins_builder.with_user(user, None),
        (Some(ref user), Some(ref password)) => jenkins_builder.with_user(user, Some(password)),
        (_, _) => jenkins_builder,
    };
    Ok(jenkins_builder.with_depth(jenkins_info.depth).build()?)
}

pub fn search_job(
    jenkins_info: &JenkinsInformation,
    pattern: &str,
) -> Result<impl Iterator<Item = jenkins_api::job::ShortJob>, failure::Error> {
    let jenkins = build_jenkins_client(jenkins_info)?;

    let re = Regex::new(pattern).unwrap();

    Ok(jenkins
        .get_home()?
        .jobs
        .into_iter()
        .filter(move |job| re.is_match(&job.name)))
}

pub fn get_job(
    jenkins_info: &JenkinsInformation,
    name: &str,
) -> Result<jenkins_api::job::CommonJob, failure::Error> {
    let jenkins = build_jenkins_client(jenkins_info)?;

    Ok(jenkins.get_job(name)?)
}

pub fn get_build(
    jenkins_info: &JenkinsInformation,
    name: &str,
    number: Option<u32>,
) -> Result<jenkins_api::build::CommonBuild, failure::Error> {
    let jenkins = build_jenkins_client(jenkins_info)?;

    match number {
        Some(n) => Ok(jenkins.get_build(name, n)?),
        None => Ok(jenkins.get_build(name, "lastBuild")?),
    }
}

pub fn list_views(
    jenkins_info: &JenkinsInformation,
    pattern: Option<String>,
) -> Result<impl Iterator<Item = jenkins_api::view::ShortView>, failure::Error> {
    let jenkins = build_jenkins_client(jenkins_info)?;

    let views = jenkins.get_home()?.views.into_iter();

    match pattern {
        Some(pattern) => {
            let re = Regex::new(&pattern).unwrap();
            Ok(views
                .filter(move |view| re.is_match(&view.name))
                .collect::<Vec<jenkins_api::view::ShortView>>()
                .into_iter())
        }
        None => Ok(views),
    }
}

pub fn list_jobs_of_view(
    jenkins_info: &JenkinsInformation,
    name: &str,
) -> Result<impl Iterator<Item = jenkins_api::job::ShortJob>, failure::Error> {
    let jenkins = build_jenkins_client(jenkins_info)?;

    Ok(jenkins.get_view(name)?.jobs.into_iter())
}

pub fn trigger_job(
    jenkins_info: &JenkinsInformation,
    name: &str,
) -> Result<jenkins_api::queue::ShortQueueItem, failure::Error> {
    let jenkins = build_jenkins_client(jenkins_info)?;

    Ok(jenkins.build_job(name)?)
}

pub fn get_queue_item(
    jenkins_info: &JenkinsInformation,
    queue_item: &jenkins_api::queue::ShortQueueItem,
) -> Result<jenkins_api::queue::QueueItem, failure::Error> {
    let jenkins = build_jenkins_client(jenkins_info)?;

    Ok(queue_item.get_full_queue_item(&jenkins)?)
}

pub fn get_queue_item_from_id(
    jenkins_info: &JenkinsInformation,
    id: i32,
) -> Result<jenkins_api::queue::QueueItem, failure::Error> {
    let jenkins = build_jenkins_client(jenkins_info)?;

    Ok(jenkins.get_queue_item(id)?)
}

pub fn get_queue(
    jenkins_info: &JenkinsInformation,
) -> Result<impl Iterator<Item = jenkins_api::queue::QueueItem>, failure::Error> {
    let jenkins = build_jenkins_client(jenkins_info)?;

    Ok(jenkins.get_queue()?.items.into_iter())
}

#[derive(Debug, Serialize)]
pub struct BuildingOn {
    pub node: String,
    pub progress: u32,
    pub build: Option<jenkins_api::build::CommonBuild>,
}

pub fn get_executors(
    jenkins_info: &JenkinsInformation,
) -> Result<impl Iterator<Item = BuildingOn>, failure::Error> {
    let jenkins = build_jenkins_client(jenkins_info)?;

    Ok(jenkins
        .get_nodes()?
        .computers
        .into_iter()
        .flat_map(move |computer| {
            let node_name = computer.display_name.clone();
            let jenkins = &jenkins;
            computer
                .executors
                .iter()
                .filter_map(move |executor| {
                    if let jenkins_api::nodes::computer::Executor::Executor {
                        current_executable,
                        progress,
                        ..
                    } = executor
                    {
                        let node_name = node_name.clone();

                        if let jenkins_api::nodes::computer::ExecutorProgress::Percent(p) = progress
                        {
                            Some(BuildingOn {
                                node: node_name,
                                progress: *p,
                                build: current_executable
                                    .clone()
                                    .map(|exec| exec.get_full_build(jenkins).unwrap()),
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .into_iter())
}
