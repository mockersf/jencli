extern crate jenkins_api;

#[macro_use]
extern crate structopt;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "jencli")]
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

fn main() {
    let opt = CommandOpt::from_args();
    println!("{:?}", opt);
}
