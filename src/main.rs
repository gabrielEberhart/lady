use std::{env, process};

use crate::params::AppParams;
use crate::util::logger::{self, Severity};

mod params;
mod util;
mod database;

fn main() {
    // read args
    env::args().for_each(|arg| {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            "--version" => {
                println!("Lady version: {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            "--verbose" | "-v" => {
                // set log level to debug
                logger::set_level(Severity::Debug);
            }
            "--quiet" | "-q" => {
                // set log level to error
                logger::set_level(Severity::Error);
            }
            _ => {
                println!("Unknown argument: {}", arg);
                process::exit(1);
            }
        }
    });

    // load AppParams file
    let mut params: AppParams = match AppParams::from_json() {
        Ok(val) => val,      // load params from file
        Err(_) => AppParams::default(), // or create a new default param
    };

    println!("Hello, world!");
}

fn print_help() {
    println!("Usage: lady [OPTIONS]");
    println!("Options:");
    println!("  --help, -h\tShow this help message");
    println!("  --version\tShow the version of the application");
    println!("  --verbose, -v\tEnable verbose logging (debug level)");
    println!("  --quiet, -q\tEnable quiet logging (error level)");
}