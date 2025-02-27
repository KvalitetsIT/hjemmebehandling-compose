use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::File,
    io::{Read, Write},
    str::FromStr,
    thread::sleep,
    time::Duration,
};

use log::{debug, error, info, warn};
use reqwest::{Method, Url};
use serde_json::Value;

use crate::client::Client;
pub struct Migrator {
    successor: Url,
    origin: Option<Url>,
    client: Client,
}
impl Migrator {
    pub fn new(origin: Option<Url>, successor: Url) -> Self {
        Self {
            origin,
            successor,
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
    ) -> Vec<(String, HashMap<String, HashMap<u64, Value>>)> {
        info!("Aquiring data...");
        resources.into_iter().fold(
            Vec::<(String, HashMap<String, HashMap<u64, Value>>)>::new(),
            |mut a, resource_type| {
                let json = match &self.origin {
                    Some(origin) => self
                        .client
                        .get(&Self::get_url(&origin, resource_type.as_str()))
                        .unwrap()
                        .json::<Value>()
                        .unwrap(),
                    None => {
                        warn!(
                            "Environment variable 'successor' not found falling back to local data"
                        );

                        let path = format!("./data/{}.json", resource_type);
                        File::open(&path)
                            .and_then(|mut file| {
                                debug!("Read: {:?}", file);
                                let mut buf = String::new();
                                file.read_to_string(&mut buf).map(|_| buf)
                            })
                            .and_then(|s| {
                                serde_json::from_str::<Value>(s.as_str()).map_err(|e| e.into())
                            })
                            .expect(format!("Expected: {}", path).as_str())
                    }
                };

                let empty: Vec<Value> = Vec::new();

                let entries = json
                    .get("entry")
                    .and_then(|entry| entry.as_array())
                    .unwrap_or(&empty);

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

                a.push((resource_type, resources));
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

    fn validate_data(&self, resources: Vec<(String, HashMap<String, HashMap<u64, Value>>)>) {
        info!("Validating migration...");
        resources.iter().for_each(|(resource_type, entries)| {
            entries.iter().for_each(|(entry, versions)| {
                let expected: u64 = versions.len() as u64;

                let actual = self
                    .client
                    .get(Self::get_url(
                        &self.successor,
                        format!("{}/{}", resource_type, entry).as_str(),
                    ))
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

    fn migrate(&self, data: &Vec<(String, HashMap<String, HashMap<u64, Value>>)>) {
        info!(
            "Migrating data from: {} to {}",
            self.origin
                .as_ref()
                .map(|u| u.to_string())
                .unwrap_or("./data/<resource>.json".to_string()),
            self.successor
        );

        data.iter().for_each(|(resource_type, entries)| {
            entries.values().for_each(|entry| {
                let mut versions: Vec<(&u64, &Value)> = entry.iter().collect();
                versions.sort_by_key(|k| k.0);

                versions.iter().for_each(|(_version, value)| {
                    let request = value
                        .get("request")
                        .expect("Could not extract request from entry");

                    let method = request
                        .get("method")
                        .and_then(|m| m.as_str())
                        .and_then(|method| Method::from_str(method).ok())
                        .expect("Could not extract method from request");

                    let full_url: Url = value
                        .get("fullUrl")
                        .and_then(|u| u.as_str())
                        .and_then(|u| Url::parse(u).ok())
                        .expect("Could not extract field 'fullUrl'");

                    let url: Url = request
                        .get("url")
                        .and_then(|u| u.as_str())
                        .and_then(|u| Url::parse(u).ok())
                        .expect("Could not extract field 'url' from request");

                    let destination = self.successor.join(full_url.path()).unwrap();

                    let response = match method {
                        Method::PUT | Method::POST => {
                            let resource =
                                value.get("resource").expect("Could not extract resource");

                            self.client.put(destination, resource)
                        }
                        Method::DELETE => self
                            .client
                            .delete(self.successor.join(full_url.path()).unwrap()),
                        _ => Err(String::from("Expected method {}, method").into()),
                    };

                    match response {
                        Ok(response) => {
                            if !response.status().is_success() {
                                error!(
                                    "{}\n{}",
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
