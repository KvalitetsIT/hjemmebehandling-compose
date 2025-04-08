use log::info;
use migrator::Migrator;
use reqwest::{self, Url};
use std::env::{self};

mod bundle;
mod client;
mod migrator;
mod record;

fn main() {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("reqwest", log::LevelFilter::Info)
        .filter_module("sqlx", log::LevelFilter::Info)
        .filter_module("hyper_util", log::LevelFilter::Info)
        .init();

    let resources: Vec<String> = env::var("RESOURCES")
        .expect("Expected 'RESOURCES' - A comma seperated list of resources")
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let origin = match env::var("ORIGIN") {
        Ok(o) => match Url::parse(o.as_str()) {
            Ok(url) => Some(url),
            Err(e) => {
                panic!("{}", e);
            }
        },
        Err(_) => None,
    };
    let successor = env::var("SUCCESSOR")
        .map(|s| {
            Url::parse(s.as_str())
                .expect("Invalid arguments - '{successor}' could not be parsed as an url")
        })
        .expect("Extected 'SUCCESSOR' the address of the service which is to be migrated");

    let migrator = Migrator::new(origin, successor);
    migrator.start(resources);

    migrator.quit();

    info!("Done");
}
