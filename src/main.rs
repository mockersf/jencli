extern crate jenkins_api;

extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate structopt;

use structopt::StructOpt;

use config::{Config, ConfigError, Environment, File, FileFormat};
use std::env;
use std::ffi::OsString;

#[derive(StructOpt, Debug)]
enum CommandOpt {
    /// search for a job
    #[structopt(name = "search")]
    Search {
        /// pattern used to search through jobs name
        pattern: String,
        /// format of the output on stdout
        #[structopt(long = "tmpl", short = "t", default_value = "{{ .job.name }}")]
        template: String,
    },

    /// get informations about a job
    #[structopt(name = "job")]
    Job {
        /// exact name of the job
        name: String,
        /// format of the output on stdout
        #[structopt(long = "tmpl", short = "t",
                    default_value = "{{ .job.name }}: {{ .job.last_build.result }} ({{ .job.last_build.timestamp }})")]
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
        #[structopt(long = "tmpl", short = "t",
                    default_value = "{{ .build.result }} {{ .build.timestamp }}")]
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
                    default_value = "{{ .job.name }}: {{ .job.estimated_duration }}")]
        template: String,
    },

    /// list running jobs
    #[structopt(name = "running")]
    Running {
        /// also list queued jobs
        #[structopt(long = "queued")]
        queued: bool,
        /// format of the output on stdout
        #[structopt(long = "tmpl", short = "t", default_value = "{{ .queue_item.job_name }}")]
        template: String,
    },
}

#[derive(StructOpt, Debug)]
#[structopt(name = "jencli", author = "")]
struct ParamsOpt {
    /// Jenkins URL
    #[structopt(env = "JENKINS_URL")]
    url: String,
    #[structopt(env = "JENKINS_USER", short = "u", long = "user")]
    user: Option<String>,
    #[structopt(env = "JENKINS_PASSWORD", short = "p", long = "password")]
    password: Option<String>,
    #[structopt(flatten)]
    command: CommandOpt,
}

#[derive(Debug, Deserialize)]
struct Settings {
    url: Option<String>,
    user: Option<String>,
    password: Option<String>,
}

impl Settings {
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

fn main() {
    let settings = Settings::new().unwrap();
    println!("{:?}", settings);
    if let Some(url) = settings.url {
        env::set_var("JENKINS_URL", &url);
    }
    if let Some(user) = settings.user {
        env::set_var("JENKINS_USER", &user);
    }
    if let Some(password) = settings.password {
        env::set_var("JENKINS_PASSWORD", &password);
    }
    let opt = ParamsOpt::from_args();
    println!("{:?}", opt);
}
