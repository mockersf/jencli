use config::{Config, ConfigError, Environment, File, FileFormat};
use std::env;
use std::ffi::OsString;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum CommandOpt {
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
        /// number of the build, will fetch lastBuild if not specified
        number: Option<u32>,
        /// format of the output on stdout
        #[structopt(long = "tmpl", short = "t",
                    default_value = "{{ fullDisplayName}} {{ result }} {{ timestamp }} ({{duration}}s)")]
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
