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
        /// display short informations about matching jobs
        #[structopt(long = "verbose", short = "v")]
        verbose: bool,
    },

    /// get informations about a job
    #[structopt(name = "job")]
    Job {
        /// exact name of the job
        name: String,
    },

    /// get informations about a build
    #[structopt(name = "build")]
    Build {
        /// name of the job
        name: String,
        /// number of the build, will fetch latest if not specified
        number: Option<u32>,
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
        /// check job status every X seconds
        #[structopt(long = "pooling", default_value = "10")]
        pooling: u32,
    },

    /// list running jobs
    #[structopt(name = "running")]
    Running {
        /// also list queued jobs
        #[structopt(long = "queued")]
        queued: bool,
    },
}

fn main() {
    let opt = CommandOpt::from_args();
    println!("{:?}", opt);
}
