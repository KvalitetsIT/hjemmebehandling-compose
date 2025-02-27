use log::{error, info};
use migrator::Migrator;
use reqwest::{self, Url};
use std::env::{self};

mod client;
mod migrator;

fn main() {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let resources: Vec<String> = env::var("resources")
        .expect("Expected 'resources' - A comma seperated list of resources")
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let origin = match env::var("origin") {
        Ok(o) => match Url::parse(o.as_str()) {
            Ok(url) => Some(url),
            Err(e) => {
                panic!("{}", e);
            }
        },
        Err(_) => None,
    };
    let successor = env::var("successor")
        .map(|s| {
            Url::parse(s.as_str())
                .expect("Invalid arguments - '{successor}' could not be parsed as an url")
        })
        .expect("Extected 'successor' the address of the service which is to be migrated");

    let migrator = Migrator::new(origin, successor);
    migrator.start(resources);

    info!("Done");
}
