use std::collections::HashMap;
use std::env;

use config::{Config, ConfigError, Environment, Source, Value};
use serde::Deserialize;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum CommandOpt {
    /// search for a job
    #[structopt(name = "search")]
    Search {
        /// pattern used to search through jobs name
        pattern: String,
        /// format of the output on stdout
        #[structopt(
            long = "tmpl",
            short = "t",
            default_value = "{{ name }}\t{{colored color }}"
        )]
        template: String,
    },

    /// get informations about a job
    #[structopt(name = "job")]
    Job {
        /// exact name of the job
        name: String,
        /// format of the output on stdout
        #[structopt(
            long = "tmpl",
            short = "t",
            default_value = "{{ name }} - {{colored color }} (#{{ lastBuild.number }})"
        )]
        template: String,
    },

    /// get informations about a build
    #[structopt(name = "build")]
    Build {
        /// name of the job
        name: String,
        /// number of the build, will fetch lastBuild if not specified
        number: Option<u32>,
        /// format of the output on stdout
        #[structopt(
            long = "tmpl",
            short = "t",
            default_value = "{{ fullDisplayName}} {{colored result }} {{date timestamp }} ({{duration}}ms)"
        )]
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
        polling: u64,
        /// format of the output on stdout
        #[structopt(
            long = "tmpl",
            short = "t",
            default_value = "{{ queueItem.task.name }} {{#if queueItem.why}}{{ queueItem.why }}{{/if}}{{#if queueItem.executable}}{{ build.displayName }} {{colored build.result }} {{build.elapsed}}s (est. {{ build.estimatedDuration }}ms){{/if}}"
        )]
        template: String,
    },

    /// list running jobs
    #[structopt(name = "running")]
    Running {
        /// do not list queued jobs
        #[structopt(long = "no-queued")]
        no_queued: bool,
        /// format of the output on stdout
        #[structopt(
            long = "tmpl",
            short = "t",
            default_value = "{{#if queueItem}}{{ queueItem.task.name }} {{#if queueItem.why}}{{ queueItem.why }}{{/if}}{{/if}}{{#if build}}{{#if build.fullDisplayName}}{{ build.fullDisplayName }}{{else}}Unknown Task{{/if}}{{#if build.result}} {{colored build.result }}{{/if}} {{#if build.elapsed}}{{build.elapsed}}s {{/if}}{{#if build.estimatedDuration}}(est. {{ build.estimatedDuration }}ms) {{/if}}- {{ build.progress }}% on {{ build.node}} {{/if}}"
        )]
        template: String,
    },

    /// list views
    #[structopt(name = "views")]
    Views {
        /// pattern used to search through views name
        pattern: Option<String>,
        /// format of the output on stdout
        #[structopt(long = "tmpl", short = "t", default_value = "{{ name }}")]
        template: String,
    },

    /// list jobs of a view
    #[structopt(name = "view")]
    View {
        /// exact name of the view
        name: String,
        /// format of the output on stdout
        #[structopt(
            long = "tmpl",
            short = "t",
            default_value = "{{ name }}\t{{colored color }}\t(#{{ lastBuild.number }})"
        )]
        template: String,
    },
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "jencli",
    author = "",
    after_help = r#"
About Templates
Templates are defined using handlebars syntax. To view all fields available for a template, set jencli logs to debug with RUST_LOG=jencli=debug
A few helpers are available:
* colored: add color to build result and job status
* date: transform timestamps to UTC dates

About Configuration
Jenkins configuration (url, user, password, depth) can be overriden in a number of way, by decreasing order of priority:
* values passed as options
* values in environment variables
* .jencli.yaml file in path
* .jencli.yaml file in user home directory
"#
)]
pub struct ParamsOpt {
    /// Jenkins URL
    #[structopt(env = "JENKINS_URL", long = "url")]
    pub url: String,
    /// Jenkins user
    #[structopt(env = "JENKINS_USER", long = "user")]
    pub user: Option<String>,
    /// Jenkins password
    #[structopt(env = "JENKINS_PASSWORD", long = "password")]
    pub password: Option<String>,
    /// Amount of data retrieved from Jenkins
    #[structopt(env = "JENKINS_DEPTH", long = "depth", default_value = "1")]
    pub depth: u8,

    #[structopt(flatten)]
    pub command: CommandOpt,
}

#[derive(Debug, Deserialize)]
pub struct JenkinsSettings {
    pub url: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub depth: Option<u8>,
}

#[derive(Debug, Clone)]
struct SourceHocon {
    conf: Result<hocon::Hocon, ()>,
    path: String,
    required: bool,
}
impl SourceHocon {
    fn new(path: &str) -> Self {
        SourceHocon {
            conf: hocon::Hocon::load_from_file(path),
            path: String::from(path),
            required: true,
        }
    }
    fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

impl config::Source for SourceHocon {
    fn clone_into_box(&self) -> Box<Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<HashMap<String, Value>, ConfigError> {
        // Coerce the file contents to a string
        match &self.conf {
            Ok(hocon::Hocon::Hash(conf)) => Ok(conf
                .iter()
                .map(|(k, v)| (k.clone(), Value::new(Some(&self.path), v.as_string())))
                .collect()),

            _ => {
                if !self.required {
                    return Ok(HashMap::new());
                }

                Err(ConfigError::Message(String::from(
                    "error parsing configuration file",
                )))
            }
        }
    }
}

impl JenkinsSettings {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Result<Self, ConfigError> {
        let mut config = Config::new();

        let filename = ".jencli.conf";

        // Load file from home directory
        if let Some(mut home_conf) = dirs::home_dir() {
            home_conf.push(filename);
            config.merge(SourceHocon::new(home_conf.to_str().unwrap()).required(false))?;
        }

        // Load file from any folder in the path
        let mut current_dir = env::current_dir().unwrap();
        let mut pathes = vec![current_dir.clone().into_os_string()];
        while current_dir.pop() {
            pathes.push(current_dir.clone().into_os_string());
        }
        for dir in pathes.iter().rev() {
            let mut file = dir.clone();
            file.push(filename);
            config.merge(SourceHocon::new(file.to_str().unwrap()).required(false))?;
        }

        // Load from environment
        config.merge(Environment::with_prefix("jenkins").separator("_"))?;

        config.try_into()
    }
}

pub fn load() -> Result<ParamsOpt, ConfigError> {
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
    Ok(ParamsOpt::from_args())
}
