use std::{
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

use crate::{bundle::Bundle, client::Client};
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

    fn aquire(&self, resources: Vec<String>) -> Vec<(String, Bundle)> {
        info!("Aquiring data...");
        resources
            .into_iter()
            .fold(Vec::<(String, Bundle)>::new(), |mut a, resource_type| {
                let json = self.get_history(&resource_type);

                let bundle = Bundle::from((resource_type.clone(), &json));
                a.push((resource_type, bundle));
                a
            })
    }

    fn get_history(&self, resource_type: &String) -> Value {
        let json = match &self.origin {
            // Fetch data from remote api
            Some(origin) => self
                .client
                .get(&Self::get_url(&origin, resource_type.as_str()))
                .unwrap()
                .json::<Value>()
                .unwrap(),

            // Fetch data from local storage
            None => {
                warn!("Environment variable 'successor' not found falling back to local data");

                let path = format!("./data/{}.json", resource_type);
                File::open(&path)
                    .and_then(|mut file| {
                        debug!("Read: {:?}", file);
                        let mut buf = String::new();
                        file.read_to_string(&mut buf).map(|_| buf)
                    })
                    .and_then(|s| serde_json::from_str::<Value>(s.as_str()).map_err(|e| e.into()))
                    .expect(format!("Expected: {}", path).as_str())
            }
        };
        json
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

    fn validate_data(&self, resources: Vec<(String, Bundle)>) {
        info!("Validating migration...");
        let t = resources.iter().all(|(resource_type, expected)| {
            let a: Value = self
                .client
                .get(Self::get_url(
                    &self.successor,
                    format!("{}", resource_type).as_str(),
                ))
                .unwrap()
                .json()
                .unwrap();

            let b: Value = self.get_history(resource_type);

            a.get("total").unwrap() == b.get("total").unwrap()
        });

        match t {
            true => info!("Success! 🥳"),
            false => error!("Failed! 👎"),
        }
    }

    fn migrate(&self, data: &Vec<(String, Bundle)>) {
        info!(
            "Migrating data from: {} to {}",
            self.origin
                .as_ref()
                .map(|u| u.to_string())
                .unwrap_or("./data/<resource>.json".to_string()),
            self.successor
        );

        data.iter().for_each(|(_, bundle)| {
            bundle.get_entries().for_each(|entry| {
                let versions: Vec<(&u64, &Value)> = entry.get_versions().collect();

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

                    let destination = self.successor.join(full_url.path()).unwrap();

                    let response = match method {
                        Method::PUT | Method::POST => {
                            let resource =
                                value.get("resource").expect("Could not extract resource");

                            self.client.put(destination, resource)
                        }
                        Method::DELETE => self.client.delete(destination),
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
