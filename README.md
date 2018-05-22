# jencli

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://travis-ci.org/mockersf/jencli.svg?branch=master)](https://travis-ci.org/mockersf/jencli)
[![Coverage Status](https://coveralls.io/repos/github/mockersf/jencli/badge.svg?branch=master)](https://coveralls.io/github/mockersf/jencli?branch=master)

A tool to work with Jenkins from the command line.

## Usage

```sh
$> jencli -h
jencli 0.1.0
A tool to work with Jenkins from the command line.

USAGE:
    jencli [OPTIONS] --url <url> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --depth <depth>          Amount of data retrieved from Jenkins [env: JENKINS_DEPTH=]  [default: 1]
        --password <password>    Jenkins password [env: JENKINS_PASSWORD=]
        --url <url>              Jenkins URL [env: JENKINS_URL=]
        --user <user>            Jenkins user [env: JENKINS_USER=]

SUBCOMMANDS:
    build      get informations about a build
    help       Prints this message or the help of the given subcommand(s)
    job        get informations about a job
    running    list running jobs
    search     search for a job
    trigger    trigger a job
```
