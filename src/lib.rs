extern crate failure;
extern crate regex;

extern crate jenkins_api;

use regex::Regex;

pub struct JenkinsInformation {
    pub url: String,
    pub user: Option<String>,
    pub password: Option<String>,
    pub depth: u8,
}

fn build_jenkins_client(
    jenkins_info: JenkinsInformation,
) -> Result<jenkins_api::Jenkins, failure::Error> {
    let mut jenkins_builder = jenkins_api::JenkinsBuilder::new(&jenkins_info.url);
    if let Some(user) = jenkins_info.user {
        jenkins_builder =
            jenkins_builder.with_user(&user, jenkins_info.password.as_ref().map(|x| &**x));
    }
    Ok(jenkins_builder.with_depth(jenkins_info.depth).build()?)
}

pub fn search_job(
    jenkins_info: JenkinsInformation,
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
    jenkins_info: JenkinsInformation,
    name: &str,
) -> Result<jenkins_api::job::CommonJob, failure::Error> {
    let jenkins = build_jenkins_client(jenkins_info)?;

    Ok(jenkins.get_job(name)?)
}

pub fn get_build(
    jenkins_info: JenkinsInformation,
    name: &str,
    number: Option<u32>,
) -> Result<jenkins_api::build::CommonBuild, failure::Error> {
    let jenkins = build_jenkins_client(jenkins_info)?;

    match number {
        Some(n) => Ok(jenkins.get_build(name, n)?),
        None => Ok(jenkins.get_build(name, "lastBuild")?),
    }
}
