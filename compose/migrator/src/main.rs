use log::{error, info};
use migrator::Migrator;
use reqwest::{self, Url};
use std::env::{self};

mod client;
mod migrator;

fn main() {
    dotenv::dotenv().ok();
    env_logger::Builder::new()
        .filter_module("migrator", log::LevelFilter::Debug)
        .init();

    let resources: Vec<String> = env::var("resources")
        .expect("Expected 'resources' - A comma seperated list of resources")
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let origin = env::var("origin")
        .expect("Extected 'origin' the address of the service which is to be migrated");

    let successor = env::var("successor")
        .expect("Extected 'successor' the address of the service which is to be migrated");

    match (Url::parse(origin.as_str()), Url::parse(successor.as_str())) {
        (Ok(from), Ok(to)) => {
            let migrator = Migrator::new(from, to);
            migrator.migrate(resources);
        }
        _ => error!(
            "Invalid arguments - Either '{origin}' or '{successor}' could not be parsed as an url"
        ),
    }

    info!("Done");
}
