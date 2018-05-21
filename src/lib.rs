extern crate failure;
extern crate regex;

extern crate jenkins_api;

use regex::Regex;

pub struct JenkinsInformation {
    pub url: String,
    pub user: Option<String>,
    pub password: Option<String>,
}

pub fn search_job(jenkins_info: JenkinsInformation, pattern: &str) -> Result<(), failure::Error> {
    let mut jenkins_builder = jenkins_api::JenkinsBuilder::new(&jenkins_info.url);
    if let Some(user) = jenkins_info.user {
        jenkins_builder =
            jenkins_builder.with_user(&user, jenkins_info.password.as_ref().map(|x| &**x));
    }
    let jenkins = jenkins_builder.build()?;

    let re = Regex::new(pattern).unwrap();
    let matching_jobs: Vec<jenkins_api::ShortJob> = jenkins
        .get_home()?
        .jobs
        .iter()
        .filter(|job| re.is_match(&job.name))
        .cloned()
        .collect();
    println!("{:#?}", matching_jobs);
    Ok(())
}
