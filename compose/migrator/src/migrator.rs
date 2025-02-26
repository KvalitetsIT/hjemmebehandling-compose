use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{Error, Read, Write},
    str::FromStr,
    thread::sleep,
    time::Duration,
};

use dotenv::Result;
use log::{error, info};
use reqwest::{Method, Url};
use serde_json::Value;

use crate::client::Client;
pub struct Migrator {
    origin: Url,
    successor: Url,
    client: Client,
}
impl Migrator {
    pub fn new(from: Url, to: Url) -> Self {
        Self {
            origin: from,
            successor: to,
            client: Client::new(),
        }
    }

    pub fn start(&self, resources: Vec<String>) {
        let data = self.aquire(resources);
        self.migrate(&data);

        Self::wait();

        self.validate_data(data);
    }

    fn aquire(
        &self,
        resources: Vec<String>,
    ) -> HashMap<String, HashMap<String, HashMap<u64, Value>>> {
        info!("Aquiring data...");
        resources.into_iter().fold(
            HashMap::<String, HashMap<String, HashMap<u64, Value>>>::new(),
            |mut a, resource_type| {
                // let entries = self
                //     .client
                //     .get(&Self::get_url(&self.origin, resource_type.as_str()))
                //     .unwrap()
                //     .json::<Value>()
                //     .unwrap();

                let entries = File::open(format!("./data/{}.json", resource_type))
                    .and_then(|mut file| {
                        info!("Read: {:?}", file);
                        let mut buf = String::new();
                        file.read_to_string(&mut buf).map(|_| buf)
                    })
                    .and_then(|s| serde_json::from_str::<Value>(s.as_str()).map_err(|e| e.into()))
                    .unwrap();

                let entries = entries
                    .get("entry")
                    .expect("The expected field 'entry' was not found")
                    .as_array()
                    .expect("Could not parse 'entry' as array");

                let resources = entries.iter().fold(
                    HashMap::<String, HashMap<u64, Value>>::new(),
                    |mut acc, entry| {
                        let request = entry.get("request").unwrap();
                        let version = u64::from_str(
                            request
                                .get("url")
                                .and_then(|url| url.as_str())
                                .unwrap()
                                .split('/')
                                .last()
                                .unwrap(),
                        )
                        .unwrap();

                        let id = entry
                            .get("fullUrl")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .split('/')
                            .last()
                            .unwrap()
                            .to_string();

                        acc.entry(id)
                            .or_insert_with(HashMap::new)
                            .insert(version, entry.clone()); // Use `.clone()` since `entries` is borrowed
                        acc
                    },
                );

                a.insert("resources".to_string(), resources);
                a
            },
        )
    }

    fn get_url(origin: &Url, resource_type: &str) -> String {
        let path = format!("/fhir/{}/_history", resource_type);
        let origin = origin.join(path.as_str()).unwrap();
        origin.to_string()
    }

    #[allow(dead_code)]
    fn create_reponse_file(name: &str, value: &Value) {
        const ERROR_MESSAGE: &str = "Could not create file";
        let path = format!("./resources/{}.json", name);
        File::create(path)
            .unwrap()
            .write_all(serde_json::to_string_pretty(value).unwrap().as_bytes())
            .expect(ERROR_MESSAGE);
    }

    fn to_value(response: reqwest::blocking::Response) -> Value {
        response.json::<Value>().unwrap()
    }

    fn wait() {
        let secs: u64 = env::var("wait")
            .map(|secs| {
                secs.parse::<u64>()
                    .expect("Could not parse 'wait' as a u64")
            })
            .unwrap_or(20);
        let duration = Duration::from_secs(secs);

        info!(
            "Waiting {} seconds before verifying migration...",
            duration.as_secs()
        );
        sleep(duration);
    }

    fn validate_data(&self, resources: HashMap<String, HashMap<String, HashMap<u64, Value>>>) {
        info!("Validating migration...");
        resources.iter().for_each(|(resource_type, entries)| {
            entries.iter().for_each(|(entry, versions)| {
                let actual: u64 = versions.len() as u64;

                let expected = self
                    .client
                    .get(Self::get_url(&self.successor, resource_type))
                    .map(|response| Self::to_value(response))
                    .unwrap()
                    .get("total")
                    .expect(
                        format!(
                            "Could not aquire field 'total' for resource: {}",
                            resource_type
                        )
                        .as_str(),
                    )
                    .as_u64()
                    .unwrap();

                match actual == expected {
                    true => info!("{}:{} - {}/{}", resource_type, entry, actual, expected),
                    false => error!("{}:{} - {}/{}", resource_type, entry, actual, expected),
                }
            });
        });
    }

    fn migrate(&self, data: &HashMap<String, HashMap<String, HashMap<u64, Value>>>) {
        info!("Migrating data from: {} to {}", self.origin, self.successor);
        data.values().for_each(|entries| {
            entries.values().for_each(|entry| {
                entry.values().for_each(|version| {
                    let request = version
                        .get("request")
                        .expect("Could not extract request from entry");

                    let method = request
                        .get("method")
                        .expect("Could not extract method from request")
                        .as_str()
                        .and_then(|method| Method::from_str(method).ok())
                        .unwrap();

                    let full_url: Url = version
                        .get("fullUrl")
                        .and_then(|u| u.as_str())
                        .and_then(|u| Url::parse(u).ok())
                        .unwrap();

                    let destination = self.successor.join(full_url.path()).unwrap();

                    let response = match method {
                        Method::PUT | Method::POST => {
                            let resource =
                                version.get("resource").expect("Could not extract resource");

                            self.client.put(destination, resource)
                        }
                        Method::DELETE => self
                            .client
                            .delete(self.successor.join(full_url.path()).unwrap()),
                        _ => Err(String::from("Expected method {}, method").into()),
                    };

                    match response {
                        Ok(response) => {
                            if response.status().is_success() {
                                info!(
                                    "{}: {}",
                                    response.status(),
                                    response.text().unwrap_or("".to_string())
                                )
                            } else {
                                error!(
                                    "{}: {}",
                                    response.status(),
                                    response.text().unwrap_or("".to_string())
                                )
                            }
                        }
                        Err(error) => error!("{:#?}", error),
                    }
                });
            });
        });
    }
}
