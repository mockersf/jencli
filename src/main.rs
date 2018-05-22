extern crate config;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;
extern crate failure;

extern crate handlebars;

extern crate env_logger;

extern crate jencli;

use config::{Config, ConfigError, Environment, File, FileFormat};
use std::env;
use std::ffi::OsString;
use structopt::StructOpt;

use handlebars::Handlebars;

#[derive(StructOpt, Debug)]
enum CommandOpt {
    /// search for a job
    #[structopt(name = "search")]
    Search {
        /// pattern used to search through jobs name
        pattern: String,
        /// format of the output on stdout
        #[structopt(long = "tmpl", short = "t", default_value = "{{ name }} - {{ color }}")]
        template: String,
    },

    /// get informations about a job
    #[structopt(name = "job")]
    Job {
        /// exact name of the job
        name: String,
        /// format of the output on stdout
        #[structopt(long = "tmpl", short = "t",
                    default_value = "{{ name }} - {{ color }} (#{{ lastBuild.number }})")]
        template: String,
    },

    /// get informations about a build
    #[structopt(name = "build")]
    Build {
        /// name of the job
        name: String,
        /// number of the build, will fetch latest if not specified
        number: Option<u32>,
        /// format of the output on stdout
        #[structopt(long = "tmpl", short = "t", default_value = "{{ result }} {{ timestamp }}")]
        template: String,
    },

    /// trigger a job
    #[structopt(name = "trigger")]
    Trigger {
        /// exact name of the job
        name: String,
        /// wait for the job to start before returning
        #[structopt(long = "wait-start")]
        wait_start: bool,
        /// wait for the job to finish before returning
        #[structopt(long = "wait-finish")]
        wait_finish: bool,
        /// check job status every X seconds, and display status with every check
        #[structopt(long = "polling", default_value = "10")]
        polling: u32,
        /// format of the output on stdout
        #[structopt(long = "tmpl", short = "t",
                    default_value = "{{ name }}: {{ estimated_duration }}")]
        template: String,
    },

    /// list running jobs
    #[structopt(name = "running")]
    Running {
        /// also list queued jobs
        #[structopt(long = "queued")]
        queued: bool,
        /// format of the output on stdout
        #[structopt(long = "tmpl", short = "t", default_value = "{{ job_name }}")]
        template: String,
    },
}

#[derive(StructOpt, Debug)]
#[structopt(name = "jencli", author = "")]
struct ParamsOpt {
    /// Jenkins URL
    #[structopt(env = "JENKINS_URL", long = "url")]
    url: String,
    /// Jenkins user
    #[structopt(env = "JENKINS_USER", long = "user")]
    user: Option<String>,
    /// Jenkins password
    #[structopt(env = "JENKINS_PASSWORD", long = "password")]
    password: Option<String>,
    /// Amount of data retrieved from Jenkins
    #[structopt(env = "JENKINS_DEPTH", long = "depth", default_value = "1")]
    depth: u8,
    #[structopt(flatten)]
    command: CommandOpt,
}

#[derive(Debug, Deserialize)]
struct JenkinsSettings {
    url: Option<String>,
    user: Option<String>,
    password: Option<String>,
    depth: Option<u8>,
}

impl JenkinsSettings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut config = Config::new();

        // Load file from home directory
        config.merge(File::new("/Users/francoism/.jencli.yaml", FileFormat::Yaml).required(false))?;

        // Load file from any folder in the path
        let mut current_dir = env::current_dir().unwrap();
        let mut pathes = vec![current_dir.clone().into_os_string()];
        while current_dir.pop() {
            pathes.push(current_dir.clone().into_os_string());
        }
        for dir in pathes.iter().rev() {
            let mut file = dir.clone();
            file.push(OsString::from("/.jencli.yaml"));
            let file_path = file.to_str().unwrap();
            config.merge(File::new(file_path, FileFormat::Yaml).required(false))?;
        }

        // Load from environment
        config.merge(Environment::with_prefix("jenkins"))?;

        config.try_into()
    }
}

fn main() -> Result<(), failure::Error> {
    env_logger::init();

    let jenkins_settings = JenkinsSettings::new()?;
    if let Some(url) = jenkins_settings.url {
        env::set_var("JENKINS_URL", &url);
    }
    if let Some(user) = jenkins_settings.user {
        env::set_var("JENKINS_USER", &user);
    }
    if let Some(password) = jenkins_settings.password {
        env::set_var("JENKINS_PASSWORD", &password);
    }
    if let Some(depth) = jenkins_settings.depth {
        env::set_var("JENKINS_DEPTH", &depth.to_string());
    }
    let opt = ParamsOpt::from_args();

    let jenkins = jencli::JenkinsInformation {
        url: opt.url,
        user: opt.user,
        password: opt.password,
        depth: opt.depth,
    };

    let reg = Handlebars::new();

    let output: Vec<String> = match opt.command {
        CommandOpt::Search { pattern, template } => jencli::search_job(jenkins, &pattern)?
            .map(|job| reg.render_template(&template, &job))
            .filter_map(|result| result.ok())
            .collect(),
        CommandOpt::Job { name, template } => vec![jencli::get_job(jenkins, &name)?]
            .iter()
            .map(|job| reg.render_template(&template, &job))
            .filter_map(|result| result.ok())
            .collect(),
        _ => unimplemented!(),
    };

    output.iter().for_each(|string| println!("{}", string));
    Ok(())
}
